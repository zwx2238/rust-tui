use crate::args::Args;
use crate::render::RenderTheme;
use crate::types::{ROLE_SYSTEM, ROLE_USER};
use crate::ui::net::UiEvent;
use crate::ui::overlay::OverlayKind;
use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, overlay_visible_rows};
use crate::ui::notice::push_notice;
use crate::ui::runtime_helpers::{TabState, start_tab_request};
use crate::ui::state::Focus;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    }
}

pub(crate) fn handle_nav_key(app: &mut crate::ui::state::App, key: KeyEvent) -> bool {
    if !app.nav_mode {
        if app.focus == Focus::Chat
            && key.modifiers == KeyModifiers::NONE
            && key.code == KeyCode::Char('g')
        {
            app.nav_mode = true;
            app.focus = Focus::Chat;
            app.follow = false;
            return true;
        }
        return false;
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('g') => {
            app.nav_mode = false;
            true
        }
        KeyCode::Char('j') | KeyCode::Char('n') => {
            nav_next(app);
            true
        }
        KeyCode::Char('k') | KeyCode::Char('p') => {
            nav_prev(app);
            true
        }
        _ => true,
    }
}

fn nav_next(app: &mut crate::ui::state::App) {
    if app.message_layouts.is_empty() {
        return;
    }
    let current = app.scroll as usize;
    let mut target = None;
    for layout in &app.message_layouts {
        if layout.label_line > current {
            target = Some(layout.label_line);
            break;
        }
    }
    if let Some(line) = target {
        app.scroll = line.min(u16::MAX as usize) as u16;
        app.follow = false;
    }
}

fn nav_prev(app: &mut crate::ui::state::App) {
    if app.message_layouts.is_empty() {
        return;
    }
    let current = app.scroll as usize;
    let mut target = None;
    for layout in app.message_layouts.iter().rev() {
        if layout.label_line < current {
            target = Some(layout.label_line);
            break;
        }
    }
    if let Some(line) = target {
        app.scroll = line.min(u16::MAX as usize) as u16;
        app.follow = false;
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
) -> bool {
    let Some(row) = jump_rows.get(row_idx) else {
        return false;
    };
    let msg_idx = row.index.saturating_sub(1);
    fork_message_by_index(ctx, msg_idx)
}

pub(crate) fn fork_message_by_index(
    ctx: &mut DispatchContext<'_>,
    msg_idx: usize,
) -> bool {
    let Some(tab_state) = ctx.tabs.get(*ctx.active_tab) else {
        return false;
    };
    let Some(msg) = tab_state.app.messages.get(msg_idx) else {
        return false;
    };
    if msg.role != ROLE_USER {
        if let Some(active) = ctx.tabs.get_mut(*ctx.active_tab) {
            crate::ui::notice::push_notice(&mut active.app, "仅支持从用户消息分叉。");
        }
        return false;
    }
    let content = msg.content.clone();
    let mut history: Vec<crate::types::Message> =
        tab_state.app.messages[..msg_idx].to_vec();
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
    let mut new_tab = TabState::new("", false, &model_key, &prompt_key);
    if history.iter().all(|m| m.role != ROLE_SYSTEM) && !system_prompt.trim().is_empty() {
        history.insert(
            0,
            crate::types::Message {
                role: ROLE_SYSTEM.to_string(),
                content: system_prompt.clone(),
            },
        );
    }
    new_tab.app.messages = history;
    new_tab.app.model_key = model_key;
    new_tab.app.prompt_key = prompt_key;
    new_tab.app.dirty_indices = (0..new_tab.app.messages.len()).collect();
    if !content.is_empty() {
        new_tab.app.input.insert_str(content);
    }
    new_tab.app.focus = Focus::Input;
    ctx.tabs.push(new_tab);
    *ctx.active_tab = ctx.tabs.len().saturating_sub(1);
    true
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
