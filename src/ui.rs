mod draw;
mod input;
mod logic;
mod net;
mod perf;
mod state;

use crate::args::Args;
use crate::render::{messages_to_viewport_text_cached, RenderCacheEntry, RenderTheme};
use crate::session::save_session;
use crate::types::Message;
use crossterm::event::{self, Event, MouseEventKind};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use draw::{inner_height, inner_width, layout_chunks, redraw, scrollbar_area};
use input::handle_key;
use logic::{
    build_label_suffixes, drain_events, format_timer, handle_stream_event, point_in_rect,
    scroll_from_mouse, try_recv,
};
use net::{request_llm_stream, LlmEvent};
use perf::seed_perf_messages;
use state::{App, Focus};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self};
use std::sync::mpsc;
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
    if args.perf {
        seed_perf_messages(&mut app);
    }
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
        let view_height = inner_height(msg_area, 0) as u16;
        let label_suffixes = build_label_suffixes(&app, &timer_text);
        let prev_scroll = app.scroll;
        let (mut text, total_lines) = messages_to_viewport_text_cached(
            &app.messages,
            msg_width,
            theme,
            &label_suffixes,
            app.pending_assistant,
            app.scroll,
            view_height,
            &mut render_cache,
        );
        let max_scroll = total_lines
            .saturating_sub(view_height as usize)
            .min(u16::MAX as usize) as u16;

        if app.follow {
            app.scroll = max_scroll;
        } else if app.scroll > max_scroll {
            app.scroll = max_scroll;
        }
        if app.scroll != prev_scroll {
            let (retext, _) = messages_to_viewport_text_cached(
                &app.messages,
                msg_width,
                theme,
                &label_suffixes,
                app.pending_assistant,
                app.scroll,
                view_height,
                &mut render_cache,
            );
            text = retext;
        }
        redraw(&mut terminal, &app, theme, &text, total_lines)?;
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
            let view_height = inner_height(msg_area, 0) as u16;
            let (text, total_lines) = messages_to_viewport_text_cached(
                &app.messages,
                msg_width,
                theme,
                &label_suffixes,
                app.pending_assistant,
                app.scroll,
                view_height,
                &mut render_cache,
            );
            redraw(&mut terminal, &app, theme, &text, total_lines)?;

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
                        let scroll_area = scrollbar_area(msg_area);
                        if point_in_rect(m.column, m.row, scroll_area)
                            && total_lines > view_height as usize
                        {
                            app.scrollbar_dragging = true;
                            app.follow = false;
                            app.scroll = scroll_from_mouse(
                                total_lines,
                                view_height,
                                scroll_area,
                                m.row,
                            );
                            app.focus = Focus::Chat;
                            continue;
                        }
                        if point_in_rect(m.column, m.row, input_area) {
                            app.focus = Focus::Input;
                            app.cursor = app.input.len();
                        } else if point_in_rect(m.column, m.row, msg_area) {
                            app.focus = Focus::Chat;
                        }
                    }
                    MouseEventKind::Up(_) => {
                        app.scrollbar_dragging = false;
                    }
                    MouseEventKind::Drag(_) => {
                        if app.scrollbar_dragging {
                            let scroll_area = scrollbar_area(msg_area);
                            app.follow = false;
                            app.scroll = scroll_from_mouse(
                                total_lines,
                                view_height,
                                scroll_area,
                                m.row,
                            );
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
