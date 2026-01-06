#[cfg(test)]
mod tests {
    use crate::types::Usage;
    use crate::ui::events::LlmEvent;
    use crate::ui::logic::{
        StreamAction, build_label_suffixes, format_timer, handle_stream_event, point_in_rect,
        scroll_from_mouse, stop_stream,
    };
    use crate::ui::state::{App, RequestHandle};
    use ratatui::layout::Rect;
    use std::sync::{Arc, atomic::AtomicBool};

    #[test]
    fn format_timer_outputs_seconds() {
        assert!(format_timer(500).contains("0.5"));
        assert!(format_timer(60_000).contains("m"));
    }

    #[test]
    fn point_in_rect_checks_bounds() {
        let rect = Rect::new(0, 0, 10, 10);
        assert!(point_in_rect(5, 5, rect));
        assert!(!point_in_rect(11, 5, rect));
    }

    #[test]
    fn scroll_from_mouse_clamps() {
        let rect = Rect::new(0, 0, 1, 10);
        let scroll = scroll_from_mouse(100, 5, rect, 9);
        assert!(scroll <= 95);
    }

    #[test]
    fn build_label_suffixes_includes_stats() {
        let mut app = App::new("system", "model", "prompt");
        app.assistant_stats.insert(1, "stats".to_string());
        let suffixes = build_label_suffixes(&app, "");
        assert_eq!(suffixes.len(), 1);
    }

    #[test]
    fn handle_stream_event_updates_app() {
        let mut app = App::new("system", "model", "prompt");
        app.messages.push(crate::types::Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: String::new(),
            tool_call_id: None,
            tool_calls: None,
        });
        app.pending_assistant = Some(1);
        let action = handle_stream_event(&mut app, LlmEvent::Chunk("hi\n".to_string()), 0);
        assert!(matches!(action, StreamAction::None));
    }

    #[test]
    fn stop_stream_clears_request() {
        let mut app = App::new("system", "model", "prompt");
        app.messages.push(crate::types::Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: String::new(),
            tool_call_id: None,
            tool_calls: None,
        });
        app.pending_assistant = Some(1);
        app.active_request = Some(RequestHandle {
            id: 1,
            cancel: Arc::new(AtomicBool::new(false)),
        });
        assert!(stop_stream(&mut app));
        assert!(app.active_request.is_none());
    }

    #[test]
    fn done_event_updates_usage() {
        let mut app = App::new("system", "model", "prompt");
        app.messages.push(crate::types::Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: String::new(),
            tool_call_id: None,
            tool_calls: None,
        });
        app.pending_assistant = Some(1);
        let usage = Usage {
            prompt_tokens: Some(1),
            completion_tokens: Some(2),
            total_tokens: Some(3),
        };
        let action = handle_stream_event(&mut app, LlmEvent::Done { usage: Some(usage) }, 0);
        assert!(matches!(action, StreamAction::Done));
        assert_eq!(app.total_tokens, 3);
    }
}
