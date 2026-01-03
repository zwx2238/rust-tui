#[cfg(test)]
mod tests {
    use crate::ui::runtime_dispatch::handle_nav_key;
    use crate::ui::state::{App, Focus};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn app() -> App {
        App::new("", "m1", "p1")
    }

    #[test]
    fn nav_mode_enters_on_g_in_chat() {
        let mut app = app();
        app.focus = Focus::Chat;
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(handle_nav_key(&mut app, key));
        assert!(app.nav_mode);
        assert_eq!(app.focus, Focus::Chat);
        assert!(!app.follow);
    }

    #[test]
    fn nav_mode_ignored_when_not_chat() {
        let mut app = app();
        app.focus = Focus::Input;
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!handle_nav_key(&mut app, key));
        assert!(!app.nav_mode);
    }

    #[test]
    fn nav_next_and_prev_update_scroll() {
        let mut app = app();
        app.focus = Focus::Chat;
        app.nav_mode = true;
        app.message_layouts = vec![
            crate::render::MessageLayout {
                index: 0,
                label_line: 2,
                button_range: None,
            },
            crate::render::MessageLayout {
                index: 1,
                label_line: 8,
                button_range: None,
            },
        ];
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(handle_nav_key(&mut app, key));
        assert_eq!(app.scroll, 2);
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(handle_nav_key(&mut app, key));
        assert_eq!(app.scroll, 8);
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(handle_nav_key(&mut app, key));
        assert_eq!(app.scroll, 2);
    }

    #[test]
    fn nav_mode_exits_on_esc() {
        let mut app = app();
        app.nav_mode = true;
        app.focus = Focus::Chat;
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(handle_nav_key(&mut app, key));
        assert!(!app.nav_mode);
    }

    #[test]
    fn nav_mode_handles_unknown_key_without_exiting() {
        let mut app = app();
        app.nav_mode = true;
        app.focus = Focus::Chat;
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(handle_nav_key(&mut app, key));
        assert!(app.nav_mode);
    }
}
