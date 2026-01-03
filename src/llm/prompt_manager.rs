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
    use super::augment_system;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn augment_system_adds_read_only_note() {
        let _guard = env_lock().lock().unwrap();
        unsafe { std::env::set_var("DEEPCHAT_READ_ONLY", "1") };
        let out = augment_system("base");
        assert!(out.contains("只读模式"));
        unsafe { std::env::remove_var("DEEPCHAT_READ_ONLY") };
    }
}
