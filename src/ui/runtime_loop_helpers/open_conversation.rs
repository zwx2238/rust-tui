use crate::args::Args;
use crate::types::Message;
use crate::ui::runtime_helpers::TabState;

pub(super) fn open_conversation_in_tab(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    let Some(conv_id) = take_pending_conversation(tabs, *active_tab) else {
        return;
    };
    if switch_to_existing_tab(tabs, &conv_id, active_tab, active_category, categories) {
        return;
    }
    let conv = match load_conversation_or_report(&conv_id, tabs, *active_tab) {
        Some(conv) => conv,
        None => return,
    };
    let mut tab = build_tab_from_conversation(&conv, registry, prompt_registry, args);
    inherit_tab_settings(tabs, *active_tab, &mut tab);
    finalize_opened_tab(tabs, categories, active_tab, active_category, tab);
}

fn take_pending_conversation(tabs: &mut [TabState], active_tab: usize) -> Option<String> {
    let active = tabs.get_mut(active_tab)?;
    active.app.pending_open_conversation.take()
}

fn switch_to_existing_tab(
    tabs: &[TabState],
    conv_id: &str,
    active_tab: &mut usize,
    active_category: &mut usize,
    categories: &[String],
) -> bool {
    let Some(idx) = tabs.iter().position(|t| t.conversation_id == conv_id) else {
        return false;
    };
    *active_tab = idx;
    if let Some(tab) = tabs.get(*active_tab)
        && let Some(pos) = categories.iter().position(|c| c == &tab.category)
    {
        *active_category = pos;
    }
    true
}

fn load_conversation_or_report(
    conv_id: &str,
    tabs: &mut [TabState],
    active_tab: usize,
) -> Option<crate::conversation::ConversationData> {
    match crate::conversation::load_conversation(conv_id) {
        Ok(c) => Some(c),
        Err(e) => {
            if let Some(active) = tabs.get_mut(active_tab) {
                push_assistant_message(active, format!("打开对话失败：{e}"));
            }
            None
        }
    }
}

fn push_assistant_message(tab_state: &mut TabState, content: String) {
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_ASSISTANT.to_string(),
        content,
        tool_call_id: None,
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
}

fn build_tab_from_conversation(
    conv: &crate::conversation::ConversationData,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> TabState {
    let category = conv_category(conv);
    let model_key = conv_model_key(conv, registry);
    let prompt_key = conv_prompt_key(conv, prompt_registry);
    let mut tab = init_tab_from_conversation(conv, &category, &model_key, &prompt_key);
    apply_conversation_state(
        &mut tab,
        conv,
        &model_key,
        &prompt_key,
        prompt_registry,
        args,
    );
    tab
}

fn conv_category(conv: &crate::conversation::ConversationData) -> String {
    if conv.category.trim().is_empty() {
        "默认".to_string()
    } else {
        conv.category.clone()
    }
}

fn conv_model_key(
    conv: &crate::conversation::ConversationData,
    registry: &crate::model_registry::ModelRegistry,
) -> String {
    conv.model_key
        .as_deref()
        .filter(|k| registry.get(k).is_some())
        .unwrap_or(&registry.default_key)
        .to_string()
}

fn conv_prompt_key(
    conv: &crate::conversation::ConversationData,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
) -> String {
    conv.prompt_key
        .as_deref()
        .filter(|k| prompt_registry.get(k).is_some())
        .unwrap_or(&prompt_registry.default_key)
        .to_string()
}

fn init_tab_from_conversation(
    conv: &crate::conversation::ConversationData,
    category: &str,
    model_key: &str,
    prompt_key: &str,
) -> TabState {
    TabState::new(
        conv.id.clone(),
        category.to_string(),
        "",
        false,
        model_key,
        prompt_key,
    )
}

fn apply_conversation_state(
    tab: &mut TabState,
    conv: &crate::conversation::ConversationData,
    model_key: &str,
    prompt_key: &str,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    tab.app.messages = conv.messages.clone();
    tab.app.code_exec_container_id = conv.code_exec_container_id.clone();
    ensure_system_prompt(tab, prompt_key, prompt_registry, args);
    tab.app.model_key = model_key.to_string();
    tab.app.prompt_key = prompt_key.to_string();
    tab.app.follow = true;
    tab.app.scroll = u16::MAX;
    tab.app.dirty_indices = (0..tab.app.messages.len()).collect();
}

fn ensure_system_prompt(
    tab: &mut TabState,
    prompt_key: &str,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    if tab
        .app
        .messages
        .iter()
        .any(|m| m.role == crate::types::ROLE_SYSTEM)
    {
        return;
    }
    let content = prompt_registry
        .get(prompt_key)
        .map(|p| p.content.as_str())
        .unwrap_or(&args.system);
    if !content.trim().is_empty() {
        tab.app.set_system_prompt(prompt_key, content);
    }
}

fn inherit_tab_settings(tabs: &[TabState], active_tab: usize, tab: &mut TabState) {
    if let Some(active) = tabs.get(active_tab) {
        tab.app.prompts_dir = active.app.prompts_dir.clone();
        tab.app.tavily_api_key = active.app.tavily_api_key.clone();
        tab.app.set_log_session_id(&active.app.log_session_id);
    }
}

fn finalize_opened_tab(
    tabs: &mut Vec<TabState>,
    categories: &mut Vec<String>,
    active_tab: &mut usize,
    active_category: &mut usize,
    tab: TabState,
) {
    tabs.push(tab);
    if let Some(tab) = tabs.last() {
        if !categories.iter().any(|c| c == &tab.category) {
            categories.push(tab.category.clone());
        }
        if let Some(pos) = categories.iter().position(|c| c == &tab.category) {
            *active_category = pos;
        }
    }
    *active_tab = tabs.len().saturating_sub(1);
}
