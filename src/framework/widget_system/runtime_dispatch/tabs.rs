use crate::framework::widget_system::runtime_dispatch::DispatchContext;
use crate::framework::widget_system::runtime::runtime_helpers::{TabState, tab_to_conversation, visible_tab_indices};

pub(crate) fn new_tab(ctx: &mut DispatchContext<'_>) {
    let category = active_category_name(ctx);
    if !ctx.categories.contains(&category) {
        ctx.categories.push(category.clone());
    }
    let conv_id =
        crate::conversation::new_conversation_id().unwrap_or_else(|_| ctx.tabs.len().to_string());
    let mut tab = TabState::new(
        conv_id,
        category,
        ctx.prompt_registry
            .get(&ctx.prompt_registry.default_key)
            .map(|p| p.content.as_str())
            .unwrap_or(&ctx.args.system),
        ctx.args.perf,
        &ctx.registry.default_key,
        &ctx.prompt_registry.default_key,
    );
    if let Some(active) = ctx.tabs.get(*ctx.active_tab) {
        tab.app.prompts_dir = active.app.prompts_dir.clone();
        tab.app.tavily_api_key = active.app.tavily_api_key.clone();
        tab.app.default_role = active.app.default_role.clone();
        tab.app.set_log_session_id(&active.app.log_session_id);
    }
    ctx.tabs.push(tab);
    let next = ctx.tabs.len().saturating_sub(1);
    set_active_tab(ctx, next);
    sync_active_category(ctx);
}

pub(crate) fn close_tab(ctx: &mut DispatchContext<'_>) {
    if ctx.tabs.len() > 1 {
        if let Some(tab) = ctx.tabs.get(*ctx.active_tab) {
            let _ = crate::conversation::save_conversation(&tab_to_conversation(tab));
        }
        ctx.tabs.remove(*ctx.active_tab);
        let mut next = *ctx.active_tab;
        if next >= ctx.tabs.len() {
            next = ctx.tabs.len().saturating_sub(1);
        }
        set_active_tab(ctx, next);
        cleanup_categories(ctx);
        ensure_active_tab_in_category(ctx);
    }
}

pub(crate) fn close_other_tabs(ctx: &mut DispatchContext<'_>) {
    if ctx.tabs.is_empty() {
        return;
    }
    if ctx.tabs.len() == 1 {
        return;
    }
    let active = ctx.tabs.remove(*ctx.active_tab);
    for tab in ctx.tabs.iter() {
        let _ = crate::conversation::save_conversation(&tab_to_conversation(tab));
    }
    ctx.tabs.clear();
    ctx.tabs.push(active);
    set_active_tab(ctx, 0);
    ctx.categories.clear();
    let keep_category = ctx
        .tabs
        .first()
        .map(|t| t.category.clone())
        .unwrap_or_else(|| "默认".to_string());
    ctx.categories.push(keep_category);
    *ctx.active_category = 0;
}

pub(crate) fn close_all_tabs(ctx: &mut DispatchContext<'_>) {
    let keep = active_tab_keep_config(ctx);
    save_all_tabs(ctx);
    reset_tabs_and_categories(ctx, &keep.category);
    let mut tab = build_default_tab(ctx);
    apply_keep_config(&mut tab, keep);
    ctx.tabs.push(tab);
    set_active_tab(ctx, 0);
}

struct KeepConfig {
    prompts_dir: String,
    tavily_api_key: String,
    log_session_id: String,
    category: String,
    default_role: String,
}

fn active_tab_keep_config(ctx: &DispatchContext<'_>) -> KeepConfig {
    let fallback = KeepConfig {
        prompts_dir: String::new(),
        tavily_api_key: String::new(),
        log_session_id: String::new(),
        category: "默认".to_string(),
        default_role: crate::types::ROLE_USER.to_string(),
    };
    ctx.tabs
        .get(*ctx.active_tab)
        .map_or(fallback, |tab| KeepConfig {
            prompts_dir: tab.app.prompts_dir.clone(),
            tavily_api_key: tab.app.tavily_api_key.clone(),
            log_session_id: tab.app.log_session_id.clone(),
            category: tab.category.clone(),
            default_role: tab.app.default_role.clone(),
        })
}

fn save_all_tabs(ctx: &DispatchContext<'_>) {
    for tab in &*ctx.tabs {
        let _ = crate::conversation::save_conversation(&tab_to_conversation(tab));
    }
}

fn reset_tabs_and_categories(ctx: &mut DispatchContext<'_>, category: &str) {
    ctx.tabs.clear();
    ctx.categories.clear();
    ctx.categories.push(category.to_string());
    *ctx.active_category = 0;
}

