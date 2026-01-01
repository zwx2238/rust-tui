use crate::args::Args;
use crate::render::RenderTheme;
use crate::types::{ROLE_ASSISTANT, ROLE_SYSTEM, ROLE_USER};
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{TabState, start_tab_request};
use crate::ui::state::Focus;
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
    tab_state.app.messages.push(crate::types::Message {
        role: ROLE_ASSISTANT.to_string(),
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

pub(crate) fn fork_message_into_new_tab(
    ctx: &mut DispatchContext<'_>,
    jump_rows: &[crate::ui::jump::JumpRow],
    row_idx: usize,
) {
    let Some(tab_state) = ctx.tabs.get(*ctx.active_tab) else {
        return;
    };
    let Some(row) = jump_rows.get(row_idx) else {
        return;
    };
    let msg_idx = row.index.saturating_sub(1);
    let Some(msg) = tab_state.app.messages.get(msg_idx) else {
        return;
    };
    let content = msg.content.clone();
    let system_prompt = tab_state
        .app
        .messages
        .iter()
        .find(|m| m.role == ROLE_SYSTEM)
        .map(|m| m.content.clone())
        .or_else(|| {
            ctx.prompt_registry
                .get(&tab_state.app.prompt_key)
                .map(|p| p.content.clone())
        })
        .unwrap_or_else(|| ctx.args.system.clone());
    let model_key = if ctx.registry.get(&tab_state.app.model_key).is_some() {
        tab_state.app.model_key.clone()
    } else {
        ctx.registry.default_key.clone()
    };
    let prompt_key = if ctx
        .prompt_registry
        .get(&tab_state.app.prompt_key)
        .is_some()
    {
        tab_state.app.prompt_key.clone()
    } else {
        ctx.prompt_registry.default_key.clone()
    };
    let mut new_tab = TabState::new(&system_prompt, false, &model_key, &prompt_key);
    new_tab.app.model_key = model_key;
    new_tab.app.prompt_key = prompt_key;
    if !content.is_empty() {
        new_tab.app.input.insert_str(content);
    }
    new_tab.app.focus = Focus::Input;
    ctx.tabs.push(new_tab);
    *ctx.active_tab = ctx.tabs.len().saturating_sub(1);
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
