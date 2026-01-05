use crate::render::RenderTheme;
use crate::ui::logic::{StreamAction, handle_stream_event};
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::TabState;

pub fn collect_stream_events(
    rx: &std::sync::mpsc::Receiver<UiEvent>,
    tabs: &mut [TabState],
    theme: &RenderTheme,
) -> (Vec<usize>, Vec<(usize, Vec<crate::types::ToolCall>)>) {
    let mut done_tabs: Vec<usize> = Vec::new();
    let mut tool_queue: Vec<(usize, Vec<crate::types::ToolCall>)> = Vec::new();
    while let Ok(event) = rx.try_recv() {
        handle_stream_event_for_tab(event, tabs, theme, &mut done_tabs, &mut tool_queue);
    }
    (done_tabs, tool_queue)
}

fn handle_stream_event_for_tab(
    ui_event: UiEvent,
    tabs: &mut [TabState],
    theme: &RenderTheme,
    done_tabs: &mut Vec<usize>,
    tool_queue: &mut Vec<(usize, Vec<crate::types::ToolCall>)>,
) {
    let UiEvent {
        tab,
        request_id,
        event,
    } = ui_event;
    let Some(tab_state) = tabs.get_mut(tab) else {
        return;
    };
    if !is_active_request(tab_state, request_id) {
        return;
    }
    let elapsed = elapsed_millis(tab_state);
    match handle_stream_event(&mut tab_state.app, event, elapsed) {
        StreamAction::Done => done_tabs.push(tab),
        StreamAction::ToolCalls(calls) => tool_queue.push((tab, calls)),
        StreamAction::None => {}
    }
    tab_state.apply_cache_shift(theme);
}

fn is_active_request(tab_state: &TabState, request_id: u64) -> bool {
    let active_id = tab_state.app.active_request.as_ref().map(|h| h.id);
    active_id == Some(request_id)
}

fn elapsed_millis(tab_state: &TabState) -> u64 {
    tab_state
        .app
        .busy_since
        .map(|t| t.elapsed().as_millis() as u64)
        .unwrap_or(0)
}
