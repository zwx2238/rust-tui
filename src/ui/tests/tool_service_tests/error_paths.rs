use crate::ui::runtime_helpers::TabState;
use crate::ui::tool_service::ToolService;
use std::sync::mpsc;

use super::support::{args, registry_empty_key, tool_call};

#[test]
fn modify_file_blocked_in_read_only_mode() {
    let registry = registry_empty_key();
    let mut args = args(None, false);
    args.read_only = true;
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let calls = vec![tool_call(
        "modify_file",
        r#"{"diff":"diff --git a/a b/a\n","path":"a"}"#,
    )];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(
        tab.app
            .messages
            .iter()
            .any(|m| m.content.contains("read_only"))
    );
}

#[test]
fn modify_file_invalid_json_reports_error() {
    let registry = registry_empty_key();
    let args = args(Some("modify_file".to_string()), false);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let calls = vec![tool_call("modify_file", "{")];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(
        tab.app
            .messages
            .iter()
            .any(|m| m.content.contains("modify_file 参数解析失败"))
    );
}

#[test]
fn empty_tool_calls_adds_fallback_message() {
    let registry = registry_empty_key();
    let args = args(None, false);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    service.apply_tool_calls(&mut tab, 0, &[]);
    assert!(
        tab.app
            .messages
            .iter()
            .any(|m| m.content.contains("未找到可靠结果"))
    );
}
