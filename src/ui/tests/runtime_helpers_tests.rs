#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::runtime_helpers::{
        PreheatTask, TabState, active_tab_position, collect_open_conversations,
        enqueue_preheat_tasks, stop_and_edit, tab_labels_for_category, tab_position_in_category,
        tab_to_conversation, visible_tab_indices,
    };
    use crate::ui::state::{Focus, RequestHandle};
    use ratatui::style::Color;
    use std::sync::{Arc, atomic::AtomicBool, mpsc};

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
    fn tab_index_helpers() {
        let tab1 = TabState::new("a".into(), "cat1".into(), "", false, "m", "p");
        let tab2 = TabState::new("b".into(), "cat2".into(), "", false, "m", "p");
        let tabs = vec![tab1, tab2];
        assert_eq!(visible_tab_indices(&tabs, "cat1"), vec![0]);
        assert_eq!(
            tab_labels_for_category(&tabs, "cat1"),
            vec![" 对话 1 ".to_string()]
        );
        assert_eq!(
            collect_open_conversations(&tabs),
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(active_tab_position(&tabs, "cat2", 1), 0);
        assert_eq!(tab_position_in_category(&tabs, "cat1", 0), Some(0));
    }

    #[test]
    fn tab_to_conversation_copies_fields() {
        let mut tab = TabState::new("id".into(), "cat".into(), "sys", false, "m", "p");
        tab.app.model_key = "m1".to_string();
        tab.app.prompt_key = "p1".to_string();
        let conv = tab_to_conversation(&tab);
        assert_eq!(conv.id, "id");
        assert_eq!(conv.category, "cat");
        assert_eq!(conv.model_key.as_deref(), Some("m1"));
    }

    #[test]
    fn enqueue_preheat_tasks_sends_tasks() {
        let mut tab = TabState::new("id".into(), "cat".into(), "sys", false, "m", "p");
        tab.app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab.app.dirty_indices = vec![0];
        let (tx, rx) = mpsc::channel::<PreheatTask>();
        enqueue_preheat_tasks(0, &mut tab, &theme(), 80, 1, &tx);
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn stop_and_edit_resets_state() {
        let mut tab = stop_and_edit_tab();
        let changed = stop_and_edit(&mut tab);
        assert!(changed);
        assert_eq!(tab.app.focus, Focus::Input);
        assert!(!tab.app.messages.iter().any(|m| m.content == "reply"));
        assert!(input_contains(&tab, "hello"));
    }

    fn stop_and_edit_tab() -> TabState {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        tab.app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab.app.messages.push(crate::types::Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "reply".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab.app.pending_assistant = Some(1);
        tab.app.active_request = Some(RequestHandle {
            id: 1,
            cancel: Arc::new(AtomicBool::new(false)),
        });
        tab
    }

    fn input_contains(tab: &TabState, needle: &str) -> bool {
        tab.app
            .input
            .lines()
            .first()
            .unwrap_or(&String::new())
            .contains(needle)
    }
}
