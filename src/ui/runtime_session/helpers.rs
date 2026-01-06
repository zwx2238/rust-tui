use crate::args::Args;
use crate::types::{ROLE_SYSTEM, ROLE_USER};
use crate::ui::runtime_helpers::TabState;

pub(crate) fn init_categories(session: &crate::session::SessionData) -> (Vec<String>, String) {
    let mut categories = session.categories.clone();
    if categories.is_empty() {
        categories.push("默认".to_string());
    }
    let name = if session.active_category.trim().is_empty() {
        categories[0].clone()
    } else {
        session.active_category.clone()
    };
    if !categories.contains(&name) {
        categories.push(name.clone());
    }
    (categories, name)
}

pub(crate) fn load_tabs(
    session: &crate::session::SessionData,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    categories: &mut Vec<String>,
) -> Result<Vec<TabState>, Box<dyn std::error::Error>> {
    let mut tabs = Vec::new();
    for conv_id in &session.open_conversations {
        let conv = crate::conversation::load_conversation(conv_id)
            .map_err(|e| format!("无法读取对话 {conv_id}: {e}"))?;
        let category = normalize_category(&conv.category, categories);
        let model_key = resolve_conv_model(&conv, registry);
        let prompt_key = resolve_conv_prompt(&conv, prompt_registry);
        let mut state = TabState::new(
            conv.id.clone(),
            category,
            "",
            false,
            &model_key,
            &prompt_key,
        );
        apply_conversation_state(&mut state, &conv, &prompt_key, prompt_registry, args);
        tabs.push(state);
    }
    if categories.is_empty() {
        categories.push("默认".to_string());
    }
    Ok(tabs)
}

pub(crate) fn ensure_default_tab(
    tabs: &mut Vec<TabState>,
    active_category_name: &str,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    if !tabs.is_empty() {
        return Ok(());
    }
    let conv_id = crate::conversation::new_conversation_id()?;
    let prompt = prompt_registry
        .get(&prompt_registry.default_key)
        .map(|p| p.content.as_str())
        .unwrap_or(&args.system);
    tabs.push(TabState::new(
        conv_id,
        active_category_name.to_string(),
        prompt,
        false,
        &registry.default_key,
        &prompt_registry.default_key,
    ));
    Ok(())
}

pub(crate) fn resolve_active_tab(
    session: &crate::session::SessionData,
    tabs: &[TabState],
) -> usize {
    session
        .active_conversation
        .as_deref()
        .and_then(|id| tabs.iter().position(|t| t.conversation_id == id))
        .unwrap_or(0)
        .min(tabs.len().saturating_sub(1))
}

pub(crate) fn resolve_active_category(
    active_category_name: &str,
    categories: &[String],
    tabs: &[TabState],
    active_tab: usize,
) -> usize {
    categories
        .iter()
        .position(|c| c == active_category_name)
        .or_else(|| {
            tabs.get(active_tab)
                .and_then(|t| categories.iter().position(|c| c == &t.category))
        })
        .unwrap_or(0)
}

pub(crate) fn last_user_message(source: &TabState) -> Option<(usize, String)> {
    let mut last_user_idx = None;
    for (idx, msg) in source.app.messages.iter().enumerate().rev() {
        if msg.role == ROLE_USER {
            last_user_idx = Some(idx);
            break;
        }
    }
    let msg_idx = last_user_idx?;
    let msg = source.app.messages.get(msg_idx)?;
    let content = msg.content.clone();
    if content.trim().is_empty() {
        return None;
    }
    Some((msg_idx, content))
}

pub(crate) fn resolve_system_prompt(
    source: &TabState,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> String {
    source
        .app
        .messages
        .iter()
        .find(|m| m.role == ROLE_SYSTEM)
        .map(|m| m.content.clone())
        .or_else(|| {
            prompt_registry
                .get(&source.app.prompt_key)
                .map(|p| p.content.clone())
        })
        .unwrap_or_else(|| args.system.clone())
}

pub(crate) fn resolve_model_key(
    source: &TabState,
    registry: &crate::model_registry::ModelRegistry,
) -> String {
    if registry.get(&source.app.model_key).is_some() {
        source.app.model_key.clone()
    } else {
        registry.default_key.clone()
    }
}

pub(crate) fn resolve_prompt_key(
    source: &TabState,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
) -> String {
    if prompt_registry.get(&source.app.prompt_key).is_some() {
        source.app.prompt_key.clone()
    } else {
        prompt_registry.default_key.clone()
    }
}

pub(crate) fn insert_system_prompt(history: &mut Vec<crate::types::Message>, system_prompt: &str) {
    if history.iter().any(|m| m.role == ROLE_SYSTEM) || system_prompt.trim().is_empty() {
        return;
    }
    history.insert(
        0,
        crate::types::Message {
            role: ROLE_SYSTEM.to_string(),
            content: system_prompt.to_string(),
            tool_call_id: None,
            tool_calls: None,
        },
    );
}

fn normalize_category(category: &str, categories: &mut Vec<String>) -> String {
    let value = if category.trim().is_empty() {
        "默认".to_string()
    } else {
        category.to_string()
    };
    if !categories.contains(&value) {
        categories.push(value.clone());
    }
    value
}

fn resolve_conv_model(
    conv: &crate::conversation::ConversationData,
    registry: &crate::model_registry::ModelRegistry,
) -> String {
    conv.model_key
        .as_deref()
        .filter(|k| registry.get(k).is_some())
        .unwrap_or(&registry.default_key)
        .to_string()
}

fn resolve_conv_prompt(
    conv: &crate::conversation::ConversationData,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
) -> String {
    conv.prompt_key
        .as_deref()
        .filter(|k| prompt_registry.get(k).is_some())
        .unwrap_or(&prompt_registry.default_key)
        .to_string()
}

fn apply_conversation_state(
    state: &mut TabState,
    conv: &crate::conversation::ConversationData,
    prompt_key: &str,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    state.app.messages = conv.messages.clone();
    state.app.code_exec_container_id = conv.code_exec_container_id.clone();
    ensure_system_prompt(state, prompt_key, prompt_registry, args);
    state.app.follow = true;
    state.app.scroll = u16::MAX;
    state.app.dirty_indices = (0..state.app.messages.len()).collect();
}

fn ensure_system_prompt(
    state: &mut TabState,
    prompt_key: &str,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    if state.app.messages.iter().any(|m| m.role == ROLE_SYSTEM) {
        return;
    }
    let content = prompt_registry
        .get(prompt_key)
        .map(|p| p.content.as_str())
        .unwrap_or(&args.system);
    if !content.trim().is_empty() {
        state.app.set_system_prompt(prompt_key, content);
    }
}
