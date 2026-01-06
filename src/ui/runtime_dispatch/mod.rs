use crate::args::Args;
use crate::render::RenderTheme;
use crate::types::ROLE_USER;
use crate::ui::events::RuntimeEvent;
use crate::ui::notice::push_notice;
use crate::ui::overlay::OverlayKind;
use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, overlay_visible_rows};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_tab_request;
use ratatui::layout::Rect;
use std::sync::mpsc;

pub(crate) mod fork;
pub(crate) mod key_helpers;
pub(crate) mod nav;
pub(crate) mod tabs;

const PROMPT_LOCKED_MSG: &str = "已开始对话，无法切换系统提示词，请新建对话。";

pub(crate) struct DispatchContext<'a> {
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: &'a mut usize,
    pub(crate) categories: &'a mut Vec<String>,
    pub(crate) active_category: &'a mut usize,
    pub(crate) msg_width: usize,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) args: &'a Args,
}

#[derive(Copy, Clone)]
pub(crate) struct LayoutContext {
    pub(crate) size: Rect,
    pub(crate) tabs_area: Rect,
    pub(crate) msg_area: Rect,
    pub(crate) input_area: Rect,
    pub(crate) category_area: Rect,
    pub(crate) view_height: u16,
}

pub(crate) fn start_pending_request(
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
    active_tab: usize,
    tab_state: &mut TabState,
) {
    let model = resolve_model(registry, &tab_state.app.model_key);
    let log_session_id = tab_state.app.log_session_id.clone();
    start_tab_request(crate::ui::runtime_requests::StartTabRequestParams {
        tab_state,
        question: "",
        base_url: &model.base_url,
        api_key: &model.api_key,
        model: &model.model,
        _show_reasoning: args.show_reasoning,
        tx,
        tab_id: active_tab,
        enable_web_search: args.web_search_enabled(),
        enable_code_exec: args.code_exec_enabled(),
        enable_read_file: args.read_file_enabled(),
        enable_read_code: args.read_code_enabled(),
        enable_modify_file: args.modify_file_enabled(),
        log_requests: args.log_requests.clone(),
        log_session_id,
    });
}

pub(crate) fn can_change_prompt(app: &crate::ui::state::App) -> bool {
    !app.messages.iter().any(|m| m.role == ROLE_USER)
}

fn with_active_tab<F: FnOnce(&mut TabState)>(ctx: &mut DispatchContext<'_>, f: F) {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        f(tab_state);
    }
}

pub(crate) fn sync_model_selection(
    view: &mut crate::ui::runtime_view::ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
) {
    if let Some(tab_state) = ctx.tabs.get(*ctx.active_tab)
        && let Some(idx) = ctx.registry.index_of(&tab_state.app.model_key)
    {
        view.model.selected = idx;
    }
    let areas = overlay_areas(layout);
    let counts = overlay_counts(ctx);
    let viewport_rows = overlay_visible_rows(OverlayKind::Model, areas, counts);
    view.model
        .clamp_with_viewport(ctx.registry.models.len(), viewport_rows);
}

pub(crate) fn sync_prompt_selection(
    view: &mut crate::ui::runtime_view::ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
) {
    if let Some(tab_state) = ctx.tabs.get(*ctx.active_tab)
        && let Some(idx) = ctx
            .prompt_registry
            .prompts
            .iter()
            .position(|p| p.key == tab_state.app.prompt_key)
    {
        view.prompt.selected = idx;
    }
    let areas = overlay_areas(layout);
    let counts = overlay_counts(ctx);
    let viewport_rows = overlay_visible_rows(OverlayKind::Prompt, areas, counts);
    view.prompt
        .clamp_with_viewport(ctx.prompt_registry.prompts.len(), viewport_rows);
}

fn overlay_areas(layout: LayoutContext) -> OverlayAreas {
    OverlayAreas {
        full: layout.size,
        msg: layout.msg_area,
    }
}

fn overlay_counts(ctx: &DispatchContext<'_>) -> OverlayRowCounts {
    OverlayRowCounts {
        tabs: ctx.tabs.len(),
        jump: 0,
        models: ctx.registry.models.len(),
        prompts: ctx.prompt_registry.prompts.len(),
        help: crate::ui::shortcut_help::help_rows_len(),
    }
}

pub(crate) fn apply_model_selection(ctx: &mut DispatchContext<'_>, idx: usize) {
    with_active_tab(ctx, |tab_state| {
        if let Some(model) = ctx.registry.models.get(idx) {
            tab_state.app.model_key = model.key.clone();
        }
    });
}

pub(crate) fn apply_prompt_selection(ctx: &mut DispatchContext<'_>, idx: usize) {
    with_active_tab(ctx, |tab_state| {
        if can_change_prompt(&tab_state.app) {
            if let Some(prompt) = ctx.prompt_registry.prompts.get(idx) {
                tab_state
                    .app
                    .set_system_prompt(&prompt.key, &prompt.content);
            }
        } else {
            push_prompt_locked(tab_state);
        }
    });
}

pub(crate) fn push_prompt_locked(tab_state: &mut TabState) {
    push_notice(&mut tab_state.app, PROMPT_LOCKED_MSG);
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
