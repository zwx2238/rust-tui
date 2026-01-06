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
    let mut out = String::new();
    if !base.trim().is_empty() {
        out.push_str(base.trim());
    }
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str("提示：公式渲染使用 txc，支持 LaTeX 子集，复杂公式可能无法渲染。");
    out.push_str(
        "\n\n工具约束：modify_file 仅接受 Git unified diff。必须包含 diff --git 行、---/+++ 行、\
@@ -a,b +c,d @@ hunk 头，并保持逐行换行。严禁使用 *** Begin Patch 或缺失 a/ b/ 前缀。",
    );
    if std::env::var("DEEPCHAT_READ_ONLY").is_ok() {
        out.push_str(
            "\n\n只读模式：禁止文件修改工具；代码执行仍可用，但仅允许在沙箱工作目录内写入临时文件。",
        );
    }
    out
}

pub fn build_history_and_prompt(
    messages: &[UiMessage],
    templates: &RigTemplates,
) -> Result<(Vec<Message>, String), String> {
    let last_user_idx = find_last_user_index(messages);
    if let Some(idx) = last_user_idx {
        if has_tool_after(messages, idx) {
            let history = build_history(messages, templates, None)?;
            let prompt = templates.render_followup()?;
            return Ok((history, prompt));
        }
        let history = build_history(messages, templates, Some(idx))?;
        return Ok((history, messages[idx].content.clone()));
    }
    let history = build_history(messages, templates, None)?;
    let prompt = templates.render_followup()?;
    Ok((history, prompt))
}

fn find_last_user_index(messages: &[UiMessage]) -> Option<usize> {
    messages
        .iter()
        .enumerate()
        .rev()
        .find(|(_, m)| m.role == crate::types::ROLE_USER)
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
) -> Result<Vec<Message>, String> {
    let slice = match end {
        Some(end) => &messages[..end],
        None => messages,
    };
    let mut history = Vec::new();
    for msg in slice {
        if let Some(entry) = map_history_message(msg, templates)? {
            history.push(entry);
        }
    }
    Ok(history)
}

fn map_history_message(
    msg: &UiMessage,
    templates: &RigTemplates,
) -> Result<Option<Message>, String> {
    if msg.role == crate::types::ROLE_SYSTEM {
        return Ok(None);
    }
    if msg.role == crate::types::ROLE_TOOL {
        let wrapped =
            templates.render_tool_result("tool", &serde_json::Value::Null, &msg.content)?;
        return Ok(Some(Message::user(wrapped)));
    }
    Ok(Some(match msg.role.as_str() {
        crate::types::ROLE_ASSISTANT => Message::assistant(msg.content.clone()),
        _ => Message::user(msg.content.clone()),
    }))
}

#[cfg(test)]
mod tests {
    use super::{augment_system, build_history_and_prompt, extract_system};
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::types::Message as UiMessage;
    use crate::types::{ROLE_ASSISTANT, ROLE_SYSTEM, ROLE_TOOL, ROLE_USER};
    use rig::completion::Message;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("deepchat_{name}_{ts}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn rig_templates() -> crate::llm::templates::RigTemplates {
        let dir = temp_dir("rig_prompt");
        let rig = dir.join("rig");
        fs::create_dir_all(&rig).unwrap();
        fs::write(
            rig.join("tools.json"),
            r#"[{"name":"tool","description":"t","parameters":{"type":"object"}}]"#,
        )
        .unwrap();
        fs::write(rig.join("tool_preamble.jinja"), "PRE").unwrap();
        fs::write(rig.join("tool_result.jinja"), "TOOL={{ output }}").unwrap();
        fs::write(rig.join("tool_followup.jinja"), "FOLLOWUP").unwrap();
        crate::llm::templates::RigTemplates::load(dir.to_string_lossy().as_ref()).unwrap()
    }

    #[test]
    fn augment_system_adds_read_only_note() {
        let _guard = env_lock().lock().unwrap();
        let prev = set_env("DEEPCHAT_READ_ONLY", "1");
        let out = augment_system("base");
        assert!(out.contains("只读模式"));
        restore_env("DEEPCHAT_READ_ONLY", prev);
    }

    #[test]
    fn extract_system_returns_first_system() {
        let msgs = vec![
            UiMessage {
                role: ROLE_USER.to_string(),
                content: "u".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            UiMessage {
                role: ROLE_SYSTEM.to_string(),
                content: "sys".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
        ];
        assert_eq!(extract_system(&msgs), "sys");
    }

    #[test]
    fn build_history_uses_last_user_as_prompt() {
        let templates = rig_templates();
        let msgs = messages_for_last_user_prompt();
        let (history, prompt) = build_history_and_prompt(&msgs, &templates).unwrap();
        assert_eq!(prompt, "last");
        assert!(history.iter().any(|m| message_text(m) == "first"));
    }

    #[test]
    fn build_history_wraps_tools_after_last_user() {
        let templates = rig_templates();
        let msgs = vec![
            UiMessage {
                role: ROLE_USER.to_string(),
                content: "ask".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            UiMessage {
                role: ROLE_TOOL.to_string(),
                content: "tool output".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
        ];
        let (history, prompt) = build_history_and_prompt(&msgs, &templates).unwrap();
        assert_eq!(prompt, "FOLLOWUP");
        assert!(
            history
                .iter()
                .any(|m| message_text(m).contains("TOOL=tool output"))
        );
    }

    fn message_text(msg: &Message) -> String {
        match msg {
            Message::User { content } => user_text(content),
            Message::Assistant { content, .. } => assistant_text(content),
        }
    }

    fn user_text(content: &rig::OneOrMany<rig::completion::message::UserContent>) -> String {
        content
            .iter()
            .filter_map(|item| match item {
                rig::completion::message::UserContent::Text(text) => Some(text.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn assistant_text(content: &rig::OneOrMany<rig::completion::AssistantContent>) -> String {
        content
            .iter()
            .filter_map(|item| match item {
                rig::completion::AssistantContent::Text(text) => Some(text.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn messages_for_last_user_prompt() -> Vec<UiMessage> {
        vec![
            UiMessage {
                role: ROLE_SYSTEM.to_string(),
                content: "sys".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            UiMessage {
                role: ROLE_USER.to_string(),
                content: "first".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            UiMessage {
                role: ROLE_ASSISTANT.to_string(),
                content: "ok".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            UiMessage {
                role: ROLE_USER.to_string(),
                content: "last".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
        ]
    }
}
