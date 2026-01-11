use crate::render::RenderTheme;
use crate::framework::widget_system::runtime::events::{LlmEvent, UiEvent};
use crate::framework::widget_system::runtime::logic::{StreamAction, handle_stream_event};
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::hooks::{EVENT_LLM_DONE, EVENT_LLM_ERROR, EVENT_LLM_TOOL_CALLS, run_hooks};

type ToolQueue = Vec<(usize, Vec<crate::types::ToolCall>)>;
pub(crate) type StreamCollectResult = (usize, Vec<usize>, ToolQueue);

pub fn collect_stream_events_from_batch(
    llm_events: &mut Vec<UiEvent>,
    tabs: &mut [TabState],
    theme: &RenderTheme,
) -> StreamCollectResult {
    let processed = llm_events.len();
    let mut done_tabs: Vec<usize> = Vec::new();
    let mut tool_queue: ToolQueue = Vec::new();
    for event in llm_events.drain(..) {
        handle_stream_event_for_tab(event, tabs, theme, &mut done_tabs, &mut tool_queue);
    }
    (processed, done_tabs, tool_queue)
}

fn handle_stream_event_for_tab(
    ui_event: UiEvent,
    tabs: &mut [TabState],
    theme: &RenderTheme,
    done_tabs: &mut Vec<usize>,
    tool_queue: &mut Vec<(usize, Vec<crate::types::ToolCall>)>,
) {
    let UiEvent { tab, request_id, event } = ui_event;
    let Some(tab_idx) = tab_index_for(&tab, tabs) else {
        return;
    };
    let Some(tab_state) = tabs.get_mut(tab_idx) else {
        return;
    };
    if !is_active_request(tab_state, request_id) {
        return;
    }
    apply_stream_event(tab_state, tab_idx, event, theme, done_tabs, tool_queue);
}

fn tab_index_for(tab: &str, tabs: &[TabState]) -> Option<usize> {
    tabs.iter().position(|t| t.conversation_id == tab)
}

fn apply_stream_event(
    tab_state: &mut TabState,
    tab_idx: usize,
    event: LlmEvent,
    theme: &RenderTheme,
    done_tabs: &mut Vec<usize>,
    tool_queue: &mut Vec<(usize, Vec<crate::types::ToolCall>)>,
) {
    let hook_call = collect_hook_call(tab_state, tab_idx, &event);
    let call_done_on_error = matches!(&event, LlmEvent::Error(_));
    let elapsed = elapsed_millis(tab_state);
    let action = handle_stream_event(&mut tab_state.app, event, elapsed);
    apply_stream_action(action, tab_idx, done_tabs, tool_queue);
    tab_state.apply_cache_shift(theme);
    fire_llm_hooks(tab_state, hook_call, call_done_on_error);
}

fn collect_hook_call(
    tab_state: &TabState,
    tab_idx: usize,
    event: &LlmEvent,
) -> Option<(&'static str, Vec<(String, String)>)> {
    if tab_state.app.hooks.is_empty() {
        return None;
    }
    hook_for_llm_event(event, tab_state, tab_idx)
}

fn apply_stream_action(
    action: StreamAction,
    tab_idx: usize,
    done_tabs: &mut Vec<usize>,
    tool_queue: &mut Vec<(usize, Vec<crate::types::ToolCall>)>,
) {
    match action {
        StreamAction::Done => done_tabs.push(tab_idx),
        StreamAction::ToolCalls(calls) => tool_queue.push((tab_idx, calls)),
        StreamAction::None => {}
    }
}

fn fire_llm_hooks(
    tab_state: &TabState,
    hook_call: Option<(&'static str, Vec<(String, String)>)>,
    call_done_on_error: bool,
) {
    let Some((hook_event, vars)) = hook_call else {
        return;
    };
    run_hooks(&tab_state.app.hooks, hook_event, vars.clone());
    if call_done_on_error {
        run_hooks(&tab_state.app.hooks, EVENT_LLM_DONE, vars);
    }
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

fn hook_for_llm_event(
    event: &LlmEvent,
    tab_state: &TabState,
    tab_idx: usize,
) -> Option<(&'static str, Vec<(String, String)>)> {
    let mut vars = base_hook_vars(tab_state, tab_idx);
    match event {
        LlmEvent::Done { usage } => {
            vars.extend(usage_vars(usage.as_ref()));
            Some((EVENT_LLM_DONE, vars))
        }
        LlmEvent::ToolCalls { calls, usage } => {
            vars.extend(tool_calls_vars(calls));
            vars.extend(usage_vars(usage.as_ref()));
            Some((EVENT_LLM_TOOL_CALLS, vars))
        }
        LlmEvent::Error(err) => {
            vars.push(("HOOK_ERROR".to_string(), err.to_string()));
            Some((EVENT_LLM_ERROR, vars))
        }
        _ => None,
    }
}

fn base_hook_vars(tab_state: &TabState, tab_idx: usize) -> Vec<(String, String)> {
    vec![
        ("HOOK_TAB_ID".to_string(), tab_idx.to_string()),
        ("HOOK_CONV_ID".to_string(), tab_state.conversation_id.clone()),
        ("HOOK_MODEL".to_string(), tab_state.app.model_key.clone()),
        ("HOOK_PROMPT".to_string(), tab_state.app.prompt_key.clone()),
    ]
}

fn usage_vars(usage: Option<&crate::types::Usage>) -> Vec<(String, String)> {
    let Some(usage) = usage else {
        return Vec::new();
    };
    let prompt = usage.prompt_tokens.unwrap_or(0);
    let completion = usage.completion_tokens.unwrap_or(0);
    let total = usage.total_tokens.unwrap_or(prompt + completion);
    vec![
        ("HOOK_USAGE_PROMPT".to_string(), prompt.to_string()),
        ("HOOK_USAGE_COMPLETION".to_string(), completion.to_string()),
        ("HOOK_USAGE_TOTAL".to_string(), total.to_string()),
    ]
}

fn tool_calls_vars(calls: &[crate::types::ToolCall]) -> Vec<(String, String)> {
    let names = calls
        .iter()
        .map(|call| call.function.name.clone())
        .collect::<Vec<_>>()
        .join(",");
    vec![
        ("HOOK_TOOL_CALLS_COUNT".to_string(), calls.len().to_string()),
        ("HOOK_TOOL_CALLS_NAMES".to_string(), names),
    ]
}
