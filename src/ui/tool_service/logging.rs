use crate::args::Args;
use crate::types::ToolCall;
use crate::ui::workspace::resolve_workspace;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn log_modify_file_raw(args: &Args, call: &ToolCall) {
    if call.function.name != "modify_file" {
        return;
    }
    let workspace = match resolve_workspace(args) {
        Ok(val) => val,
        Err(_) => return,
    };
    let Some(mut file) = open_modify_file_log(&workspace) else {
        return;
    };
    write_modify_file_log(&mut file, call);
}

fn open_modify_file_log(
    workspace: &crate::ui::workspace::WorkspaceConfig,
) -> Option<std::fs::File> {
    let log_dir = workspace.host_path.join(".deepchat");
    if create_dir_all(&log_dir).is_err() {
        return None;
    }
    let log_path = log_dir.join("modify_file.log");
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok()
}

fn write_modify_file_log(file: &mut std::fs::File, call: &ToolCall) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let _ = writeln!(
        file,
        "ts={} id={} args={}",
        ts, call.id, call.function.arguments
    );
}
