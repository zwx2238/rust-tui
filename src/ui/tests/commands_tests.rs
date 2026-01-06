#[cfg(test)]
mod tests {
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::commands::{
        command_has_args, command_suggestions_for_input, commands_help_text, list_conversation_ids,
    };
    use std::fs;

    #[test]
    fn commands_help_includes_exit() {
        let text = commands_help_text();
        assert!(text.contains("/exit"));
    }

    #[test]
    fn commands_help_includes_open_usage() {
        let text = commands_help_text();
        assert!(text.contains("/open <id>"));
    }

    #[test]
    fn command_args_detection() {
        assert!(command_has_args("/open"));
        assert!(!command_has_args("/help"));
    }

    #[test]
    fn command_args_detection_unknown() {
        assert!(!command_has_args("/unknown"));
    }

    #[test]
    fn list_conversation_ids_reads_dir() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-commands");
        let conv_dir = temp.join(".local/share/deepseek/conversations");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&conv_dir).unwrap();
        fs::write(conv_dir.join("a.json"), "{}").unwrap();
        fs::write(conv_dir.join("b.json"), "{}").unwrap();
        let prev = set_env("HOME", &temp.to_string_lossy());
        let ids = list_conversation_ids().unwrap();
        assert_eq!(ids, vec!["a".to_string(), "b".to_string()]);
        restore_env("HOME", prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn command_suggestions_for_input_prefix() {
        let suggestions = command_suggestions_for_input("/he", 3);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.insert == "/help"));
    }

    #[test]
    fn command_suggestions_empty_for_non_command() {
        let suggestions = command_suggestions_for_input("hello", 5);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn command_suggestions_empty_for_command_without_args() {
        let suggestions = command_suggestions_for_input("/help test", 9);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn command_suggestions_for_open_arguments() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-command-suggest");
        let conv_dir = temp.join(".local/share/deepseek/conversations");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&conv_dir).unwrap();
        fs::write(conv_dir.join("conv1.json"), "{}").unwrap();
        fs::write(conv_dir.join("test123.json"), "{}").unwrap();
        let prev = set_env("HOME", &temp.to_string_lossy());

        let line = "/open co";
        let suggestions = command_suggestions_for_input(line, line.chars().count());
        assert!(suggestions.iter().any(|s| s.insert == "conv1"));

        let line = "/open tst";
        let suggestions = command_suggestions_for_input(line, line.chars().count());
        assert!(suggestions.iter().any(|s| s.insert == "test123"));

        restore_env("HOME", prev);
        let _ = fs::remove_dir_all(&temp);
    }
}
