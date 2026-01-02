use crate::types::{Message, ROLE_ASSISTANT, ROLE_SYSTEM};
use crate::ui::commands::{commands_help_text, list_conversation_ids};
use crate::ui::state::{App, PendingCommand};

pub(crate) fn handle_command_line(
    line: &str,
    app: &mut App,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();
    match line {
        "/exit" | "/quit" => return Ok(true),
        "/reset" | "/clear" => {
            let system = app.messages.iter().find(|m| m.role == ROLE_SYSTEM).cloned();
            app.messages.clear();
            app.assistant_stats.clear();
            if let Some(sys) = system {
                app.messages.push(sys);
            }
            app.follow = true;
        }
        "/save" => {
            app.pending_command = Some(PendingCommand::SaveSession);
        }
        "/help" => {
            app.messages.push(Message {
                role: ROLE_ASSISTANT.to_string(),
                content: commands_help_text(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        _ if cmd == "/category" => {
            app.pending_category_name = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            app.pending_command = Some(PendingCommand::NewCategory);
        }
        _ if cmd == "/open" => {
            if arg.is_empty() {
                app.messages.push(Message {
                    role: ROLE_ASSISTANT.to_string(),
                    content: "用法：/open <conversation_id>".to_string(),
                    tool_call_id: None,
                    tool_calls: None,
                });
            } else {
                app.pending_open_conversation = Some(arg.to_string());
                app.pending_command = Some(PendingCommand::OpenConversation);
            }
        }
        _ if cmd == "/list-conv" => match list_conversation_ids() {
            Ok(ids) => {
                let content = if ids.is_empty() {
                    "暂无对话文件。".to_string()
                } else {
                    format!("可用对话：\n{}", ids.join("\n"))
                };
                app.messages.push(Message {
                    role: ROLE_ASSISTANT.to_string(),
                    content,
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
            Err(e) => {
                app.messages.push(Message {
                    role: ROLE_ASSISTANT.to_string(),
                    content: format!("读取对话目录失败：{e}"),
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
        },
        _ => {
            app.messages.push(Message {
                role: ROLE_ASSISTANT.to_string(),
                content: format!("未知命令：{line}"),
                tool_call_id: None,
                tool_calls: None,
            });
        }
    }
    Ok(false)
}
