use crate::ui::runtime_dispatch::DispatchContext;
use crate::ui::runtime_helpers::TabState;

pub(crate) fn new_tab(ctx: &mut DispatchContext<'_>) {
    let mut tab = TabState::new(
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
    }
    ctx.tabs.push(tab);
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

pub(crate) fn close_other_tabs(ctx: &mut DispatchContext<'_>) {
    if ctx.tabs.is_empty() {
        return;
    }
    if ctx.tabs.len() == 1 {
        return;
    }
    let active = ctx.tabs.remove(*ctx.active_tab);
    ctx.tabs.clear();
    ctx.tabs.push(active);
    *ctx.active_tab = 0;
}

pub(crate) fn close_all_tabs(ctx: &mut DispatchContext<'_>) {
    let (prompts_dir, tavily_api_key) = ctx
        .tabs
        .get(*ctx.active_tab)
        .map(|tab| (tab.app.prompts_dir.clone(), tab.app.tavily_api_key.clone()))
        .unwrap_or_else(|| (String::new(), String::new()));
    ctx.tabs.clear();
    let mut tab = TabState::new(
        ctx.prompt_registry
            .get(&ctx.prompt_registry.default_key)
            .map(|p| p.content.as_str())
            .unwrap_or(&ctx.args.system),
        ctx.args.perf,
        &ctx.registry.default_key,
        &ctx.prompt_registry.default_key,
    );
    tab.app.prompts_dir = prompts_dir;
    tab.app.tavily_api_key = tavily_api_key;
    ctx.tabs.push(tab);
    *ctx.active_tab = 0;
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
