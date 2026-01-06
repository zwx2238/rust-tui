use crate::ui::runtime_helpers::TabState;
use crate::ui::tool_service::ToolService;
use std::sync::mpsc;

use super::support::{args, registry_empty_key, tool_call};

#[test]
fn web_search_disabled_adds_error_message() {
    let registry = registry_empty_key();
    let args = args(None, false);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let calls = vec![tool_call("web_search", r#"{"query":"hi"}"#)];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(
        tab.app
            .messages
            .iter()
            .any(|m| m.content.contains("web_search 未启用"))
    );
}

#[test]
fn code_exec_enabled_sets_pending_request() {
    let registry = registry_empty_key();
    let args = args(Some("code_exec".to_string()), false);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let calls = vec![tool_call(
        "code_exec",
        r#"{"language":"python","code":"print(1)"}"#,
    )];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(tab.app.pending_code_exec.is_some());
}

#[test]
fn web_search_enabled_reports_missing_key() {
    let registry = registry_empty_key();
    let args = args(Some("web_search".to_string()), false);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let calls = vec![tool_call("web_search", r#"{"query":"hi"}"#)];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(
        tab.app
            .messages
            .iter()
            .any(|m| m.content.contains("tavily_api_key"))
    );
}

#[test]
fn code_exec_disabled_adds_error_message() {
    let registry = registry_empty_key();
    let args = args(Some("-code_exec".to_string()), false);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let calls = vec![tool_call(
        "code_exec",
        r#"{"language":"python","code":"print(1)"}"#,
    )];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(
        tab.app
            .messages
            .iter()
            .any(|m| m.content.contains("code_exec 未启用"))
    );
}
