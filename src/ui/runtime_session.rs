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
    let (mut categories, active_category_name) = init_categories(session);
    let mut tabs = load_tabs(session, registry, prompt_registry, args, &mut categories)?;
    ensure_default_tab(&mut tabs, &active_category_name, registry, prompt_registry, args)?;
    let active_tab = resolve_active_tab(session, &tabs);
    let active_category = resolve_active_category(&active_category_name, &categories, &tabs, active_tab);
    Ok((tabs, active_tab, categories, active_category))
}

pub(crate) fn fork_last_tab_for_retry(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> Option<(usize, String)> {
    let source = tabs.get((*active_tab).min(tabs.len().saturating_sub(1)))?;
    let (msg_idx, content) = last_user_message(source)?;
    let mut history: Vec<crate::types::Message> = source.app.messages[..msg_idx].to_vec();
    let system_prompt = resolve_system_prompt(source, prompt_registry, args);
    let model_key = resolve_model_key(source, registry);
    let prompt_key = resolve_prompt_key(source, prompt_registry);
    let conv_id = crate::conversation::new_conversation_id().ok()?;
    let mut new_tab = TabState::new(conv_id, source.category.clone(), "", false, &model_key, &prompt_key);
    new_tab.app.set_log_session_id(&source.app.log_session_id);
    insert_system_prompt(&mut history, &system_prompt);
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
    let workers = resolve_worker_count();
    let preheat_rx = Arc::new(Mutex::new(preheat_rx));
    for _ in 0..workers {
        spawn_preheat_worker(Arc::clone(&preheat_rx), preheat_res_tx.clone());
    }
}

fn init_categories(session: &crate::session::SessionData) -> (Vec<String>, String) {
    let mut categories = session.categories.clone();
    if categories.is_empty() { categories.push("默认".to_string()); }
    let name = if session.active_category.trim().is_empty() { categories[0].clone() } else { session.active_category.clone() };
    if !categories.contains(&name) { categories.push(name.clone()); }
    (categories, name)
}

fn load_tabs(
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
        let mut state = TabState::new(conv.id.clone(), category, "", false, &model_key, &prompt_key);
        apply_conversation_state(&mut state, &conv, &prompt_key, prompt_registry, args);
        tabs.push(state);
    }
    if categories.is_empty() { categories.push("默认".to_string()); }
    Ok(tabs)
}

fn normalize_category(category: &str, categories: &mut Vec<String>) -> String {
    let value = if category.trim().is_empty() { "默认".to_string() } else { category.to_string() };
    if !categories.contains(&value) { categories.push(value.clone()); }
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
    if state.app.messages.iter().any(|m| m.role == ROLE_SYSTEM) { return; }
    let content = prompt_registry.get(prompt_key).map(|p| p.content.as_str()).unwrap_or(&args.system);
    if !content.trim().is_empty() { state.app.set_system_prompt(prompt_key, content); }
}

fn ensure_default_tab(
    tabs: &mut Vec<TabState>,
    active_category_name: &str,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    if !tabs.is_empty() { return Ok(()); }
    let conv_id = crate::conversation::new_conversation_id()?;
    let prompt = prompt_registry.get(&prompt_registry.default_key).map(|p| p.content.as_str()).unwrap_or(&args.system);
    tabs.push(TabState::new(conv_id, active_category_name.to_string(), prompt, false, &registry.default_key, &prompt_registry.default_key));
    Ok(())
}

fn resolve_active_tab(session: &crate::session::SessionData, tabs: &[TabState]) -> usize {
    session
        .active_conversation
        .as_deref()
        .and_then(|id| tabs.iter().position(|t| t.conversation_id == id))
        .unwrap_or(0)
        .min(tabs.len().saturating_sub(1))
}

fn resolve_active_category(
    active_category_name: &str,
    categories: &[String],
    tabs: &[TabState],
    active_tab: usize,
) -> usize {
    categories
        .iter()
        .position(|c| c == active_category_name)
        .or_else(|| tabs.get(active_tab).and_then(|t| categories.iter().position(|c| c == &t.category)))
        .unwrap_or(0)
}

fn last_user_message(source: &TabState) -> Option<(usize, String)> {
    let mut last_user_idx = None;
    for (idx, msg) in source.app.messages.iter().enumerate().rev() {
        if msg.role == ROLE_USER { last_user_idx = Some(idx); break; }
    }
    let msg_idx = last_user_idx?;
    let msg = source.app.messages.get(msg_idx)?;
    let content = msg.content.clone();
    if content.trim().is_empty() { return None; }
    Some((msg_idx, content))
}

fn resolve_system_prompt(
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
        .or_else(|| prompt_registry.get(&source.app.prompt_key).map(|p| p.content.clone()))
        .unwrap_or_else(|| args.system.clone())
}

fn resolve_model_key(
    source: &TabState,
    registry: &crate::model_registry::ModelRegistry,
) -> String {
    if registry.get(&source.app.model_key).is_some() { source.app.model_key.clone() }
    else { registry.default_key.clone() }
}

fn resolve_prompt_key(
    source: &TabState,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
) -> String {
    if prompt_registry.get(&source.app.prompt_key).is_some() { source.app.prompt_key.clone() }
    else { prompt_registry.default_key.clone() }
}

fn insert_system_prompt(history: &mut Vec<crate::types::Message>, system_prompt: &str) {
    if history.iter().any(|m| m.role == ROLE_SYSTEM) || system_prompt.trim().is_empty() { return; }
    history.insert(0, crate::types::Message { role: ROLE_SYSTEM.to_string(), content: system_prompt.to_string(), tool_call_id: None, tool_calls: None });
}

fn resolve_worker_count() -> usize {
    std::env::var("PREHEAT_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism().map(|n| (n.get() / 2).max(1)).unwrap_or(1)
        })
}

fn spawn_preheat_worker(
    preheat_rx: Arc<Mutex<mpsc::Receiver<PreheatTask>>>,
    preheat_res_tx: mpsc::Sender<PreheatResult>,
) {
    std::thread::spawn(move || {
        loop {
            let task = {
                let guard = match preheat_rx.lock() { Ok(g) => g, Err(_) => break };
                guard.recv().ok()
            };
            let task = match task { Some(t) => t, None => break };
            let entry = crate::render::build_cache_entry(&task.msg, task.width, &task.theme, task.streaming);
            let _ = preheat_res_tx.send(PreheatResult { tab: task.tab, idx: task.idx, entry });
        }
    });
}
