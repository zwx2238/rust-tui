use crate::args::Args;
use crate::model_registry::ModelRegistry;
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_code_exec::handle_code_exec_exit;
use crate::ui::runtime_helpers::TabState;
use std::sync::mpsc;

pub(crate) fn auto_finalize_code_exec(
    tabs: &mut [TabState],
    registry: &ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    if !args.yolo_enabled() {
        return;
    }
    for tab_state in tabs.iter_mut() {
        if tab_state.app.pending_code_exec.is_some() && tab_state.app.code_exec_result_ready {
            handle_code_exec_exit(tab_state, registry, args, tx);
        }
    }
}
