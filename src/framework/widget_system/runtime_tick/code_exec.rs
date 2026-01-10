use crate::services::runtime_code_exec_output::build_code_exec_tool_output;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;

pub fn update_code_exec_results(tabs: &mut [TabState]) {
    for tab_state in tabs.iter_mut() {
        update_tab_code_exec(tab_state);
    }
}

fn update_tab_code_exec(tab_state: &mut TabState) {
    if tab_state.app.pending_code_exec.is_none() || tab_state.app.code_exec_result_ready {
        return;
    }
    if !is_code_exec_done(tab_state) {
        return;
    }
    let output = build_code_exec_output(tab_state);
    if let Some(content) = output {
        tab_state.app.code_exec_finished_output = Some(content);
        tab_state.app.code_exec_result_ready = true;
    }
}

fn is_code_exec_done(tab_state: &TabState) -> bool {
    tab_state
        .app
        .code_exec_live
        .as_ref()
        .and_then(|live| live.lock().ok().map(|l| l.done))
        .unwrap_or(false)
}

fn build_code_exec_output(tab_state: &TabState) -> Option<String> {
    let pending = tab_state.app.pending_code_exec.clone()?;
    let live = tab_state.app.code_exec_live.clone()?;
    let live = live.lock().ok()?;
    Some(build_code_exec_tool_output(&pending, &live))
}
