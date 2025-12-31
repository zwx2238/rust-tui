use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::runtime_helpers::{start_tab_request, TabState};
use crate::ui::net::UiEvent;
use ratatui::layout::Rect;
use std::sync::mpsc;

mod key;
mod mouse;

pub(crate) use key::handle_key_event_loop;
pub(crate) use mouse::handle_mouse_event_loop;

pub(crate) struct DispatchContext<'a> {
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: &'a mut usize,
    pub(crate) last_session_id: &'a mut Option<String>,
    pub(crate) msg_width: usize,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::system_prompts::PromptRegistry,
    pub(crate) args: &'a Args,
}

#[derive(Copy, Clone)]
pub(crate) struct LayoutContext {
    pub(crate) size: Rect,
    pub(crate) tabs_area: Rect,
    pub(crate) msg_area: Rect,
    pub(crate) input_area: Rect,
    pub(crate) view_height: u16,
    pub(crate) total_lines: usize,
}

pub(crate) fn start_pending_request(
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
    active_tab: usize,
    tab_state: &mut TabState,
) {
    let model = resolve_model(registry, &tab_state.app.model_key);
    start_tab_request(
        tab_state,
        "",
        &model.base_url,
        &model.api_key,
        &model.model,
        args.show_reasoning,
        tx,
        active_tab,
    );
}

pub(crate) fn can_change_prompt(app: &crate::ui::state::App) -> bool {
    !app.messages.iter().any(|m| m.role == "user")
}

pub(crate) fn cycle_model(registry: &crate::model_registry::ModelRegistry, key: &mut String) {
    if registry.models.is_empty() {
        return;
    }
    let idx = registry.index_of(key).unwrap_or(0);
    let next = (idx + 1) % registry.models.len();
    *key = registry.models[next].key.clone();
}

pub(crate) fn resolve_model<'a>(
    registry: &'a crate::model_registry::ModelRegistry,
    key: &str,
) -> &'a crate::model_registry::ModelProfile {
    registry
        .get(key)
        .or_else(|| registry.get(&registry.default_key))
        .expect("model registry is empty")
}