fn build_default_tab(ctx: &mut DispatchContext<'_>) -> TabState {
    let conv_id = crate::conversation::new_conversation_id().unwrap_or_else(|_| "new".to_string());
    TabState::new(
        conv_id,
        active_category_name(ctx),
        ctx.prompt_registry
            .get(&ctx.prompt_registry.default_key)
            .map(|p| p.content.as_str())
            .unwrap_or(&ctx.args.system),
        ctx.args.perf,
        &ctx.registry.default_key,
        &ctx.prompt_registry.default_key,
    )
}

fn apply_keep_config(tab: &mut TabState, keep: KeepConfig) {
    tab.app.prompts_dir = keep.prompts_dir;
    tab.app.tavily_api_key = keep.tavily_api_key;
    tab.app.default_role = keep.default_role;
    if !keep.log_session_id.is_empty() {
        tab.app.set_log_session_id(&keep.log_session_id);
    }
}

pub(crate) fn prev_tab(ctx: &mut DispatchContext<'_>) {
    let category = active_category_name(ctx);
    let visible = visible_tab_indices(ctx.tabs, &category);
    if visible.is_empty() {
        return;
    }
    let pos = visible
        .iter()
        .position(|idx| *idx == *ctx.active_tab)
        .unwrap_or(0);
    let next_pos = if pos == 0 { visible.len() - 1 } else { pos - 1 };
    set_active_tab(ctx, visible[next_pos]);
}

pub(crate) fn next_tab(ctx: &mut DispatchContext<'_>) {
    let category = active_category_name(ctx);
    let visible = visible_tab_indices(ctx.tabs, &category);
    if visible.is_empty() {
        return;
    }
    let pos = visible
        .iter()
        .position(|idx| *idx == *ctx.active_tab)
        .unwrap_or(0);
    let next_pos = (pos + 1) % visible.len();
    set_active_tab(ctx, visible[next_pos]);
}

pub(crate) fn prev_category(ctx: &mut DispatchContext<'_>) {
    if ctx.categories.is_empty() {
        ctx.categories.push("默认".to_string());
    }
    if *ctx.active_category == 0 {
        *ctx.active_category = ctx.categories.len().saturating_sub(1);
    } else {
        *ctx.active_category -= 1;
    }
    ensure_active_tab_in_category(ctx);
}

pub(crate) fn next_category(ctx: &mut DispatchContext<'_>) {
    if ctx.categories.is_empty() {
        ctx.categories.push("默认".to_string());
    }
    *ctx.active_category = (*ctx.active_category + 1) % ctx.categories.len();
    ensure_active_tab_in_category(ctx);
}

fn active_category_name(ctx: &mut DispatchContext<'_>) -> String {
    if ctx.categories.is_empty() {
        ctx.categories.push("默认".to_string());
    }
    ctx.categories
        .get(*ctx.active_category)
        .cloned()
        .unwrap_or_else(|| "默认".to_string())
}

fn ensure_active_tab_in_category(ctx: &mut DispatchContext<'_>) {
    if ctx.tabs.is_empty() {
        return;
    }
    let category = active_category_name(ctx);
    if let Some(idx) = ctx.tabs.iter().position(|t| t.category == category) {
        set_active_tab(ctx, idx);
    } else {
        set_active_tab(ctx, 0);
        sync_active_category(ctx);
    }
}

fn sync_active_category(ctx: &mut DispatchContext<'_>) {
    if let Some(tab) = ctx.tabs.get(*ctx.active_tab) {
        if let Some(idx) = ctx.categories.iter().position(|c| c == &tab.category) {
            *ctx.active_category = idx;
        } else {
            ctx.categories.push(tab.category.clone());
            *ctx.active_category = ctx.categories.len().saturating_sub(1);
        }
    }
}

fn cleanup_categories(ctx: &mut DispatchContext<'_>) {
    if ctx.categories.is_empty() {
        ctx.categories.push("默认".to_string());
        *ctx.active_category = 0;
        return;
    }
    ctx.categories
        .retain(|cat| ctx.tabs.iter().any(|t| &t.category == cat));
    if ctx.categories.is_empty() {
        ctx.categories.push("默认".to_string());
    }
    if *ctx.active_category >= ctx.categories.len() {
        *ctx.active_category = 0;
    }
}

fn set_active_tab(ctx: &mut DispatchContext<'_>, idx: usize) {
    if *ctx.active_tab == idx {
        return;
    }
    *ctx.active_tab = idx;
    if let Some(tab) = ctx.tabs.get_mut(idx) {
        tab.app.follow = false;
    }
}
