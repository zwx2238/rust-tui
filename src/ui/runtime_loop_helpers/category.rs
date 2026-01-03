use crate::args::Args;
use crate::ui::runtime_helpers::TabState;

pub(super) fn create_category_and_tab(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    let Some(active) = tabs.get_mut(*active_tab) else {
        return;
    };
    let name = take_category_name(active, categories);
    let mut tab = build_new_tab_for_category(active, &name, registry, prompt_registry, args);
    tab.app.dirty_indices = (0..tab.app.messages.len()).collect();
    tabs.push(tab);
    *active_category = categories.iter().position(|c| c == &name).unwrap_or(0);
    *active_tab = tabs.len().saturating_sub(1);
}

fn take_category_name(active: &mut TabState, categories: &mut Vec<String>) -> String {
    let name = active
        .app
        .pending_category_name
        .take()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| next_category_name(categories));
    if !categories.contains(&name) {
        categories.push(name.clone());
    }
    name
}

fn build_new_tab_for_category(
    active: &TabState,
    name: &str,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> TabState {
    let conv_id = crate::conversation::new_conversation_id()
        .unwrap_or_else(|_| active.app.log_session_id.clone());
    let model_key = select_model_key(active, registry);
    let prompt_key = select_prompt_key(active, prompt_registry);
    let system = prompt_registry
        .get(&prompt_key)
        .map(|p| p.content.as_str())
        .unwrap_or(&args.system);
    let mut tab = TabState::new(
        conv_id,
        name.to_string(),
        system,
        false,
        &model_key,
        &prompt_key,
    );
    tab.app.prompts_dir = active.app.prompts_dir.clone();
    tab.app.tavily_api_key = active.app.tavily_api_key.clone();
    tab.app.set_log_session_id(&active.app.log_session_id);
    tab.app.model_key = model_key;
    tab.app.prompt_key = prompt_key;
    tab
}

fn select_model_key(
    active: &TabState,
    registry: &crate::model_registry::ModelRegistry,
) -> String {
    if registry.get(&active.app.model_key).is_some() {
        return active.app.model_key.clone();
    }
    registry.default_key.clone()
}

fn select_prompt_key(
    active: &TabState,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
) -> String {
    if prompt_registry.get(&active.app.prompt_key).is_some() {
        return active.app.prompt_key.clone();
    }
    prompt_registry.default_key.clone()
}

fn next_category_name(existing: &[String]) -> String {
    let mut idx = existing.len().max(1);
    loop {
        let name = format!("分类 {idx}");
        if !existing.iter().any(|c| c == &name) {
            return name;
        }
        idx += 1;
    }
}
