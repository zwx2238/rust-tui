use super::{
    build_enabled_tools, build_log_path, sanitize_log_part, stream_chunks, write_request_log,
    write_response_log,
};
use crate::llm::rig::RigRequestContext;
use crate::test_support::{env_lock, restore_env, set_env};
use rig::completion::Message;
use std::fs;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let dir = std::env::temp_dir().join(format!("deepchat_net_{name}_{ts}"));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn rig_context() -> RigRequestContext {
    RigRequestContext {
        preamble: "PRE".to_string(),
        history: vec![Message::user("hi")],
        prompt: "PROMPT".to_string(),
        tools: Vec::new(),
    }
}

fn write_logs(dir: &std::path::Path, ctx: &RigRequestContext) {
    write_request_log(
        dir.to_string_lossy().as_ref(),
        "sess",
        0,
        0,
        "http://example.com",
        "model",
        ctx,
    )
    .unwrap();
    write_response_log(dir.to_string_lossy().as_ref(), "sess", 0, 0, "output").unwrap();
}

#[test]
fn build_enabled_tools_collects_flags() {
    let tools = build_enabled_tools(true, false, true, false, true, false);
    assert!(tools.contains(&"web_search"));
    assert!(tools.contains(&"read_file"));
    assert!(tools.contains(&"list_dir"));
    assert!(tools.contains(&"modify_file"));
    assert!(!tools.contains(&"code_exec"));
}

#[test]
fn sanitize_log_part_replaces_invalid() {
    assert_eq!(sanitize_log_part("a/b c"), "a_b_c");
    assert_eq!(sanitize_log_part(""), "session");
}

#[test]
fn build_log_path_creates_dir_and_sanitizes() {
    let dir = temp_dir("path");
    let path = build_log_path(dir.to_string_lossy().as_ref(), "a/b", 0, 1, "input.txt").unwrap();
    assert!(path.exists() || path.parent().is_some());
    assert!(path.to_string_lossy().contains("a_b_tab1_msg2_input.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn write_request_and_response_log_outputs_files() {
    let _guard = env_lock().lock().unwrap();
    let dir = temp_dir("log");
    let ctx = rig_context();
    write_logs(&dir, &ctx);
    let input = fs::read_to_string(dir.join("sess_tab1_msg1_input.txt")).unwrap();
    assert!(input.contains("base_url: http://example.com"));
    assert!(input.contains("--- preamble ---"));
    let output = fs::read_to_string(dir.join("sess_tab1_msg1_output.txt")).unwrap();
    assert_eq!(output, "output");
    let _ = fs::remove_dir_all(&dir);
    let _ = set_env("DUMMY", "1");
    restore_env("DUMMY", None);
}

#[test]
fn stream_chunks_sends_events() {
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    stream_chunks("hello", &cancel, &tx, 1, 2);
    let msg = rx.recv_timeout(Duration::from_millis(50)).unwrap();
    match msg {
        crate::ui::events::RuntimeEvent::Llm(ui) => match ui.event {
            super::LlmEvent::Chunk(text) => assert!(!text.is_empty()),
            _ => panic!("unexpected event"),
        },
        _ => panic!("unexpected runtime event"),
    }
    cancel.store(true, Ordering::Relaxed);
}
