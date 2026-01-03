#[cfg(test)]
mod tests {
    use crate::ui::input::handle_key;
    use crate::ui::state::{App, Focus};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn app() -> App {
        App::new("", "m1", "p1")
    }

    #[test]
    fn esc_returns_true() {
        let mut app = app();
        app.focus = Focus::Input;
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let handled = handle_key(key, &mut app).unwrap();
        assert!(handled);
    }

    #[test]
    fn enter_sets_pending_send_and_clears_input() {
        let mut app = app();
        app.focus = Focus::Input;
        app.input.insert_str("hello");
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let handled = handle_key(key, &mut app).unwrap();
        assert!(!handled);
        assert_eq!(app.pending_send.as_deref(), Some("hello"));
        assert!(app.input.lines().join("").is_empty());
    }

    #[test]
    fn ctrl_u_clears_input() {
        let mut app = app();
        app.focus = Focus::Input;
        app.input.insert_str("abc");
        let key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let handled = handle_key(key, &mut app).unwrap();
        assert!(!handled);
        assert!(app.input.lines().join("").is_empty());
    }

    #[test]
    fn scroll_keys_update_scroll() {
        let mut app = app();
        app.focus = Focus::Chat;
        app.scroll = 10;
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        handle_key(key, &mut app).unwrap();
        assert_eq!(app.scroll, 9);
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        handle_key(key, &mut app).unwrap();
        assert_eq!(app.scroll, 10);
        let key = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);
        handle_key(key, &mut app).unwrap();
        assert_eq!(app.scroll, 0);
        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        handle_key(key, &mut app).unwrap();
        assert_eq!(app.scroll, u16::MAX);
        assert!(!app.follow);
    }

    #[test]
    fn ctrl_a_selects_all() {
        let mut app = app();
        app.focus = Focus::Input;
        app.input.insert_str("hello");
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let handled = handle_key(key, &mut app).unwrap();
        assert!(!handled);
        assert!(app.input.is_selecting());
    }

    #[test]
    fn ctrl_j_inserts_newline() {
        let mut app = app();
        app.focus = Focus::Input;
        app.input.insert_str("hello");
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);
        let handled = handle_key(key, &mut app).unwrap();
        assert!(!handled);
        assert!(app.input.lines().len() > 1);
    }

    #[test]
    fn tab_applies_command_suggestion() {
        let mut app = app();
        app.focus = Focus::Input;
        app.input.insert_str("/he");
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let handled = handle_key(key, &mut app).unwrap();
        assert!(!handled);
        let line = app.input.lines().join("");
        assert!(line.starts_with("/help"));
    }

    #[test]
    fn command_suggestions_navigation_moves_selection() {
        let mut app = app();
        app.focus = Focus::Input;
        app.input.insert_str("/");
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let _ = handle_key(key, &mut app).unwrap();
        let selected = app.command_select.selected;
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let _ = handle_key(key, &mut app).unwrap();
        assert!(app.command_select.selected >= selected);
    }

    #[test]
    fn pending_code_exec_blocks_input() {
        let mut app = app();
        app.focus = Focus::Input;
        app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "c1".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        app.input.insert_str("hi");
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let handled = handle_key(key, &mut app).unwrap();
        assert!(!handled);
        assert_eq!(app.input.lines().join(""), "hi");
    }

    #[test]
    fn f12_scrolls_to_bottom() {
        let mut app = app();
        app.focus = Focus::Chat;
        app.scroll = 1;
        let key = KeyEvent::new(KeyCode::F(12), KeyModifiers::NONE);
        let handled = handle_key(key, &mut app).unwrap();
        assert!(!handled);
        assert_eq!(app.scroll, u16::MAX);
        assert!(app.follow);
    }
}
