use crate::args::Args;
use crate::types::{ROLE_SYSTEM, ROLE_USER};
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub(crate) fn restore_tabs_from_session(
    session: &crate::session::SessionData,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> Result<(Vec<TabState>, usize, Vec<String>, usize), Box<dyn std::error::Error>> {
    let mut tabs = Vec::new();
    let mut categories = session.categories.clone();
    if categories.is_empty() {
        categories.push("默认".to_string());
    }
    let active_category_name = if session.active_category.trim().is_empty() {
        categories[0].clone()
    } else {
        session.active_category.clone()
    };
    if !categories.contains(&active_category_name) {
        categories.push(active_category_name.clone());
    }
    for conv_id in &session.open_conversations {
        let conv = crate::conversation::load_conversation(conv_id)
            .map_err(|e| format!("无法读取对话 {conv_id}: {e}"))?;
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
        let category = if conv.category.trim().is_empty() {
            "默认".to_string()
        } else {
            conv.category.clone()
        };
        if !categories.contains(&category) {
            categories.push(category.clone());
        }
        let mut state = TabState::new(
            conv.id.clone(),
            category.clone(),
            "",
            false,
            &model_key,
            &prompt_key,
        );
        state.app.messages = conv.messages.clone();
        state.app.code_exec_container_id = conv.code_exec_container_id.clone();
        if state.app.messages.iter().all(|m| m.role != ROLE_SYSTEM) {
            let content = prompt_registry
                .get(&prompt_key)
                .map(|p| p.content.as_str())
                .unwrap_or(&args.system);
            if !content.trim().is_empty() {
                state.app.set_system_prompt(&prompt_key, content);
            }
        }
        state.app.follow = true;
        state.app.scroll = u16::MAX;
        state.app.dirty_indices = (0..state.app.messages.len()).collect();
        tabs.push(state);
    }
    categories.retain(|c| tabs.iter().any(|t| t.category == *c));
    if categories.is_empty() {
        categories.push("默认".to_string());
    }
    if tabs.is_empty() {
        let conv_id = crate::conversation::new_conversation_id()?;
        tabs.push(TabState::new(
            conv_id,
            active_category_name.clone(),
            prompt_registry
                .get(&prompt_registry.default_key)
                .map(|p| p.content.as_str())
                .unwrap_or(&args.system),
            false,
            &registry.default_key,
            &prompt_registry.default_key,
        ));
    }
    let active_tab = session
        .active_conversation
        .as_deref()
        .and_then(|id| tabs.iter().position(|t| t.conversation_id == id))
        .unwrap_or(0)
        .min(tabs.len().saturating_sub(1));
    let active_category = categories
        .iter()
        .position(|c| c == &active_category_name)
        .or_else(|| {
            tabs.get(active_tab)
                .and_then(|t| categories.iter().position(|c| c == &t.category))
        })
        .unwrap_or(0);
    Ok((tabs, active_tab, categories, active_category))
}

pub(crate) fn fork_last_tab_for_retry(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> Option<(usize, String)> {
    let source_idx = (*active_tab).min(tabs.len().saturating_sub(1));
    let Some(source) = tabs.get(source_idx) else {
        return None;
    };
    let mut last_user_idx = None;
    for (idx, msg) in source.app.messages.iter().enumerate().rev() {
        if msg.role == ROLE_USER {
            last_user_idx = Some(idx);
            break;
        }
    }
    let Some(msg_idx) = last_user_idx else {
        return None;
    };
    let Some(msg) = source.app.messages.get(msg_idx) else {
        return None;
    };
    let content = msg.content.clone();
    if content.trim().is_empty() {
        return None;
    }
    let mut history: Vec<crate::types::Message> = source.app.messages[..msg_idx].to_vec();
    let system_prompt = source
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
        .unwrap_or_else(|| args.system.clone());
    let model_key = if registry.get(&source.app.model_key).is_some() {
        source.app.model_key.clone()
    } else {
        registry.default_key.clone()
    };
    let prompt_key = if prompt_registry.get(&source.app.prompt_key).is_some() {
        source.app.prompt_key.clone()
    } else {
        prompt_registry.default_key.clone()
    };
    let conv_id = crate::conversation::new_conversation_id().ok()?;
    let mut new_tab = TabState::new(
        conv_id,
        source.category.clone(),
        "",
        false,
        &model_key,
        &prompt_key,
    );
    new_tab.app.set_log_session_id(&source.app.log_session_id);
    if history.iter().all(|m| m.role != ROLE_SYSTEM) && !system_prompt.trim().is_empty() {
        history.insert(
            0,
            crate::types::Message {
                role: ROLE_SYSTEM.to_string(),
                content: system_prompt,
                tool_call_id: None,
                tool_calls: None,
            },
        );
    }
    new_tab.app.messages = history;
    new_tab.app.model_key = model_key;
    new_tab.app.prompt_key = prompt_key;
    new_tab.app.prompts_dir = source.app.prompts_dir.clone();
    new_tab.app.tavily_api_key = source.app.tavily_api_key.clone();
    new_tab.app.dirty_indices = (0..new_tab.app.messages.len()).collect();
    tabs.push(new_tab);
    *active_tab = tabs.len().saturating_sub(1);
    Some((*active_tab, content))
}

pub(crate) fn spawn_preheat_workers(
    preheat_rx: mpsc::Receiver<PreheatTask>,
    preheat_res_tx: mpsc::Sender<PreheatResult>,
) {
    let workers = std::env::var("PREHEAT_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| (n.get() / 2).max(1))
                .unwrap_or(1)
        });
    let preheat_rx = Arc::new(Mutex::new(preheat_rx));
    for _ in 0..workers {
        let rx = Arc::clone(&preheat_rx);
        let tx = preheat_res_tx.clone();
        std::thread::spawn(move || {
            loop {
                let task = {
                    let guard = match rx.lock() {
                        Ok(g) => g,
                        Err(_) => break,
                    };
                    guard.recv().ok()
                };
                let task = match task {
                    Some(t) => t,
                    None => break,
                };
                let entry = crate::render::build_cache_entry(
                    &task.msg,
                    task.width,
                    &task.theme,
                    task.streaming,
                );
                let _ = tx.send(PreheatResult {
                    tab: task.tab,
                    idx: task.idx,
                    entry,
                });
            }
        });
    }
}
