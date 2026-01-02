use crate::args::Args;
use crate::session::SessionLocation;
use crate::types::Message;
use crate::ui::net::UiEvent;
use crate::ui::runtime_code_exec::{
    handle_code_exec_approve, handle_code_exec_deny, handle_code_exec_exit, handle_code_exec_stop,
};
use crate::ui::runtime_file_patch::{handle_file_patch_apply, handle_file_patch_cancel};
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::PendingCommand;

pub(crate) fn handle_pending_command(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    pending: PendingCommand,
    session_location: &mut Option<SessionLocation>,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<UiEvent>,
) {
    match pending {
        PendingCommand::SaveSession => {
            for tab in &*tabs {
                let _ = crate::conversation::save_conversation(
                    &crate::ui::runtime_helpers::tab_to_conversation(tab),
                );
            }
            let open_conversations = crate::ui::runtime_helpers::collect_open_conversations(tabs);
            let active_conv = tabs.get(*active_tab).map(|t| t.conversation_id.clone());
            let save_result = crate::session::save_session(
                categories,
                &open_conversations,
                active_conv.as_deref(),
                categories.get(*active_category).map(|s| s.as_str()),
                session_location.as_ref(),
            );
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                match save_result {
                    Ok(loc) => {
                        *session_location = Some(loc.clone());
                        let hint = loc.display_hint();
                        let idx = tab_state.app.messages.len();
                        tab_state.app.messages.push(Message {
                            role: crate::types::ROLE_ASSISTANT.to_string(),
                            content: format!("已保存会话：{hint}"),
                            tool_call_id: None,
                            tool_calls: None,
                        });
                        tab_state.app.dirty_indices.push(idx);
                    }
                    Err(e) => {
                        let idx = tab_state.app.messages.len();
                        tab_state.app.messages.push(Message {
                            role: crate::types::ROLE_ASSISTANT.to_string(),
                            content: format!("保存失败：{e}"),
                            tool_call_id: None,
                            tool_calls: None,
                        });
                        tab_state.app.dirty_indices.push(idx);
                    }
                }
            }
        }
        PendingCommand::ApproveCodeExec => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                handle_code_exec_approve(tab_state, *active_tab, registry, args, tx);
            }
        }
        PendingCommand::DenyCodeExec => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                handle_code_exec_deny(tab_state, *active_tab, registry, args, tx);
            }
        }
        PendingCommand::ExitCodeExec => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                handle_code_exec_exit(tab_state, *active_tab, registry, args, tx);
            }
        }
        PendingCommand::StopCodeExec => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                handle_code_exec_stop(tab_state);
            }
        }
        PendingCommand::ApplyFilePatch => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                handle_file_patch_apply(tab_state, *active_tab, registry, args, tx);
            }
        }
        PendingCommand::CancelFilePatch => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                handle_file_patch_cancel(tab_state, *active_tab, registry, args, tx);
            }
        }
        PendingCommand::NewCategory => {
            create_category_and_tab(
                tabs,
                active_tab,
                categories,
                active_category,
                registry,
                prompt_registry,
                args,
            );
        }
        PendingCommand::OpenConversation => {
            open_conversation_in_tab(
                tabs,
                active_tab,
                categories,
                active_category,
                registry,
                prompt_registry,
                args,
            );
        }
    }
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

fn create_category_and_tab(
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
    let name = active
        .app
        .pending_category_name
        .take()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| next_category_name(categories));
    if !categories.contains(&name) {
        categories.push(name.clone());
    }
    let conv_id = crate::conversation::new_conversation_id()
        .unwrap_or_else(|_| active.app.log_session_id.clone());
    let model_key = if registry.get(&active.app.model_key).is_some() {
        active.app.model_key.clone()
    } else {
        registry.default_key.clone()
    };
    let prompt_key = if prompt_registry.get(&active.app.prompt_key).is_some() {
        active.app.prompt_key.clone()
    } else {
        prompt_registry.default_key.clone()
    };
    let system = prompt_registry
        .get(&prompt_key)
        .map(|p| p.content.as_str())
        .unwrap_or(&args.system);
    let mut tab = TabState::new(
        conv_id,
        name.clone(),
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
    tab.app.dirty_indices = (0..tab.app.messages.len()).collect();
    tabs.push(tab);
    *active_category = categories.iter().position(|c| c == &name).unwrap_or(0);
    *active_tab = tabs.len().saturating_sub(1);
}

fn open_conversation_in_tab(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    let conv_id = {
        let Some(active) = tabs.get_mut(*active_tab) else {
            return;
        };
        active.app.pending_open_conversation.take()
    };
    let Some(conv_id) = conv_id else {
        return;
    };
    if let Some(idx) = tabs.iter().position(|t| t.conversation_id == conv_id) {
        *active_tab = idx;
        if let Some(tab) = tabs.get(*active_tab) {
            if let Some(pos) = categories.iter().position(|c| c == &tab.category) {
                *active_category = pos;
            }
        }
        return;
    }
    let conv = match crate::conversation::load_conversation(&conv_id) {
        Ok(c) => c,
        Err(e) => {
            if let Some(active) = tabs.get_mut(*active_tab) {
                active.app.messages.push(Message {
                    role: crate::types::ROLE_ASSISTANT.to_string(),
                    content: format!("打开对话失败：{e}"),
                    tool_call_id: None,
                    tool_calls: None,
                });
                active
                    .app
                    .dirty_indices
                    .push(active.app.messages.len().saturating_sub(1));
            }
            return;
        }
    };
    let category = if conv.category.trim().is_empty() {
        "默认".to_string()
    } else {
        conv.category.clone()
    };
    let model_key = conv
        .model_key
        .as_deref()
        .filter(|k| registry.get(k).is_some())
        .unwrap_or(&registry.default_key)
        .to_string();
    let prompt_key = conv
        .prompt_key
        .as_deref()
        .filter(|k| prompt_registry.get(k).is_some())
        .unwrap_or(&prompt_registry.default_key)
        .to_string();
    let mut tab = TabState::new(
        conv.id.clone(),
        category,
        "",
        false,
        &model_key,
        &prompt_key,
    );
    tab.app.messages = conv.messages.clone();
    tab.app.code_exec_container_id = conv.code_exec_container_id.clone();
    if tab
        .app
        .messages
        .iter()
        .all(|m| m.role != crate::types::ROLE_SYSTEM)
    {
        let content = prompt_registry
            .get(&prompt_key)
            .map(|p| p.content.as_str())
            .unwrap_or(&args.system);
        if !content.trim().is_empty() {
            tab.app.set_system_prompt(&prompt_key, content);
        }
    }
    if let Some(active) = tabs.get(*active_tab) {
        tab.app.prompts_dir = active.app.prompts_dir.clone();
        tab.app.tavily_api_key = active.app.tavily_api_key.clone();
        tab.app.set_log_session_id(&active.app.log_session_id);
    }
    tab.app.model_key = model_key;
    tab.app.prompt_key = prompt_key;
    tab.app.follow = true;
    tab.app.scroll = u16::MAX;
    tab.app.dirty_indices = (0..tab.app.messages.len()).collect();
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
