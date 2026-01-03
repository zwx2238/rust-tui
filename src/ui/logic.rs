use crate::types::ToolCall;
use crate::types::{Message, ROLE_ASSISTANT};
use crate::ui::net::LlmEvent;
use crate::ui::scroll::max_scroll_u16;
use crate::ui::state::App;

pub enum StreamAction {
    None,
    Done,
    ToolCalls(Vec<ToolCall>),
}

pub fn handle_stream_event(app: &mut App, event: LlmEvent, elapsed_ms: u64) -> StreamAction {
    match event {
        LlmEvent::Chunk(s) => handle_chunk(app, &s),
        LlmEvent::Error(err) => handle_error(app, &err),
        LlmEvent::Done { usage } => handle_done(app, usage.as_ref(), elapsed_ms),
        LlmEvent::ToolCalls { calls, usage } => {
            handle_tool_calls(app, calls, usage.as_ref(), elapsed_ms)
        }
    }
}

fn handle_chunk(app: &mut App, chunk: &str) -> StreamAction {
    app.stream_buffer.push_str(chunk);
    flush_completed_lines(app);
    StreamAction::None
}

fn handle_error(app: &mut App, err: &str) -> StreamAction {
    set_pending_assistant_content(app, err);
    clear_stream_state(app);
    StreamAction::Done
}

fn handle_done(
    app: &mut App,
    usage: Option<&crate::types::Usage>,
    elapsed_ms: u64,
) -> StreamAction {
    flush_remaining_buffer(app);
    record_assistant_stats(app, usage, elapsed_ms);
    update_usage_totals(app, usage);
    clear_stream_state(app);
    StreamAction::Done
}

fn handle_tool_calls(
    app: &mut App,
    calls: Vec<ToolCall>,
    usage: Option<&crate::types::Usage>,
    elapsed_ms: u64,
) -> StreamAction {
    flush_remaining_buffer(app);
    attach_tool_calls(app, calls.clone(), elapsed_ms, usage);
    update_usage_totals(app, usage);
    clear_stream_state(app);
    StreamAction::ToolCalls(calls)
}

fn attach_tool_calls(
    app: &mut App,
    calls: Vec<ToolCall>,
    elapsed_ms: u64,
    usage: Option<&crate::types::Usage>,
) {
    let stats = format_stats(usage, elapsed_ms);
    if let Some(idx) = app.pending_assistant.take() {
        if let Some(msg) = app.messages.get_mut(idx) {
            msg.tool_calls = Some(calls);
        }
        app.assistant_stats.insert(idx, stats);
    }
}

fn record_assistant_stats(app: &mut App, usage: Option<&crate::types::Usage>, elapsed_ms: u64) {
    let stats = format_stats(usage, elapsed_ms);
    if let Some(idx) = app.pending_assistant.take() {
        app.assistant_stats.insert(idx, stats);
    }
}

fn update_usage_totals(app: &mut App, usage: Option<&crate::types::Usage>) {
    let Some(u) = usage else {
        return;
    };
    let p = u.prompt_tokens.unwrap_or(0);
    let c = u.completion_tokens.unwrap_or(0);
    let t = u.total_tokens.unwrap_or(p + c);
    app.total_prompt_tokens = app.total_prompt_tokens.saturating_add(p);
    app.total_completion_tokens = app.total_completion_tokens.saturating_add(c);
    app.total_tokens = app.total_tokens.saturating_add(t);
}

fn clear_stream_state(app: &mut App) {
    app.pending_assistant = None;
    app.pending_reasoning = None;
    app.stream_buffer.clear();
    app.active_request = None;
    app.busy = false;
    app.busy_since = None;
}

pub fn stop_stream(app: &mut App) -> bool {
    let Some(handle) = app.active_request.take() else {
        return false;
    };
    handle.cancel();
    flush_remaining_buffer(app);
    if let Some(idx) = app.pending_assistant.take() {
        app.assistant_stats.insert(idx, "已终止".to_string());
        app.dirty_indices.push(idx);
    }
    app.pending_reasoning = None;
    app.stream_buffer.clear();
    app.busy = false;
    app.busy_since = None;
    app.follow = true;
    true
}

pub fn timer_text(app: &App) -> String {
    if !app.busy {
        return String::new();
    }
    let ms = app
        .busy_since
        .map(|t| t.elapsed().as_millis() as u64)
        .unwrap_or(0);
    format_timer(ms)
}

pub fn build_label_suffixes(app: &App, timer_text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (idx, stats) in &app.assistant_stats {
        out.push((*idx, stats.clone()));
    }
    if app.busy
        && let Some(idx) = app.pending_assistant
        && !timer_text.is_empty()
    {
        out.push((idx, timer_text.to_string()));
    }
    out
}

pub fn format_timer(ms: u64) -> String {
    let secs = ms as f64 / 1000.0;
    if secs < 60.0 {
        format!("{:.1}s", secs)
    } else {
        let m = (secs / 60.0).floor() as u64;
        let s = secs - (m as f64 * 60.0);
        format!("{}m{:04.1}s", m, s)
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
    let max_scroll = max_scroll_u16(total_lines, view_height);
    let y = mouse_y.saturating_sub(scroll_area.y);
    let track = scroll_area.height.saturating_sub(1).max(1);
    let ratio = y.min(track) as f32 / track as f32;
    let scroll = (ratio * max_scroll as f32).round() as u16;
    scroll.min(max_scroll)
}

pub fn drain_events() -> Result<(), Box<dyn std::error::Error>> {
    while crossterm::event::poll(std::time::Duration::from_millis(0))? {
        let _ = crossterm::event::read()?;
    }
    Ok(())
}

fn format_stats(usage: Option<&crate::types::Usage>, elapsed_ms: u64) -> String {
    let time = format_timer(elapsed_ms);
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
                role: ROLE_ASSISTANT.to_string(),
                content: text.to_string(),
                tool_call_id: None,
                tool_calls: None,
            });
            app.pending_assistant = Some(app.messages.len().saturating_sub(1));
            if let Some(idx) = app.pending_assistant {
                app.dirty_indices.push(idx);
            }
        }
    } else {
        app.messages.push(Message {
            role: ROLE_ASSISTANT.to_string(),
            content: text.to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        app.pending_assistant = Some(app.messages.len().saturating_sub(1));
        if let Some(idx) = app.pending_assistant {
            app.dirty_indices.push(idx);
        }
    }
}

fn set_pending_assistant_content(app: &mut App, content: &str) {
    if let Some(idx) = app.pending_assistant
        && let Some(msg) = app.messages.get_mut(idx)
    {
        msg.content = content.to_string();
        app.dirty_indices.push(idx);
        return;
    }
    app.messages.push(Message {
        role: ROLE_ASSISTANT.to_string(),
        content: content.to_string(),
        tool_call_id: None,
        tool_calls: None,
    });
    app.pending_assistant = Some(app.messages.len().saturating_sub(1));
    if let Some(idx) = app.pending_assistant {
        app.dirty_indices.push(idx);
    }
}
