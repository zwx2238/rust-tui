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
) -> (Vec<TabState>, usize) {
    let mut tabs = Vec::new();
    for tab in &session.tabs {
        let model_key = tab
            .model_key
            .as_deref()
            .filter(|k| registry.get(k).is_some())
            .unwrap_or(&registry.default_key)
            .to_string();
        let prompt_key = tab
            .prompt_key
            .as_deref()
            .filter(|k| prompt_registry.get(k).is_some())
            .unwrap_or(&prompt_registry.default_key)
            .to_string();
        let mut state = TabState::new("", false, &model_key, &prompt_key);
        state.app.messages = tab.messages.clone();
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
    if tabs.is_empty() {
        tabs.push(TabState::new(
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
        .active_tab
        .min(tabs.len().saturating_sub(1));
    (tabs, active_tab)
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
    let mut history: Vec<crate::types::Message> =
        source.app.messages[..msg_idx].to_vec();
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
    let mut new_tab = TabState::new("", false, &model_key, &prompt_key);
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
