#[cfg(test)]
mod tests {
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::command_input::handle_command_line;
    use crate::ui::state::{App, PendingCommand};
    use std::fs;

    fn make_app() -> App {
        App::new("system", "model", "prompt")
    }

    #[test]
    fn exit_and_help_commands() {
        let mut app = make_app();
        let exit = handle_command_line("/exit", &mut app).unwrap();
        assert!(exit);
        let mut app = make_app();
        let exit = handle_command_line("/help", &mut app).unwrap();
        assert!(!exit);
        assert!(app.messages.iter().any(|m| m.content.contains("可用命令")));
    }

    #[test]
    fn reset_clears_messages_except_system() {
        let mut app = make_app();
        app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let _ = handle_command_line("/reset", &mut app).unwrap();
        assert_eq!(app.messages.len(), 1);
        assert_eq!(app.messages[0].role, crate::types::ROLE_SYSTEM);
    }

    #[test]
    fn save_sets_pending_command() {
        let mut app = make_app();
        let _ = handle_command_line("/save", &mut app).unwrap();
        assert_eq!(app.pending_command, Some(PendingCommand::SaveSession));
    }

    #[test]
    fn category_and_open_commands() {
        let mut app = make_app();
        let _ = handle_command_line("/category newcat", &mut app).unwrap();
        assert_eq!(app.pending_command, Some(PendingCommand::NewCategory));
        assert_eq!(app.pending_category_name.as_deref(), Some("newcat"));

        let mut app = make_app();
        let _ = handle_command_line("/open", &mut app).unwrap();
        assert!(app.messages.last().unwrap().content.contains("用法"));

        let mut app = make_app();
        let _ = handle_command_line("/open abc", &mut app).unwrap();
        assert_eq!(app.pending_command, Some(PendingCommand::OpenConversation));
        assert_eq!(app.pending_open_conversation.as_deref(), Some("abc"));
    }

    #[test]
    fn list_conversations_empty() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-list-conv");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_env("HOME", &temp.to_string_lossy());
        let conv_dir = temp.join(".local/share/deepseek/conversations");
        fs::create_dir_all(&conv_dir).unwrap();
        let mut app = make_app();
        let _ = handle_command_line("/list-conv", &mut app).unwrap();
        let content = app.messages.last().unwrap().content.clone();
        assert!(content.contains("对话"));
        restore_env("HOME", prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn unknown_command_adds_message() {
        let mut app = make_app();
        let _ = handle_command_line("/unknown", &mut app).unwrap();
        assert!(app.messages.last().unwrap().content.contains("未知命令"));
    }
}
