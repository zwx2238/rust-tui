use crate::framework::widget_system::runtime::state::CodeExecReasonTarget;

#[derive(Copy, Clone)]
pub(super) struct CodeExecButtonsMode {
    pub(super) reason_target: Option<CodeExecReasonTarget>,
    pub(super) running: bool,
    pub(super) finished: bool,
}

pub(super) fn resolve_code_exec_mode(
    reason_target: Option<CodeExecReasonTarget>,
    live: Option<&crate::framework::widget_system::runtime::state::CodeExecLive>,
) -> CodeExecButtonsMode {
    let finished = live
        .map(|l| l.done || l.exit_code.is_some())
        .unwrap_or(false);
    let running = live.is_some() && !finished;
    CodeExecButtonsMode {
        reason_target,
        running,
        finished,
    }
}
