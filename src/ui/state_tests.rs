#[cfg(test)]
mod tests {
    use crate::ui::state::App;

    #[test]
    fn app_initializes_with_system_prompt() {
        let app = App::new("sys", "model", "prompt");
        assert!(!app.messages.is_empty());
        assert_eq!(app.messages[0].role, crate::types::ROLE_SYSTEM);
    }

    #[test]
    fn set_system_prompt_updates_message() {
        let mut app = App::new("sys", "model", "prompt");
        app.set_system_prompt("p2", "new");
        assert_eq!(app.prompt_key, "p2");
        assert!(app.messages[0].content.contains("new"));
    }

    #[test]
    fn set_log_session_id_updates_field() {
        let mut app = App::new("sys", "model", "prompt");
        app.set_log_session_id("abc");
        assert_eq!(app.log_session_id, "abc");
    }
}
