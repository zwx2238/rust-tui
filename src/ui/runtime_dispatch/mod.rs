use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::runtime_helpers::{start_tab_request, TabState};
use crate::ui::net::UiEvent;
use ratatui::layout::Rect;
use std::sync::mpsc;

mod key;
mod mouse;

const PROMPT_LOCKED_MSG: &str = "已开始对话，无法切换系统提示词，请新开 tab。";

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

pub(crate) fn sync_model_selection(
    view: &mut crate::ui::runtime_view::ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
) {
    if let Some(tab_state) = ctx.tabs.get(*ctx.active_tab) {
        if let Some(idx) = ctx.registry.index_of(&tab_state.app.model_key) {
            view.model.selected = idx;
        }
    }
    let viewport_rows =
        crate::ui::model_popup::model_visible_rows(layout.size, ctx.registry.models.len());
    view.model
        .clamp_with_viewport(ctx.registry.models.len(), viewport_rows);
}

pub(crate) fn sync_prompt_selection(
    view: &mut crate::ui::runtime_view::ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
) {
    if let Some(tab_state) = ctx.tabs.get(*ctx.active_tab) {
        if let Some(idx) = ctx
            .prompt_registry
            .prompts
            .iter()
            .position(|p| p.key == tab_state.app.prompt_key)
        {
            view.prompt.selected = idx;
        }
    }
    let viewport_rows = crate::ui::prompt_popup::prompt_visible_rows(
        layout.size,
        ctx.prompt_registry.prompts.len(),
    );
    view.prompt
        .clamp_with_viewport(ctx.prompt_registry.prompts.len(), viewport_rows);
}

pub(crate) fn apply_model_selection(
    ctx: &mut DispatchContext<'_>,
    idx: usize,
) {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if let Some(model) = ctx.registry.models.get(idx) {
            tab_state.app.model_key = model.key.clone();
        }
    }
}

pub(crate) fn apply_prompt_selection(
    ctx: &mut DispatchContext<'_>,
    idx: usize,
) {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if can_change_prompt(&tab_state.app) {
            if let Some(prompt) = ctx.prompt_registry.prompts.get(idx) {
                tab_state
                    .app
                    .set_system_prompt(&prompt.key, &prompt.content);
            }
        } else {
            push_prompt_locked(tab_state);
        }
    }
}

pub(crate) fn push_prompt_locked(tab_state: &mut TabState) {
    tab_state.app.messages.push(crate::types::Message {
        role: "assistant".to_string(),
        content: PROMPT_LOCKED_MSG.to_string(),
    });
}

pub(crate) fn new_tab(ctx: &mut DispatchContext<'_>) {
    ctx.tabs.push(TabState::new(
        ctx.prompt_registry
            .get(&ctx.prompt_registry.default_key)
            .map(|p| p.content.as_str())
            .unwrap_or(&ctx.args.system),
        ctx.args.perf,
        &ctx.registry.default_key,
        &ctx.prompt_registry.default_key,
    ));
    *ctx.active_tab = ctx.tabs.len().saturating_sub(1);
}

pub(crate) fn close_tab(ctx: &mut DispatchContext<'_>) {
    if ctx.tabs.len() > 1 {
        ctx.tabs.remove(*ctx.active_tab);
        if *ctx.active_tab >= ctx.tabs.len() {
            *ctx.active_tab = ctx.tabs.len().saturating_sub(1);
        }
    }
}

pub(crate) fn prev_tab(ctx: &mut DispatchContext<'_>) {
    if !ctx.tabs.is_empty() {
        *ctx.active_tab = if *ctx.active_tab == 0 {
            ctx.tabs.len().saturating_sub(1)
        } else {
            *ctx.active_tab - 1
        };
    }
}

pub(crate) fn next_tab(ctx: &mut DispatchContext<'_>) {
    if !ctx.tabs.is_empty() {
        *ctx.active_tab = (*ctx.active_tab + 1) % ctx.tabs.len();
    }
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
