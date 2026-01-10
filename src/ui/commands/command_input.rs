use crate::types::{Message, ROLE_ASSISTANT, ROLE_SYSTEM};
use crate::ui::commands::{commands_help_text, list_conversation_ids};
use crate::ui::state::{App, PendingCommand};

pub(crate) fn handle_command_line(
    line: &str,
    app: &mut App,
) -> Result<bool, Box<dyn std::error::Error>> {
    if is_exit_cmd(line) {
        return Ok(true);
    }
    if handle_simple_commands(line, app)? {
        return Ok(false);
    }
    handle_command_action(line, app)?;
    Ok(false)
}

fn is_exit_cmd(line: &str) -> bool {
    matches!(line, "/exit" | "/quit")
}

fn is_reset_cmd(line: &str) -> bool {
    matches!(line, "/reset" | "/clear")
}

fn split_cmd(line: &str) -> (&str, &str) {
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();
    (cmd, arg)
}

fn handle_simple_commands(line: &str, app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    if is_reset_cmd(line) {
        reset_app(app);
        return Ok(true);
    }
    if line == "/save" {
        app.pending_command = Some(PendingCommand::SaveSession);
        return Ok(true);
    }
    if line == "/help" {
        push_help(app);
        return Ok(true);
    }
    Ok(false)
}

fn handle_command_action(line: &str, app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let (cmd, arg) = split_cmd(line);
    match cmd {
        "/category" => handle_category(app, arg),
        "/open" => handle_open(app, arg),
        "/list-conv" => handle_list_conv(app)?,
        _ => push_unknown(app, line),
    }
    Ok(())
}

fn reset_app(app: &mut App) {
    let system = app.messages.iter().find(|m| m.role == ROLE_SYSTEM).cloned();
    app.messages.clear();
    app.assistant_stats.clear();
    if let Some(sys) = system {
        app.messages.push(sys);
    }
    app.follow = false;
}

fn push_help(app: &mut App) {
    app.messages.push(Message {
        role: ROLE_ASSISTANT.to_string(),
        content: commands_help_text(),
        tool_call_id: None,
        tool_calls: None,
    });
}

fn handle_category(app: &mut App, arg: &str) {
    app.pending_category_name = if arg.is_empty() {
        None
    } else {
        Some(arg.to_string())
    };
    app.pending_command = Some(PendingCommand::NewCategory);
}

fn handle_open(app: &mut App, arg: &str) {
    if arg.is_empty() {
        push_notice(app, "用法：/open <conversation_id>");
        return;
    }
    app.pending_open_conversation = Some(arg.to_string());
    app.pending_command = Some(PendingCommand::OpenConversation);
}

fn handle_list_conv(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let ids = list_conversation_ids()?;
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
    Ok(())
}

fn push_unknown(app: &mut App, line: &str) {
    app.messages.push(Message {
        role: ROLE_ASSISTANT.to_string(),
        content: format!("未知命令：{line}"),
        tool_call_id: None,
        tool_calls: None,
    });
}

fn push_notice(app: &mut App, content: &str) {
    app.messages.push(Message {
        role: ROLE_ASSISTANT.to_string(),
        content: content.to_string(),
        tool_call_id: None,
        tool_calls: None,
    });
}
