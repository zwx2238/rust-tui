#[cfg(test)]
mod tests {
    use crate::render::{
        RenderTheme, build_cache_entry, insert_empty_cache_entry, messages_to_plain_lines,
    };
    use crate::types::{Message, ROLE_USER};
    use ratatui::style::Color;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: None,
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: None,
        }
    }

    #[test]
    fn build_cache_entry_has_lines() {
        let msg = Message {
            role: ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        };
        let entry = build_cache_entry(&msg, 40, &theme(), false);
        assert!(entry.line_count > 0);
        assert!(entry.rendered);
    }

    #[test]
    fn insert_empty_cache_entry_extends() {
        let mut cache = Vec::new();
        insert_empty_cache_entry(&mut cache, 2, &theme());
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn messages_to_plain_lines_returns_text() {
        let msg = Message {
            role: ROLE_USER.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        };
        let lines = messages_to_plain_lines(&[msg], 40, &theme());
        assert!(!lines.is_empty());
    }
}
