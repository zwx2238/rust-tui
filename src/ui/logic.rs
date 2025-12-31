use crate::types::Message;
use crate::ui::net::LlmEvent;
use crate::ui::state::App;

pub fn handle_stream_event(app: &mut App, event: LlmEvent, elapsed_secs: u64) -> bool {
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

pub fn build_label_suffixes(app: &App, timer_text: &str) -> Vec<(usize, String)> {
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

pub fn format_timer(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else {
        let m = secs / 60;
        let s = secs % 60;
        format!("{}m{:02}s", m, s)
    }
}

pub fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

pub fn scroll_from_mouse(
    total_lines: usize,
    view_height: u16,
    scroll_area: ratatui::layout::Rect,
    mouse_y: u16,
) -> u16 {
    if total_lines <= view_height as usize || scroll_area.height <= 1 {
        return 0;
    }
    let max_scroll = total_lines
        .saturating_sub(view_height as usize)
        .min(u16::MAX as usize) as u16;
    let y = mouse_y.saturating_sub(scroll_area.y);
    let track = scroll_area.height.saturating_sub(1).max(1);
    let ratio = y.min(track) as f32 / track as f32;
    let scroll = (ratio * max_scroll as f32).round() as u16;
    scroll.min(max_scroll)
}

pub fn tab_label(idx: usize) -> String {
    format!(" {} ", idx + 1)
}

pub fn tab_label_width(idx: usize) -> u16 {
    tab_label(idx).len() as u16
}

pub fn drain_events() -> Result<(), Box<dyn std::error::Error>> {
    while crossterm::event::poll(std::time::Duration::from_millis(0))? {
        let _ = crossterm::event::read()?;
    }
    Ok(())
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
            app.dirty_indices.push(idx);
        } else {
            app.messages.push(Message {
                role: "assistant".to_string(),
                content: text.to_string(),
            });
            app.pending_assistant = Some(app.messages.len().saturating_sub(1));
            if let Some(idx) = app.pending_assistant {
                app.dirty_indices.push(idx);
            }
        }
    } else {
        app.messages.push(Message {
            role: "assistant".to_string(),
            content: text.to_string(),
        });
        app.pending_assistant = Some(app.messages.len().saturating_sub(1));
        if let Some(idx) = app.pending_assistant {
            app.dirty_indices.push(idx);
        }
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
    app.dirty_indices.push(insert_at);
    app.cache_shift = Some(insert_at);
    if let Some(idx) = app.pending_assistant {
        app.pending_assistant = Some(idx.saturating_add(1));
    }
}

fn set_pending_assistant_content(app: &mut App, content: &str) {
    if let Some(idx) = app.pending_assistant {
        if let Some(msg) = app.messages.get_mut(idx) {
            msg.content = content.to_string();
            app.dirty_indices.push(idx);
            return;
        }
    }
    app.messages.push(Message {
        role: "assistant".to_string(),
        content: content.to_string(),
    });
    app.pending_assistant = Some(app.messages.len().saturating_sub(1));
    if let Some(idx) = app.pending_assistant {
        app.dirty_indices.push(idx);
    }
}
