#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::runtime_events_helpers::{hit_test_edit_button, selection_view_text};
    use crate::ui::runtime_helpers::TabState;
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

    #[test]
    fn selection_view_text_builds_text() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        tab.app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let text = selection_view_text(&mut tab, 40, &theme(), 10);
        assert!(!text.lines.is_empty());
    }

    #[test]
    fn hit_test_edit_button_returns_none_without_layouts() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let msg_area = Rect::new(0, 0, 40, 10);
        let hit = hit_test_edit_button(&mut tab, msg_area, 40, &theme(), 10, 5, 5);
        assert!(hit.is_none());
    }

    #[test]
    fn hit_test_edit_button_returns_none_outside_area() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        tab.app.message_layouts = vec![crate::render::MessageLayout {
            index: 0,
            label_line: 0,
            button_range: Some((0, 2)),
        }];
        let msg_area = Rect::new(10, 10, 10, 5);
        let hit = hit_test_edit_button(&mut tab, msg_area, 40, &theme(), 10, 0, 0);
        assert!(hit.is_none());
    }

    #[test]
    fn hit_test_edit_button_returns_index_when_hit() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        tab.app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab.app.message_layouts = vec![crate::render::MessageLayout {
            index: 7,
            label_line: 0,
            button_range: Some((0, 4)),
        }];
        let msg_area = Rect::new(0, 0, 40, 10);
        let hit = hit_test_edit_button(&mut tab, msg_area, 40, &theme(), 10, 2, 1);
        assert_eq!(hit, Some(7));
    }
}
