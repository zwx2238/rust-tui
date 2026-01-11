use crate::args::Args;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;

pub(super) fn create_tab_in_active_category(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    let category = resolve_active_category(categories, active_category);
    let tab = build_new_tab(tabs, *active_tab, &category, registry, prompt_registry, args);
    tabs.push(tab);
    *active_tab = tabs.len().saturating_sub(1);
}

fn resolve_active_category(categories: &mut Vec<String>, active_category: &mut usize) -> String {
    if categories.is_empty() {
        categories.push("默认".to_string());
        *active_category = 0;
    }
    if *active_category >= categories.len() {
        *active_category = 0;
    }
    categories
        .get(*active_category)
        .cloned()
        .unwrap_or_else(|| "默认".to_string())
}

fn build_new_tab(
    tabs: &[TabState],
    active_tab: usize,
    category: &str,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> TabState {
    let conv_id =
        crate::conversation::new_conversation_id().unwrap_or_else(|_| tabs.len().to_string());
    let system = prompt_registry
        .get(&prompt_registry.default_key)
        .map(|p| p.content.as_str())
        .unwrap_or(&args.system);
    let mut tab = TabState::new(
        conv_id,
        category.to_string(),
        system,
        args.perf,
        &registry.default_key,
        &prompt_registry.default_key,
    );
    inherit_tab_settings(tabs, active_tab, &mut tab);
    tab
}

fn inherit_tab_settings(tabs: &[TabState], active_tab: usize, tab: &mut TabState) {
    if let Some(active) = tabs.get(active_tab) {
        tab.app.prompts_dir = active.app.prompts_dir.clone();
        tab.app.tavily_api_key = active.app.tavily_api_key.clone();
        tab.app.default_role = active.app.default_role.clone();
        tab.app.set_log_session_id(&active.app.log_session_id);
    }
}
