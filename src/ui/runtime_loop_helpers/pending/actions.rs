use crate::args::Args;
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_code_exec::{
    handle_code_exec_approve, handle_code_exec_deny, handle_code_exec_exit, handle_code_exec_stop,
};
use crate::ui::runtime_file_patch::{handle_file_patch_apply, handle_file_patch_cancel};
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::PendingCommand;

enum CodeExecAction {
    Approve,
    Deny,
    Exit,
    Stop,
}

enum FilePatchAction {
    Apply,
    Cancel,
}

pub(crate) fn handle_code_exec_command(
    pending: PendingCommand,
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<RuntimeEvent>,
) -> bool {
    let action = match pending {
        PendingCommand::ApproveCodeExec => Some(CodeExecAction::Approve),
        PendingCommand::DenyCodeExec => Some(CodeExecAction::Deny),
        PendingCommand::ExitCodeExec => Some(CodeExecAction::Exit),
        PendingCommand::StopCodeExec => Some(CodeExecAction::Stop),
        _ => None,
    };
    if let Some(action) = action {
        handle_code_exec_action(tabs, active_tab, registry, args, tx, action);
        return true;
    }
    false
}

pub(crate) fn handle_file_patch_command(
    pending: PendingCommand,
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<RuntimeEvent>,
) -> bool {
    let action = match pending {
        PendingCommand::ApplyFilePatch => Some(FilePatchAction::Apply),
        PendingCommand::CancelFilePatch => Some(FilePatchAction::Cancel),
        _ => None,
    };
    if let Some(action) = action {
        handle_file_patch_action(tabs, active_tab, registry, args, tx, action);
        return true;
    }
    false
}

fn handle_code_exec_action(
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<RuntimeEvent>,
    action: CodeExecAction,
) {
    let Some(tab_state) = tabs.get_mut(active_tab) else {
        return;
    };
    match action {
        CodeExecAction::Approve => {
            handle_code_exec_approve(tab_state, active_tab, registry, args, tx)
        }
        CodeExecAction::Deny => handle_code_exec_deny(tab_state, active_tab, registry, args, tx),
        CodeExecAction::Exit => handle_code_exec_exit(tab_state, active_tab, registry, args, tx),
        CodeExecAction::Stop => handle_code_exec_stop(tab_state),
    }
}

fn handle_file_patch_action(
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<RuntimeEvent>,
    action: FilePatchAction,
) {
    let Some(tab_state) = tabs.get_mut(active_tab) else {
        return;
    };
    match action {
        FilePatchAction::Apply => {
            handle_file_patch_apply(tab_state, active_tab, registry, args, tx)
        }
        FilePatchAction::Cancel => {
            handle_file_patch_cancel(tab_state, active_tab, registry, args, tx)
        }
    }
}
