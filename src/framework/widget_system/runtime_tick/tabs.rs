use crate::framework::widget_system::runtime::runtime_helpers::TabState;

pub fn finalize_done_tabs(
    tabs: &mut [TabState],
    done_tabs: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    for &tab in done_tabs {
        if let Some(tab_state) = tabs.get_mut(tab) {
            tab_state.app.busy = false;
            tab_state.app.busy_since = None;
        }
    }
    Ok(())
}

pub fn update_tab_widths(tabs: &mut [TabState], msg_width: usize) {
    for tab_state in tabs.iter_mut() {
        tab_state.last_width = msg_width;
    }
}
