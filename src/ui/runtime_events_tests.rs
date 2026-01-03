#[cfg(test)]
mod tests {
    use crate::ui::runtime_events::{handle_paste_event, handle_tab_category_click};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::state::Focus;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn handle_paste_event_inserts_text() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.focus = Focus::Input;
        handle_paste_event("a\r\nb", &mut tabs, 0);
        let text = tabs[0].app.input.lines().join("\n");
        assert_eq!(text, "a\nb");
    }

    #[test]
    fn handle_tab_category_click_updates_category() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "分类 2".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 0usize;
        let categories = vec!["默认".to_string(), "分类 2".to_string()];
        let mut active_category = 0usize;
        let tabs_area = Rect::new(0, 0, 20, 1);
        let category_area = Rect::new(0, 2, 10, 5);
        let handled = handle_tab_category_click(
            1,
            3,
            &mut tabs,
            &mut active_tab,
            &categories,
            &mut active_category,
            tabs_area,
            category_area,
        );
        assert!(handled);
        assert_eq!(active_category, 1);
        assert_eq!(active_tab, 1);
    }

    #[test]
    fn handle_tab_category_click_updates_tab() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 0usize;
        let categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let labels = crate::ui::runtime_helpers::tab_labels_for_category(&tabs, "默认");
        let tabs_area = Rect::new(0, 0, 20, 1);
        let category_area = Rect::new(0, 2, 10, 5);
        let second_tab_x = labels[0].width() as u16 + 1;
        let handled = handle_tab_category_click(
            second_tab_x,
            0,
            &mut tabs,
            &mut active_tab,
            &categories,
            &mut active_category,
            tabs_area,
            category_area,
        );
        assert!(handled);
        assert_eq!(active_tab, 1);
    }

    #[test]
    fn handle_tab_category_click_ignores_outside() {
        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        let mut active_tab = 0usize;
        let categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let handled = handle_tab_category_click(
            50,
            50,
            &mut tabs,
            &mut active_tab,
            &categories,
            &mut active_category,
            Rect::new(0, 0, 10, 1),
            Rect::new(0, 2, 10, 1),
        );
        assert!(!handled);
        assert_eq!(active_tab, 0);
        assert_eq!(active_category, 0);
    }

    #[test]
    fn mouse_scroll_updates_scroll() {
        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.scroll = 5;
        let mut active_tab = 0usize;
        let mut active_category = 0usize;
        let categories = vec!["默认".to_string()];
        let msg_area = Rect::new(0, 0, 40, 10);
        let input_area = Rect::new(0, 10, 40, 3);
        let tabs_area = Rect::new(0, 0, 40, 1);
        let category_area = Rect::new(0, 0, 10, 5);
        let theme = crate::render::RenderTheme {
            bg: ratatui::style::Color::Black,
            fg: Some(ratatui::style::Color::White),
            code_bg: ratatui::style::Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(ratatui::style::Color::Cyan),
        };
        let m = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        };
        crate::ui::runtime_events::handle_mouse_event(
            m,
            &mut tabs,
            &mut active_tab,
            &categories,
            &mut active_category,
            tabs_area,
            msg_area,
            input_area,
            category_area,
            40,
            10,
            100,
            &theme,
        );
        assert!(tabs[0].app.scroll < 5);
        let m = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        };
        crate::ui::runtime_events::handle_mouse_event(
            m,
            &mut tabs,
            &mut active_tab,
            &categories,
            &mut active_category,
            tabs_area,
            msg_area,
            input_area,
            category_area,
            40,
            10,
            100,
            &theme,
        );
        assert!(tabs[0].app.scroll >= 5);
    }

    #[test]
    fn ctrl_c_copies_chat_selection() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.focus = Focus::Chat;
        tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tabs[0].app.chat_selection = Some(crate::ui::selection::Selection {
            start: (0, 0),
            end: (0, 1),
        });
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let handled = crate::ui::runtime_events::handle_key_event(
            key,
            &mut tabs,
            0,
            40,
            &crate::render::RenderTheme {
                bg: ratatui::style::Color::Black,
                fg: Some(ratatui::style::Color::White),
                code_bg: ratatui::style::Color::Black,
                code_theme: "base16-ocean.dark",
                heading_fg: Some(ratatui::style::Color::Cyan),
            },
        )
        .unwrap();
        assert!(!handled);
    }

    #[test]
    fn mouse_down_on_scrollbar_starts_dragging() {
        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        let mut active_tab = 0usize;
        let mut active_category = 0usize;
        let categories = vec!["默认".to_string()];
        let msg_area = Rect::new(0, 0, 40, 10);
        let input_area = Rect::new(0, 10, 40, 3);
        let tabs_area = Rect::new(0, 0, 40, 1);
        let category_area = Rect::new(0, 0, 10, 5);
        let scroll_area = crate::ui::draw::scrollbar_area(msg_area);
        let theme = crate::render::RenderTheme {
            bg: ratatui::style::Color::Black,
            fg: Some(ratatui::style::Color::White),
            code_bg: ratatui::style::Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(ratatui::style::Color::Cyan),
        };
        let m = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: scroll_area.x,
            row: scroll_area.y,
            modifiers: KeyModifiers::NONE,
        };
        crate::ui::runtime_events::handle_mouse_event(
            m,
            &mut tabs,
            &mut active_tab,
            &categories,
            &mut active_category,
            tabs_area,
            msg_area,
            input_area,
            category_area,
            40,
            5,
            100,
            &theme,
        );
        assert!(tabs[0].app.scrollbar_dragging);
    }

    #[test]
    fn mouse_down_on_input_focuses_input() {
        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        let mut active_tab = 0usize;
        let mut active_category = 0usize;
        let categories = vec!["默认".to_string()];
        let msg_area = Rect::new(0, 0, 40, 10);
        let input_area = Rect::new(0, 10, 40, 3);
        let tabs_area = Rect::new(0, 0, 40, 1);
        let category_area = Rect::new(0, 0, 10, 5);
        let theme = crate::render::RenderTheme {
            bg: ratatui::style::Color::Black,
            fg: Some(ratatui::style::Color::White),
            code_bg: ratatui::style::Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(ratatui::style::Color::Cyan),
        };
        let m = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: input_area.x + 1,
            row: input_area.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        crate::ui::runtime_events::handle_mouse_event(
            m,
            &mut tabs,
            &mut active_tab,
            &categories,
            &mut active_category,
            tabs_area,
            msg_area,
            input_area,
            category_area,
            40,
            5,
            100,
            &theme,
        );
        assert_eq!(tabs[0].app.focus, Focus::Input);
        assert!(tabs[0].app.input_selecting);
    }
}
