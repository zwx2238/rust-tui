#[cfg(test)]
mod tests {
    use crate::types::{Message, ROLE_ASSISTANT, ROLE_USER};
    use crate::ui::jump::{build_jump_rows, max_preview_width};
    use ratatui::layout::Rect;

    #[test]
    fn build_jump_rows_skips_unknown_roles() {
        let messages = vec![
            Message {
                role: ROLE_USER.to_string(),
                content: "hello".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: "unknown".to_string(),
                content: "tool output".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: ROLE_ASSISTANT.to_string(),
                content: "world".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
        ];
        let rows = build_jump_rows(&messages, 40, 10, None);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].index, 1);
        assert_eq!(rows[1].index, 3);
    }

    #[test]
    fn max_preview_width_has_minimum() {
        let area = Rect::new(0, 0, 10, 5);
        let width = max_preview_width(area);
        assert!(width >= 10);
    }
}
