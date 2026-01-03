#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::runtime_events::handle_mouse_event;
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::selection::Selection;
    use crate::ui::state::Focus;
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    fn layout() -> (Rect, Rect, Rect, Rect) {
        let msg_area = Rect::new(0, 1, 40, 10);
        let input_area = Rect::new(0, 11, 40, 3);
        let tabs_area = Rect::new(200, 200, 0, 0);
        let category_area = Rect::new(200, 200, 0, 0);
        (tabs_area, msg_area, input_area, category_area)
    }

    #[test]
    fn mouse_down_on_message_starts_chat_selection() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let mut active_tab = 0usize;
        let mut active_category = 0usize;
        let categories = vec!["默认".to_string()];
        let (tabs_area, msg_area, input_area, category_area) = layout();
        let m = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: msg_area.x + 2,
            row: msg_area.y + 2,
            modifiers: KeyModifiers::NONE,
        };
        let _ = handle_mouse_event(
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
            20,
            &theme(),
        );
        assert_eq!(tabs[0].app.focus, Focus::Chat);
        assert!(tabs[0].app.chat_selecting);
        assert!(tabs[0].app.chat_selection.is_some());
    }

    #[test]
    fn mouse_drag_updates_chat_selection() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello world".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tabs[0].app.chat_selecting = true;
        tabs[0].app.chat_selection = Some(Selection { start: (0, 0), end: (0, 0) });
        let mut active_tab = 0usize;
        let mut active_category = 0usize;
        let categories = vec!["默认".to_string()];
        let (tabs_area, msg_area, input_area, category_area) = layout();
        let m = MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: msg_area.x + 5,
            row: msg_area.y + 2,
            modifiers: KeyModifiers::NONE,
        };
        let _ = handle_mouse_event(
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
            20,
            &theme(),
        );
        let sel = tabs[0].app.chat_selection.unwrap();
        assert!(sel.end.1 >= sel.start.1);
    }

    #[test]
    fn mouse_up_clears_empty_selection() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.chat_selecting = true;
        tabs[0].app.chat_selection = Some(Selection { start: (0, 0), end: (0, 0) });
        let mut active_tab = 0usize;
        let mut active_category = 0usize;
        let categories = vec!["默认".to_string()];
        let (tabs_area, msg_area, input_area, category_area) = layout();
        let m = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: msg_area.x + 1,
            row: msg_area.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        let _ = handle_mouse_event(
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
            20,
            &theme(),
        );
        assert!(!tabs[0].app.chat_selecting);
        assert!(tabs[0].app.chat_selection.is_none());
    }

    #[test]
    fn mouse_down_on_edit_button_returns_index() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tabs[0].app.message_layouts = vec![crate::render::MessageLayout {
            index: 3,
            label_line: 0,
            button_range: Some((0, 4)),
        }];
        let mut active_tab = 0usize;
        let mut active_category = 0usize;
        let categories = vec!["默认".to_string()];
        let (tabs_area, msg_area, input_area, category_area) = layout();
        let m = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: msg_area.x + 2,
            row: msg_area.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        let hit = handle_mouse_event(
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
            20,
            &theme(),
        );
        assert_eq!(hit, Some(3));
    }
}
