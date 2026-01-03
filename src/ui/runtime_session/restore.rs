use crate::args::Args;
use crate::ui::runtime_helpers::TabState;

use super::helpers::{
    ensure_default_tab, init_categories, load_tabs, resolve_active_category, resolve_active_tab,
};

type RestoreTabsResult =
    Result<(Vec<TabState>, usize, Vec<String>, usize), Box<dyn std::error::Error>>;

pub(crate) fn restore_tabs_from_session(
    session: &crate::session::SessionData,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> RestoreTabsResult {
    let (mut categories, active_category_name) = init_categories(session);
    let mut tabs = load_tabs(session, registry, prompt_registry, args, &mut categories)?;
    ensure_default_tab(
        &mut tabs,
        &active_category_name,
        registry,
        prompt_registry,
        args,
    )?;
    let active_tab = resolve_active_tab(session, &tabs);
    let active_category =
        resolve_active_category(&active_category_name, &categories, &tabs, active_tab);
    Ok((tabs, active_tab, categories, active_category))
}
