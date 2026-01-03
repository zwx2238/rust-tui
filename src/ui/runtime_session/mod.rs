mod fork;
mod helpers;
mod preheat;
mod restore;

use crate::args::Args;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use std::sync::mpsc;

type RestoreTabsResult =
    Result<(Vec<TabState>, usize, Vec<String>, usize), Box<dyn std::error::Error>>;

pub(crate) fn restore_tabs_from_session(
    session: &crate::session::SessionData,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> RestoreTabsResult {
    restore::restore_tabs_from_session(session, registry, prompt_registry, args)
}

pub(crate) fn fork_last_tab_for_retry(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> Option<(usize, String)> {
    fork::fork_last_tab_for_retry(tabs, active_tab, registry, prompt_registry, args)
}

pub(crate) fn spawn_preheat_workers(
    preheat_rx: mpsc::Receiver<PreheatTask>,
    preheat_res_tx: mpsc::Sender<PreheatResult>,
) {
    preheat::spawn_preheat_workers(preheat_rx, preheat_res_tx);
}
