use crate::test_support::env_lock;
use crate::types::{ToolCall, ToolFunctionCall};
use crate::ui::tools::run_tool;
use std::fs;

use super::support::{setup_fake_docker, temp_dir, workspace};

#[test]
fn list_dir_outputs_entries() {
    let _guard = env_lock().lock().unwrap();
    let ws = workspace();
    let _docker = setup_fake_docker(&ws);
    let dir = ws.host_path.clone();
    let file = dir.join("a.txt");
    let sub = dir.join("sub");
    fs::write(&file, "hi").unwrap();
    fs::create_dir_all(&sub).unwrap();
    let call = ToolCall {
        id: "8".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "list_dir".to_string(),
            arguments: format!(r#"{{"path":"{}"}}"#, dir.display()),
        },
    };
    let result = run_tool(&call, "", &ws);
    assert!(result.content.contains("a.txt"));
    let _ = fs::remove_dir_all(&ws.host_path);
}

#[test]
fn list_dir_respects_root() {
    let _guard = env_lock().lock().unwrap();
    let ws = workspace();
    let _docker = setup_fake_docker(&ws);
    let root = ws.host_path.clone();
    let good = root.join("ok");
    fs::create_dir_all(&good).unwrap();
    let result = run_tool(&list_dir_call("9", &root), "", &ws);
    assert!(result.content.contains("[list_dir]"));
    let outside = temp_dir("tools-list-outside");
    let result = run_tool(&list_dir_call("10", &outside), "", &ws);
    assert!(result.content.contains("workspace 之外"));
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}

fn list_dir_call(id: &str, path: &std::path::Path) -> ToolCall {
    ToolCall {
        id: id.to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "list_dir".to_string(),
            arguments: format!(r#"{{"path":"{}"}}"#, path.display()),
        },
    }
}
