mod draw;
mod input;
mod net;
mod state;

use crate::args::Args;
use crate::render::{messages_to_text_cached, RenderCacheEntry, RenderTheme};
use crate::session::save_session;
use crate::types::Message;
use crossterm::event::{self, Event, MouseEventKind};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use draw::{inner_height, inner_width, layout_chunks, redraw};
use input::handle_key;
use net::{request_llm_stream, LlmEvent};
use state::{App, Focus};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

pub fn run(
    args: Args,
    api_key: String,
    _cfg: Option<crate::config::Config>,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let base_url = args.base_url.trim_end_matches('/');
    let url = format!("{base_url}/chat/completions");

    let mut app = App::new(&args.system);
    let mut last_session_id: Option<String> = None;
    let mut render_cache: Vec<RenderCacheEntry> = Vec::new();
    let (tx, rx) = mpsc::channel::<LlmEvent>();

    loop {
        let size = terminal.size()?;
        let (msg_area, input_area) = layout_chunks(size);
        let msg_width = inner_width(msg_area, 1);
        let timer_text = if app.busy {
            let secs = app
                .busy_since
                .map(|t| t.elapsed().as_secs())
                .unwrap_or(0);
            format_timer(secs)
        } else {
            String::new()
        };
        let label_suffixes = build_label_suffixes(&app, &timer_text);
        let text = messages_to_text_cached(
            &app.messages,
            msg_width,
            theme,
            &label_suffixes,
            app.pending_assistant,
            &mut render_cache,
        );
        let total_lines = text.lines.len();
        let view_height = inner_height(msg_area, 0) as usize;
        let max_scroll = total_lines.saturating_sub(view_height).min(u16::MAX as usize) as u16;

        if app.follow {
            app.scroll = max_scroll;
        } else if app.scroll > max_scroll {
            app.scroll = max_scroll;
        }
        redraw(&mut terminal, &app, theme, &text)?;
        if app.focus == Focus::Input && !app.busy {
            terminal.show_cursor()?;
        } else {
            terminal.hide_cursor()?;
        }

        if let Some(line) = app.pending_send.take() {
            app.busy = true;
            app.busy_since = Some(Instant::now());
            app.messages.push(Message {
                role: "user".to_string(),
                content: line,
            });
            let idx = app.messages.len();
            app.messages.push(Message {
                role: "assistant".to_string(),
                content: String::new(),
            });
            app.pending_assistant = Some(idx);
            app.pending_reasoning = None;
            app.stream_buffer.clear();
            let label_suffixes = build_label_suffixes(&app, &format_timer(0));
            let text = messages_to_text_cached(
                &app.messages,
                msg_width,
                theme,
                &label_suffixes,
                app.pending_assistant,
                &mut render_cache,
            );
            redraw(&mut terminal, &app, theme, &text)?;

            let url = url.clone();
            let api_key = api_key.clone();
            let model = args.model.clone();
            let show_reasoning = args.show_reasoning;
            let messages = app.messages.clone();
            let tx = tx.clone();
            thread::spawn(move || {
                request_llm_stream(&url, &api_key, &model, show_reasoning, &messages, tx);
            });
        }

        if app.busy {
            let mut done = false;
            while let Some(event) = try_recv(&rx) {
                let elapsed = app
                    .busy_since
                    .map(|t| t.elapsed().as_secs())
                    .unwrap_or(0);
                if handle_stream_event(&mut app, event, elapsed) {
                    done = true;
                    break;
                }
            }
            if done {
                app.busy = false;
                app.busy_since = None;
                drain_events()?;
            }
        }

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(key, &mut app, &mut last_session_id)? {
                        break;
                    }
                }
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::Down(_) => {
                        if point_in_rect(m.column, m.row, input_area) {
                            app.focus = Focus::Input;
                            app.cursor = app.input.len();
                        } else if point_in_rect(m.column, m.row, msg_area) {
                            app.focus = Focus::Chat;
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        app.scroll = app.scroll.saturating_sub(3);
                        app.follow = false;
                        app.focus = Focus::Chat;
                    }
                    MouseEventKind::ScrollDown => {
                        app.scroll = app.scroll.saturating_add(3);
                        app.follow = false;
                        app.focus = Focus::Chat;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if last_session_id.is_none() {
        if let Ok(id) = save_session(&app.messages) {
            last_session_id = Some(id);
        }
    }
    if let Some(id) = last_session_id {
        println!("回放指令：deepchat --resume {id}");
    }

    Ok(())
}

fn try_recv(rx: &Receiver<LlmEvent>) -> Option<LlmEvent> {
    rx.try_recv().ok()
}

fn handle_stream_event(app: &mut App, event: LlmEvent, elapsed_secs: u64) -> bool {
    match event {
        LlmEvent::Chunk(s) => {
            app.stream_buffer.push_str(&s);
            flush_completed_lines(app);
            false
        }
        LlmEvent::Reasoning(s) => {
            append_reasoning(app, &s);
            false
        }
        LlmEvent::Error(err) => {
            set_pending_assistant_content(app, &err);
            app.pending_assistant = None;
            app.pending_reasoning = None;
            app.stream_buffer.clear();
            true
        }
        LlmEvent::Done { usage } => {
            flush_remaining_buffer(app);
            let stats = format_stats(usage.as_ref(), elapsed_secs);
            if let Some(idx) = app.pending_assistant.take() {
                app.assistant_stats = Some((idx, stats));
            }
            app.pending_reasoning = None;
            true
        }
    }
}

fn format_stats(usage: Option<&crate::types::Usage>, elapsed_secs: u64) -> String {
    let time = format_timer(elapsed_secs);
    let tokens = if let Some(u) = usage {
        let p = u.prompt_tokens.unwrap_or(0);
        let c = u.completion_tokens.unwrap_or(0);
        let t = u.total_tokens.unwrap_or(p + c);
        format!("{p}/{c}/{t}")
    } else {
        "n/a".to_string()
    };
    format!("{time} · tokens: {tokens}")
}

fn flush_completed_lines(app: &mut App) {
    while let Some(pos) = app.stream_buffer.find('\n') {
        let line: String = app.stream_buffer.drain(..=pos).collect();
        append_to_pending_assistant(app, &line);
    }
}

fn flush_remaining_buffer(app: &mut App) {
    if !app.stream_buffer.is_empty() {
        let rest = app.stream_buffer.clone();
        app.stream_buffer.clear();
        append_to_pending_assistant(app, &rest);
    }
}

fn append_to_pending_assistant(app: &mut App, text: &str) {
    if let Some(idx) = app.pending_assistant {
        if let Some(msg) = app.messages.get_mut(idx) {
            msg.content.push_str(text);
        } else {
            app.messages.push(Message {
                role: "assistant".to_string(),
                content: text.to_string(),
            });
            app.pending_assistant = Some(app.messages.len().saturating_sub(1));
        }
    } else {
        app.messages.push(Message {
            role: "assistant".to_string(),
            content: text.to_string(),
        });
        app.pending_assistant = Some(app.messages.len().saturating_sub(1));
    }
}

fn append_reasoning(app: &mut App, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    if let Some(idx) = app.pending_reasoning {
        if let Some(msg) = app.messages.get_mut(idx) {
            msg.content.push_str(text);
            return;
        }
    }
    let insert_at = app.pending_assistant.unwrap_or(app.messages.len());
    app.messages.insert(
        insert_at,
        Message {
            role: "assistant".to_string(),
            content: format!("推理> {text}"),
        },
    );
    app.pending_reasoning = Some(insert_at);
    if let Some(idx) = app.pending_assistant {
        app.pending_assistant = Some(idx.saturating_add(1));
    }
}

fn set_pending_assistant_content(app: &mut App, content: &str) {
    if let Some(idx) = app.pending_assistant {
        if let Some(msg) = app.messages.get_mut(idx) {
            msg.content = content.to_string();
            return;
        }
    }
    app.messages.push(Message {
        role: "assistant".to_string(),
        content: content.to_string(),
    });
    app.pending_assistant = Some(app.messages.len().saturating_sub(1));
}

fn build_label_suffixes(app: &App, timer_text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    if let Some((idx, stats)) = &app.assistant_stats {
        out.push((*idx, stats.clone()));
    }
    if app.busy {
        if let Some(idx) = app.pending_assistant {
            if !timer_text.is_empty() {
                out.push((idx, timer_text.to_string()));
            }
        }
    }
    out
}

fn format_timer(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else {
        let m = secs / 60;
        let s = secs % 60;
        format!("{}m{:02}s", m, s)
    }
}

fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

fn drain_events() -> Result<(), Box<dyn std::error::Error>> {
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }
    Ok(())
}
