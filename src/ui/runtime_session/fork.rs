use crate::args::Args;
use crate::ui::runtime_helpers::TabState;

use super::helpers::{
    insert_system_prompt, last_user_message, resolve_model_key, resolve_prompt_key,
    resolve_system_prompt,
};

pub(crate) fn fork_last_tab_for_retry(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> Option<(usize, String)> {
    let source = tabs.get((*active_tab).min(tabs.len().saturating_sub(1)))?;
    let (msg_idx, content) = last_user_message(source)?;
    let system_prompt = resolve_system_prompt(source, prompt_registry, args);
    let model_key = resolve_model_key(source, registry);
    let prompt_key = resolve_prompt_key(source, prompt_registry);
    let mut history: Vec<crate::types::Message> = source.app.messages[..msg_idx].to_vec();
    let mut new_tab = create_retry_tab(source, &model_key, &prompt_key)?;
    insert_system_prompt(&mut history, &system_prompt);
    apply_retry_history(&mut new_tab, source, history, model_key, prompt_key);
    tabs.push(new_tab);
    *active_tab = tabs.len().saturating_sub(1);
    Some((*active_tab, content))
}

fn create_retry_tab(source: &TabState, model_key: &str, prompt_key: &str) -> Option<TabState> {
    let conv_id = crate::conversation::new_conversation_id().ok()?;
    let mut tab = TabState::new(
        conv_id,
        source.category.clone(),
        "",
        false,
        model_key,
        prompt_key,
    );
    tab.app.set_log_session_id(&source.app.log_session_id);
    tab.app.hooks = source.app.hooks.clone();
    Some(tab)
}

fn apply_retry_history(
    tab: &mut TabState,
    source: &TabState,
    history: Vec<crate::types::Message>,
    model_key: String,
    prompt_key: String,
) {
    tab.app.messages = history;
    tab.app.model_key = model_key;
    tab.app.prompt_key = prompt_key;
    tab.app.prompts_dir = source.app.prompts_dir.clone();
    tab.app.tavily_api_key = source.app.tavily_api_key.clone();
    tab.app.dirty_indices = (0..tab.app.messages.len()).collect();
}
