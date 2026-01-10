use crate::render::RenderTheme;
use crate::framework::widget_system::runtime::runtime_helpers::{PreheatResult, PreheatTask, TabState, enqueue_preheat_tasks};
use std::sync::mpsc;

pub fn apply_preheat_results(preheat: &mut Vec<PreheatResult>, tabs: &mut [TabState]) {
    for result in preheat.drain(..) {
        if let Some(tab_state) = tabs.get_mut(result.tab) {
            crate::render::set_cache_entry(&mut tab_state.render_cache, result.idx, result.entry);
        }
    }
}

pub fn preheat_inactive_tabs(
    tabs: &mut [TabState],
    active_tab: usize,
    theme: &RenderTheme,
    msg_width: usize,
    preheat_tx: &mpsc::Sender<PreheatTask>,
) {
    for (idx, tab_state) in tabs.iter_mut().enumerate() {
        if idx != active_tab {
            enqueue_preheat_tasks(idx, tab_state, theme, msg_width, 32, preheat_tx);
        }
    }
}
