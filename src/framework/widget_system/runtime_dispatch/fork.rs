use crate::types::{ROLE_SYSTEM, ROLE_USER};
use crate::framework::widget_system::notice::push_notice;
use crate::framework::widget_system::runtime_dispatch::DispatchContext;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::runtime::state::Focus;

pub(crate) fn fork_message_into_new_tab(
    ctx: &mut DispatchContext<'_>,
    jump_rows: &[crate::framework::widget_system::widgets::jump::JumpRow],
    row_idx: usize,
) -> bool {
    let Some(row) = jump_rows.get(row_idx) else {
        return false;
    };
    let msg_idx = row.index.saturating_sub(1);
    fork_message_by_index(ctx, msg_idx)
}

pub(crate) fn fork_message_by_index(ctx: &mut DispatchContext<'_>, msg_idx: usize) -> bool {
    let seed = match build_fork_seed(ctx, msg_idx) {
        Ok(seed) => seed,
        Err(ForkError::NonUser) => {
            notify_fork_requires_user(ctx);
            return false;
        }
        Err(_) => return false,
    };
    let mut new_tab = build_fork_tab(ctx, &seed);
    seed_input_into_tab(&mut new_tab, &seed.content);
    ctx.tabs.push(new_tab);
    *ctx.active_tab = ctx.tabs.len().saturating_sub(1);
    sync_active_category(ctx);
    true
}

struct ForkSeed {
    content: String,
    history: Vec<crate::types::Message>,
    system_prompt: String,
    model_key: String,
    prompt_key: String,
    category: String,
    log_session_id: String,
    prompts_dir: String,
    tavily_api_key: String,
}

enum ForkError {
    NoTab,
    NoMessage,
    NonUser,
}

fn build_fork_seed(ctx: &DispatchContext<'_>, msg_idx: usize) -> Result<ForkSeed, ForkError> {
    let tab_state = ctx.tabs.get(*ctx.active_tab).ok_or(ForkError::NoTab)?;
    let msg = tab_state
        .app
        .messages
        .get(msg_idx)
        .ok_or(ForkError::NoMessage)?;
    if msg.role != ROLE_USER {
        return Err(ForkError::NonUser);
    }
    let content = msg.content.clone();
    let history = tab_state.app.messages[..msg_idx].to_vec();
    let system_prompt = resolve_fork_system_prompt(ctx, tab_state);
    let model_key = resolve_fork_model_key(ctx, tab_state);
    let prompt_key = resolve_fork_prompt_key(ctx, tab_state);
    Ok(ForkSeed {
        content,
        history,
        system_prompt,
        model_key,
        prompt_key,
        category: tab_state.category.clone(),
        log_session_id: tab_state.app.log_session_id.clone(),
        prompts_dir: tab_state.app.prompts_dir.clone(),
        tavily_api_key: tab_state.app.tavily_api_key.clone(),
    })
}

fn resolve_fork_system_prompt(ctx: &DispatchContext<'_>, tab_state: &TabState) -> String {
    tab_state
        .app
        .messages
        .iter()
        .find(|m| m.role == ROLE_SYSTEM)
        .map(|m| m.content.clone())
        .or_else(|| {
            ctx.prompt_registry
                .get(&tab_state.app.prompt_key)
                .map(|p| p.content.clone())
        })
        .unwrap_or_else(|| ctx.args.system.clone())
}

fn resolve_fork_model_key(ctx: &DispatchContext<'_>, tab_state: &TabState) -> String {
    if ctx.registry.get(&tab_state.app.model_key).is_some() {
        tab_state.app.model_key.clone()
    } else {
        ctx.registry.default_key.clone()
    }
}

fn resolve_fork_prompt_key(ctx: &DispatchContext<'_>, tab_state: &TabState) -> String {
    if ctx.prompt_registry.get(&tab_state.app.prompt_key).is_some() {
        tab_state.app.prompt_key.clone()
    } else {
        ctx.prompt_registry.default_key.clone()
    }
}

fn build_fork_tab(ctx: &mut DispatchContext<'_>, seed: &ForkSeed) -> TabState {
    let conv_id =
        crate::conversation::new_conversation_id().unwrap_or_else(|_| ctx.tabs.len().to_string());
    ensure_category(ctx, &seed.category);
    let mut new_tab = TabState::new(
        conv_id,
        seed.category.clone(),
        "",
        false,
        &seed.model_key,
        &seed.prompt_key,
    );
    new_tab.app.set_log_session_id(&seed.log_session_id);
    apply_fork_history(&mut new_tab, seed);
    new_tab.app.model_key = seed.model_key.clone();
    new_tab.app.prompt_key = seed.prompt_key.clone();
    new_tab.app.prompts_dir = seed.prompts_dir.clone();
    new_tab.app.tavily_api_key = seed.tavily_api_key.clone();
    new_tab
}

fn apply_fork_history(new_tab: &mut TabState, seed: &ForkSeed) {
    let mut history = seed.history.clone();
    if history.iter().all(|m| m.role != ROLE_SYSTEM) && !seed.system_prompt.trim().is_empty() {
        history.insert(
            0,
            crate::types::Message {
                role: ROLE_SYSTEM.to_string(),
                content: seed.system_prompt.clone(),
                tool_call_id: None,
                tool_calls: None,
            },
        );
    }
    new_tab.app.messages = history;
    new_tab.app.dirty_indices = (0..new_tab.app.messages.len()).collect();
}

fn seed_input_into_tab(new_tab: &mut TabState, content: &str) {
    if !content.is_empty() {
        new_tab.app.input.insert_str(content);
    }
    new_tab.app.focus = Focus::Input;
}

fn ensure_category(ctx: &mut DispatchContext<'_>, category: &str) {
    if !ctx.categories.contains(&category.to_string()) {
        ctx.categories.push(category.to_string());
    }
}

fn sync_active_category(ctx: &mut DispatchContext<'_>) {
    if let Some(tab) = ctx.tabs.get(*ctx.active_tab)
        && let Some(idx) = ctx.categories.iter().position(|c| c == &tab.category)
    {
        *ctx.active_category = idx;
    }
}

fn notify_fork_requires_user(ctx: &mut DispatchContext<'_>) {
    if let Some(active) = ctx.tabs.get_mut(*ctx.active_tab) {
        push_notice(&mut active.app, "仅支持从用户消息分叉。");
    }
}
