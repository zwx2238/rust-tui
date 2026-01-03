#[cfg(test)]
mod tests {
    use crate::ui::net::UiEvent;
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_requests::{
        StartFollowupRequestParams, StartTabRequestParams, start_followup_request,
        start_tab_request,
    };
    use crate::ui::state::RequestHandle;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::mpsc;

    #[test]
    fn start_tab_request_with_missing_api_key() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        let (tx, _rx) = mpsc::channel::<UiEvent>();
        let log_session_id = tab.app.log_session_id.clone();
        start_tab_request(StartTabRequestParams {
            tab_state: &mut tab,
            question: "test",
            base_url: "http://example.com",
            api_key: "",
            model: "model",
            _show_reasoning: false,
            tx: &tx,
            tab_id: 0,
            enable_web_search: false,
            enable_code_exec: false,
            enable_read_file: false,
            enable_read_code: false,
            enable_modify_file: false,
            log_requests: None,
            log_session_id,
        });
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("缺少 API Key"))
        );
    }

    #[test]
    fn start_followup_request_with_missing_api_key() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        let (tx, _rx) = mpsc::channel::<UiEvent>();
        let log_session_id = tab.app.log_session_id.clone();
        start_followup_request(StartFollowupRequestParams {
            tab_state: &mut tab,
            base_url: "http://example.com",
            api_key: "",
            model: "model",
            _show_reasoning: false,
            tx: &tx,
            tab_id: 0,
            enable_web_search: false,
            enable_code_exec: false,
            enable_read_file: false,
            enable_read_code: false,
            enable_modify_file: false,
            log_requests: None,
            log_session_id,
        });
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("缺少 API Key"))
        );
    }

    #[test]
    fn start_tab_request_uses_pending_send() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        tab.app.pending_send = Some("hello".to_string());
        let (tx, _rx) = mpsc::channel::<UiEvent>();
        let log_session_id = tab.app.log_session_id.clone();
        start_tab_request(StartTabRequestParams {
            tab_state: &mut tab,
            question: "",
            base_url: "http://example.com",
            api_key: "key",
            model: "model",
            _show_reasoning: false,
            tx: &tx,
            tab_id: 0,
            enable_web_search: false,
            enable_code_exec: false,
            enable_read_file: false,
            enable_read_code: false,
            enable_modify_file: false,
            log_requests: None,
            log_session_id,
        });
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.role == crate::types::ROLE_USER && m.content == "hello")
        );
    }

    #[test]
    fn start_tab_request_no_question_no_pending_does_nothing() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        let (tx, _rx) = mpsc::channel::<UiEvent>();
        let log_session_id = tab.app.log_session_id.clone();
        start_tab_request(StartTabRequestParams {
            tab_state: &mut tab,
            question: "",
            base_url: "http://example.com",
            api_key: "key",
            model: "model",
            _show_reasoning: false,
            tx: &tx,
            tab_id: 0,
            enable_web_search: false,
            enable_code_exec: false,
            enable_read_file: false,
            enable_read_code: false,
            enable_modify_file: false,
            log_requests: None,
            log_session_id,
        });
        assert!(tab.app.messages.is_empty());
    }

    #[test]
    fn start_tab_request_cancels_active_request() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        let cancel = Arc::new(AtomicBool::new(false));
        tab.app.active_request = Some(RequestHandle {
            id: 1,
            cancel: cancel.clone(),
        });
        let (tx, _rx) = mpsc::channel::<UiEvent>();
        let log_session_id = tab.app.log_session_id.clone();
        start_tab_request(StartTabRequestParams {
            tab_state: &mut tab,
            question: "test",
            base_url: "http://example.com",
            api_key: "key",
            model: "model",
            _show_reasoning: false,
            tx: &tx,
            tab_id: 0,
            enable_web_search: false,
            enable_code_exec: false,
            enable_read_file: false,
            enable_read_code: false,
            enable_modify_file: false,
            log_requests: None,
            log_session_id,
        });
        assert!(cancel.load(std::sync::atomic::Ordering::Relaxed));
        assert!(tab.app.active_request.is_none());
    }

    #[test]
    fn start_followup_request_cancels_active_request() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        let cancel = Arc::new(AtomicBool::new(false));
        tab.app.active_request = Some(RequestHandle {
            id: 1,
            cancel: cancel.clone(),
        });
        let (tx, _rx) = mpsc::channel::<UiEvent>();
        let log_session_id = tab.app.log_session_id.clone();
        start_followup_request(StartFollowupRequestParams {
            tab_state: &mut tab,
            base_url: "http://example.com",
            api_key: "key",
            model: "model",
            _show_reasoning: false,
            tx: &tx,
            tab_id: 0,
            enable_web_search: false,
            enable_code_exec: false,
            enable_read_file: false,
            enable_read_code: false,
            enable_modify_file: false,
            log_requests: None,
            log_session_id,
        });
        assert!(cancel.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn start_tab_request_sets_busy_state() {
        let _guard = crate::test_support::env_lock().lock().unwrap();
        let prev = crate::test_support::set_env("DEEPCHAT_TEST_SKIP_REQUEST", "1");
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        let (tx, _rx) = mpsc::channel::<UiEvent>();
        let log_session_id = tab.app.log_session_id.clone();
        start_tab_request(StartTabRequestParams {
            tab_state: &mut tab,
            question: "test",
            base_url: "http://example.com",
            api_key: "key",
            model: "model",
            _show_reasoning: false,
            tx: &tx,
            tab_id: 0,
            enable_web_search: false,
            enable_code_exec: false,
            enable_read_file: false,
            enable_read_code: false,
            enable_modify_file: false,
            log_requests: None,
            log_session_id,
        });
        assert!(tab.app.busy);
        assert!(tab.app.pending_assistant.is_some());
        assert!(tab.app.active_request.is_some());
        crate::test_support::restore_env("DEEPCHAT_TEST_SKIP_REQUEST", prev);
    }

    #[test]
    fn start_followup_request_sets_busy_state() {
        let _guard = crate::test_support::env_lock().lock().unwrap();
        let prev = crate::test_support::set_env("DEEPCHAT_TEST_SKIP_REQUEST", "1");
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        let (tx, _rx) = mpsc::channel::<UiEvent>();
        let log_session_id = tab.app.log_session_id.clone();
        start_followup_request(StartFollowupRequestParams {
            tab_state: &mut tab,
            base_url: "http://example.com",
            api_key: "key",
            model: "model",
            _show_reasoning: false,
            tx: &tx,
            tab_id: 0,
            enable_web_search: false,
            enable_code_exec: false,
            enable_read_file: false,
            enable_read_code: false,
            enable_modify_file: false,
            log_requests: None,
            log_session_id,
        });
        assert!(tab.app.busy);
        assert!(tab.app.pending_assistant.is_some());
        assert!(tab.app.active_request.is_some());
        crate::test_support::restore_env("DEEPCHAT_TEST_SKIP_REQUEST", prev);
    }
}
