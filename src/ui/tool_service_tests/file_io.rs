use crate::ui::runtime_helpers::TabState;
use crate::ui::tool_service::ToolService;
use crate::test_support::env_lock;
use std::path::PathBuf;
use std::sync::mpsc;

use super::support::{args, registry_empty_key, setup_fake_docker, tool_call};

#[test]
fn read_file_enabled_reads_file() {
    let _guard = env_lock().lock().unwrap();
    let registry = registry_empty_key();
    let args = args(None, false);
    let _ = std::fs::create_dir_all(&args.workspace);
    let _docker = setup_fake_docker(&args.workspace);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let path = PathBuf::from(&args.workspace).join("deepchat_read_file.txt");
    std::fs::write(&path, "hello").unwrap();
    let calls = vec![tool_call(
        "read_file",
        &format!(r#"{{"path":"{}"}}"#, path.to_string_lossy()),
    )];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(tab.app.messages.iter().any(|m| m.content.contains("hello")));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn read_code_enabled_reads_file_with_numbers() {
    let _guard = env_lock().lock().unwrap();
    let registry = registry_empty_key();
    let args = args(None, false);
    let _ = std::fs::create_dir_all(&args.workspace);
    let _docker = setup_fake_docker(&args.workspace);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let path = PathBuf::from(&args.workspace).join("deepchat_read_code.rs");
    std::fs::write(&path, "line1\nline2").unwrap();
    let calls = vec![tool_call(
        "read_code",
        &format!(r#"{{"path":"{}"}}"#, path.to_string_lossy()),
    )];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(tab.app.messages.iter().any(|m| m.content.contains("1 | line1")));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn read_file_disabled_adds_error_message() {
    let registry = registry_empty_key();
    let args = args(Some("-read_file".to_string()), false);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let calls = vec![tool_call("read_file", r#"{"path":"a.txt"}"#)];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(tab.app.messages.iter().any(|m| m.content.contains("read_file 未启用")));
}

#[test]
fn list_dir_disabled_when_read_file_disabled() {
    let registry = registry_empty_key();
    let args = args(Some("-read_file".to_string()), false);
    let (tx, _rx) = mpsc::channel();
    let service = ToolService::new(&registry, &args, &tx);
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    let calls = vec![tool_call("list_dir", r#"{"path":"."}"#)];
    service.apply_tool_calls(&mut tab, 0, &calls);
    assert!(tab.app.messages.iter().any(|m| m.content.contains("list_dir 未启用")));
}
