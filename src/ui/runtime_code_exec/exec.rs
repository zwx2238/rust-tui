use crate::ui::code_exec_container::{
    run_bash_in_container_stream, run_python_in_container_stream,
};
use crate::ui::runtime_code_exec::helpers::{mark_exec_error, mark_unsupported_language};
use crate::ui::state::{CodeExecLive, PendingCodeExec};

pub(super) fn spawn_exec(
    container_id: String,
    run_id: String,
    pending: PendingCodeExec,
    live: std::sync::Arc<std::sync::Mutex<CodeExecLive>>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    if pending.language == "python" {
        spawn_python_exec(container_id, run_id, pending, live, cancel);
    } else if pending.language == "bash" || pending.language == "sh" {
        spawn_bash_exec(container_id, run_id, pending, live, cancel);
    } else {
        mark_unsupported_language(&live, &pending.language);
    }
}

fn spawn_python_exec(
    container_id: String,
    run_id: String,
    pending: PendingCodeExec,
    live: std::sync::Arc<std::sync::Mutex<CodeExecLive>>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    std::thread::spawn(move || {
        let code = pending.exec_code.as_deref().unwrap_or(&pending.code);
        if let Err(err) =
            run_python_in_container_stream(&container_id, &run_id, code, live.clone(), cancel)
        {
            mark_exec_error(&live, err);
        }
    });
}

fn spawn_bash_exec(
    container_id: String,
    run_id: String,
    pending: PendingCodeExec,
    live: std::sync::Arc<std::sync::Mutex<CodeExecLive>>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    std::thread::spawn(move || {
        let code = pending.exec_code.as_deref().unwrap_or(&pending.code);
        if let Err(err) =
            run_bash_in_container_stream(&container_id, &run_id, code, live.clone(), cancel)
        {
            mark_exec_error(&live, err);
        }
    });
}
