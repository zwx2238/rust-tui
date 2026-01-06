use crate::test_support::env_lock;
use crate::types::{ToolCall, ToolFunctionCall};
use crate::ui::tools::run_tool;
use std::fs;

use super::support::{setup_fake_docker, temp_dir, workspace};

#[test]
fn read_file_respects_root() {
    let _guard = env_lock().lock().unwrap();
    let ws = workspace();
    let _docker = setup_fake_docker(&ws);
    let root = ws.host_path.clone();
    let good_path = root.join("a.txt");
    fs::write(&good_path, "hello").unwrap();
    let result = run_tool(&read_file_call("1", &good_path), "", &ws);
    assert!(result.content.contains("[read_file]"));
    assert!(result.content.contains("hello"));
    let bad_path = temp_dir("tools-outside").join("b.txt");
    fs::write(&bad_path, "nope").unwrap();
    let result = run_tool(&read_file_call("2", &bad_path), "", &ws);
    assert!(result.content.contains("workspace 之外"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn read_file_empty_path_errors() {
    let call = ToolCall {
        id: "3".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "read_file".to_string(),
            arguments: r#"{"path":""}"#.to_string(),
        },
    };
    let result = run_tool(&call, "", &workspace());
    assert!(result.content.contains("path 不能为空"));
}

#[test]
fn read_file_invalid_json_errors() {
    let call = ToolCall {
        id: "3b".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "read_file".to_string(),
            arguments: "{".to_string(),
        },
    };
    let result = run_tool(&call, "", &workspace());
    assert!(result.content.contains("参数解析失败"));
}

#[test]
fn read_file_too_large_errors() {
    let _guard = env_lock().lock().unwrap();
    let ws = workspace();
    let _docker = setup_fake_docker(&ws);
    let dir = ws.host_path.clone();
    let path = dir.join("big.txt");
    let data = "a".repeat(1024);
    fs::write(&path, data).unwrap();
    let call = ToolCall {
        id: "3c".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "read_file".to_string(),
            arguments: format!(r#"{{"path":"{}","max_bytes":1}}"#, path.display()),
        },
    };
    let result = run_tool(&call, "", &ws);
    assert!(result.content.contains("文件过大"));
    let _ = fs::remove_dir_all(&ws.host_path);
}

#[test]
fn read_file_respects_line_range() {
    let _guard = env_lock().lock().unwrap();
    let ws = workspace();
    let _docker = setup_fake_docker(&ws);
    let dir = ws.host_path.clone();
    let path = dir.join("lines.txt");
    fs::write(&path, "a\nb\nc").unwrap();
    let call = ToolCall {
        id: "3d".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "read_file".to_string(),
            arguments: format!(
                r#"{{"path":"{}","start_line":2,"end_line":2}}"#,
                path.display()
            ),
        },
    };
    let result = run_tool(&call, "", &ws);
    assert!(result.content.contains("b"));
    let _ = fs::remove_dir_all(&ws.host_path);
}

#[test]
fn read_code_includes_line_numbers() {
    let _guard = env_lock().lock().unwrap();
    let ws = workspace();
    let _docker = setup_fake_docker(&ws);
    let dir = ws.host_path.clone();
    let path = dir.join("a.rs");
    fs::write(&path, "line1\nline2").unwrap();
    let call = ToolCall {
        id: "4".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "read_code".to_string(),
            arguments: format!(r#"{{"path":"{}"}}"#, path.display()),
        },
    };
    let result = run_tool(&call, "", &ws);
    assert!(result.content.contains("1 | line1"));
    let _ = fs::remove_dir_all(&ws.host_path);
}

fn read_file_call(id: &str, path: &std::path::Path) -> ToolCall {
    ToolCall {
        id: id.to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "read_file".to_string(),
            arguments: format!(r#"{{"path":"{}"}}"#, path.display()),
        },
    }
}
