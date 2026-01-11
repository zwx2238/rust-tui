use crate::llm::templates::RigTemplates;
use crate::types::Message as UiMessage;
use rig::completion::Message;

pub fn extract_system(messages: &[UiMessage]) -> String {
    messages
        .iter()
        .find(|m| m.role == crate::types::ROLE_SYSTEM)
        .map(|m| m.content.clone())
        .unwrap_or_default()
}

pub fn augment_system(base: &str) -> String {
    base.trim().to_string()
}

pub fn build_history_and_prompt(
    messages: &[UiMessage],
    templates: &RigTemplates,
    default_role: &str,
) -> Result<(Vec<Message>, String), String> {
    let last_user_idx = find_last_user_index(messages, default_role);
    if let Some(idx) = last_user_idx {
        if has_tool_after(messages, idx) {
            let history = build_history(messages, templates, None, default_role)?;
            let prompt = templates.render_followup()?;
            return Ok((history, prompt));
        }
        let history = build_history(messages, templates, Some(idx), default_role)?;
        return Ok((history, messages[idx].content.clone()));
    }
    let history = build_history(messages, templates, None, default_role)?;
    let prompt = templates.render_followup()?;
    Ok((history, prompt))
}

fn find_last_user_index(messages: &[UiMessage], default_role: &str) -> Option<usize> {
    messages
        .iter()
        .enumerate()
        .rev()
        .find(|(_, m)| is_input_role(&m.role, default_role))
        .map(|(idx, _)| idx)
}

fn has_tool_after(messages: &[UiMessage], idx: usize) -> bool {
    messages
        .iter()
        .skip(idx + 1)
        .any(|m| m.role == crate::types::ROLE_TOOL)
}

fn build_history(
    messages: &[UiMessage],
    templates: &RigTemplates,
    end: Option<usize>,
    default_role: &str,
) -> Result<Vec<Message>, String> {
    let slice = match end {
        Some(end) => &messages[..end],
        None => messages,
    };
    let mut history = Vec::new();
    let mut system_seen = false;
    for msg in slice {
        if should_skip_system(&msg.role, default_role, &mut system_seen) {
            continue;
        }
        if let Some(entry) = map_history_message(msg, templates, default_role)? {
            history.push(entry);
        }
    }
    Ok(history)
}

fn map_history_message(
    msg: &UiMessage,
    templates: &RigTemplates,
    default_role: &str,
) -> Result<Option<Message>, String> {
    if is_input_role(&msg.role, default_role) {
        return Ok(Some(Message::user(msg.content.clone())));
    }
    if msg.role == crate::types::ROLE_REASONING {
        return Ok(None);
    }
    if msg.role == crate::types::ROLE_TOOL {
        let wrapped =
            templates.render_tool_result("tool", &serde_json::Value::Null, &msg.content)?;
        return Ok(Some(Message::user(wrapped)));
    }
    if msg.role == crate::types::ROLE_SYSTEM {
        return Ok(None);
    }
    Ok(Some(match msg.role.as_str() {
        crate::types::ROLE_ASSISTANT => Message::assistant(msg.content.clone()),
        _ => Message::user(msg.content.clone()),
    }))
}

fn is_input_role(role: &str, default_role: &str) -> bool {
    role == default_role || role == crate::types::ROLE_USER
}

fn should_skip_system(role: &str, default_role: &str, seen: &mut bool) -> bool {
    if role != crate::types::ROLE_SYSTEM {
        return false;
    }
    if !*seen {
        *seen = true;
        return true;
    }
    default_role != crate::types::ROLE_SYSTEM
}
