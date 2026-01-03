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
    let mut last_user_idx = None;
    for (idx, msg) in messages.iter().enumerate() {
        if msg.role == crate::types::ROLE_USER {
            last_user_idx = Some(idx);
        }
    }
    if let Some(idx) = last_user_idx {
        let has_tool_after_user = messages
            .iter()
            .skip(idx + 1)
            .any(|m| m.role == crate::types::ROLE_TOOL);
        if has_tool_after_user {
            let mut history = Vec::new();
            for msg in messages {
                if msg.role == crate::types::ROLE_SYSTEM {
                    continue;
                }
                if msg.role == crate::types::ROLE_TOOL {
                    let wrapped = templates.render_tool_result(
                        "tool",
                        &serde_json::Value::Null,
                        &msg.content,
                    )?;
                    history.push(Message {
                        role: "user".to_string(),
                        content: wrapped,
                    });
                } else {
                    history.push(Message {
                        role: msg.role.clone(),
                        content: msg.content.clone(),
                    });
                }
            }
            return Ok((history, templates.render_followup()?));
        }
    }
    let mut history = Vec::new();
    if let Some(idx) = last_user_idx {
        for msg in &messages[..idx] {
            if msg.role == crate::types::ROLE_SYSTEM {
                continue;
            }
            if msg.role == crate::types::ROLE_TOOL {
                let wrapped =
                    templates.render_tool_result("tool", &serde_json::Value::Null, &msg.content)?;
                history.push(Message {
                    role: "user".to_string(),
                    content: wrapped,
                });
            } else {
                history.push(Message {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                });
            }
        }
        Ok((history, messages[idx].content.clone()))
    } else {
        for msg in messages {
            if msg.role == crate::types::ROLE_SYSTEM {
                continue;
            }
            if msg.role == crate::types::ROLE_TOOL {
                let wrapped =
                    templates.render_tool_result("tool", &serde_json::Value::Null, &msg.content)?;
                history.push(Message {
                    role: "user".to_string(),
                    content: wrapped,
                });
            } else {
                history.push(Message {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                });
            }
        }
        Ok((history, templates.render_followup()?))
    }
}

#[cfg(test)]
mod tests {
    use super::{augment_system, build_history_and_prompt, extract_system};
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::types::Message as UiMessage;
    use crate::types::{ROLE_ASSISTANT, ROLE_SYSTEM, ROLE_TOOL, ROLE_USER};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
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
        let msgs = vec![
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
        ];
        let (history, prompt) = build_history_and_prompt(&msgs, &templates).unwrap();
        assert_eq!(prompt, "last");
        assert!(history.iter().any(|m| m.content == "first"));
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
        assert!(history.iter().any(|m| m.content.contains("TOOL=tool output")));
    }
}
