use crate::render::{RenderCacheEntry, RenderTheme, insert_empty_cache_entry};
use crate::types::Message;
use crate::framework::widget_system::runtime::perf::seed_perf_messages;
use crate::framework::widget_system::runtime::state::{App, Focus};
use std::collections::BTreeMap;
use std::sync::mpsc;

pub(crate) struct TabState {
    pub(crate) conversation_id: String,
    pub(crate) category: String,
    pub(crate) app: App,
    pub(crate) render_cache: Vec<RenderCacheEntry>,
    pub(crate) last_width: usize,
}

pub(crate) struct PreheatTask {
    pub(crate) tab: usize,
    pub(crate) idx: usize,
    pub(crate) msg: Message,
    pub(crate) width: usize,
    pub(crate) theme: RenderTheme,
    pub(crate) streaming: bool,
}

pub(crate) struct PreheatResult {
    pub(crate) tab: usize,
    pub(crate) idx: usize,
    pub(crate) entry: RenderCacheEntry,
}

impl TabState {
    pub(crate) fn new(
        conversation_id: String,
        category: String,
        system: &str,
        perf: bool,
        default_model: &str,
        default_prompt: &str,
    ) -> Self {
        let mut app = App::new(system, default_model, default_prompt);
        if perf {
            seed_perf_messages(&mut app);
            app.dirty_indices = (0..app.messages.len()).collect();
        }
        Self {
            conversation_id,
            category,
            app,
            render_cache: Vec::new(),
            last_width: 0,
        }
    }

    pub(crate) fn apply_cache_shift(&mut self, theme: &RenderTheme) {
        if let Some(shift) = self.app.cache_shift.take() {
            insert_empty_cache_entry(&mut self.render_cache, shift, theme);
        }
    }
}

pub(crate) fn enqueue_preheat_tasks(
    tab_idx: usize,
    tab: &mut TabState,
    theme: &RenderTheme,
    width: usize,
    limit: usize,
    tx: &mpsc::Sender<PreheatTask>,
) {
    tab.apply_cache_shift(theme);
    let mut remaining = limit;
    while remaining > 0 {
        let idx = match tab.app.dirty_indices.pop() {
            Some(i) => i,
            None => break,
        };
        if let Some(msg) = tab.app.messages.get(idx).cloned() {
            let streaming = tab.app.pending_assistant == Some(idx);
            let _ = tx.send(PreheatTask {
                tab: tab_idx,
                idx,
                msg,
                width,
                theme: theme.clone(),
                streaming,
            });
        }
        remaining -= 1;
    }
}

pub(crate) fn visible_tab_indices(tabs: &[TabState], category: &str) -> Vec<usize> {
    tabs.iter()
        .enumerate()
        .filter(|(_, tab)| tab.category == category)
        .map(|(idx, _)| idx)
        .collect()
}

pub(crate) fn tab_labels_for_category(tabs: &[TabState], category: &str) -> Vec<String> {
    let visible = visible_tab_indices(tabs, category);
    visible
        .iter()
        .enumerate()
        .map(|(i, _)| format!(" 对话 {} ", i + 1))
        .collect()
}

pub(crate) fn collect_open_conversations(tabs: &[TabState]) -> Vec<String> {
    tabs.iter().map(|tab| tab.conversation_id.clone()).collect()
}

pub(crate) fn active_tab_position(tabs: &[TabState], category: &str, active_tab: usize) -> usize {
    let mut pos = 0usize;
    for (idx, tab) in tabs.iter().enumerate() {
        if tab.category == category {
            if idx == active_tab {
                return pos;
            }
            pos += 1;
        }
    }
    0
}

pub(crate) fn tab_position_in_category(
    tabs: &[TabState],
    category: &str,
    tab_index: usize,
) -> Option<usize> {
    let mut pos = 0usize;
    for (idx, tab) in tabs.iter().enumerate() {
        if tab.category == category {
            if idx == tab_index {
                return Some(pos);
            }
            pos += 1;
        }
    }
    None
}

pub(crate) fn tab_to_conversation(tab: &TabState) -> crate::conversation::ConversationData {
    crate::conversation::ConversationData {
        id: tab.conversation_id.clone(),
        category: tab.category.clone(),
        messages: tab.app.messages.clone(),
        model_key: Some(tab.app.model_key.clone()),
        prompt_key: Some(tab.app.prompt_key.clone()),
        code_exec_container_id: tab.app.code_exec_container_id.clone(),
    }
}

pub(crate) fn stop_and_edit(tab_state: &mut TabState) -> bool {
    let (remove, user_text) = {
        let app = &mut tab_state.app;
        if !cancel_active_request(app) {
            return false;
        }
        let assistant_idx = app.pending_assistant.take();
        let reasoning_idx = app.pending_reasoning.take();
        let user_idx = find_last_user_idx(app, assistant_idx);
        let remove = collect_remove_indices(assistant_idx, reasoning_idx, user_idx);
        if remove.is_empty() {
            clear_stream_state(app);
            return false;
        }
        let user_text = user_text_at(app, user_idx);
        (remove, user_text)
    };
    apply_message_removals(tab_state, &remove);
    reset_edit_state(&mut tab_state.app, &user_text);
    true
}

fn cancel_active_request(app: &mut App) -> bool {
    let Some(handle) = app.active_request.take() else {
        return false;
    };
    handle.cancel();
    true
}

fn find_last_user_idx(app: &App, assistant_idx: Option<usize>) -> Option<usize> {
    let search_end = assistant_idx.unwrap_or(app.messages.len());
    app.messages[..search_end]
        .iter()
        .rposition(|m| m.role == app.default_role || m.role == crate::types::ROLE_USER)
}

fn collect_remove_indices(
    assistant_idx: Option<usize>,
    reasoning_idx: Option<usize>,
    user_idx: Option<usize>,
) -> Vec<usize> {
    let mut remove = Vec::new();
    if let Some(idx) = assistant_idx {
        remove.push(idx);
    }
    if let Some(idx) = reasoning_idx {
        remove.push(idx);
    }
    if let Some(idx) = user_idx {
        remove.push(idx);
    }
    remove.sort_unstable();
    remove.dedup();
    remove
}

fn user_text_at(app: &App, user_idx: Option<usize>) -> String {
    user_idx
        .and_then(|idx| app.messages.get(idx).map(|m| m.content.clone()))
        .unwrap_or_default()
}

fn apply_message_removals(tab_state: &mut TabState, remove: &[usize]) {
    if remove.is_empty() {
        return;
    }
    let app = &mut tab_state.app;
    for idx in remove.iter().rev() {
        if *idx < app.messages.len() {
            app.messages.remove(*idx);
        }
        if *idx < tab_state.render_cache.len() {
            tab_state.render_cache.remove(*idx);
        }
    }
    shift_stats_after_removals(&mut app.assistant_stats, remove);
}

fn reset_edit_state(app: &mut App, user_text: &str) {
    clear_stream_state(app);
    app.dirty_indices = (0..app.messages.len()).collect();
    app.input = tui_textarea::TextArea::default();
    if !user_text.is_empty() {
        app.input.insert_str(user_text);
    }
    app.focus = Focus::Input;
}

fn clear_stream_state(app: &mut App) {
    app.stream_buffer.clear();
    app.busy = false;
    app.busy_since = None;
}

fn shift_stats_after_removals(stats: &mut BTreeMap<usize, String>, removed: &[usize]) {
    if stats.is_empty() || removed.is_empty() {
        return;
    }
    let mut removed_sorted = removed.to_vec();
    removed_sorted.sort_unstable();
    removed_sorted.dedup();
    let mut updated = BTreeMap::new();
    for (idx, val) in stats.iter() {
        if removed_sorted.binary_search(idx).is_ok() {
            continue;
        }
        let shift = removed_sorted.iter().filter(|r| **r < *idx).count();
        let new_idx = idx.saturating_sub(shift);
        updated.insert(new_idx, val.clone());
    }
    *stats = updated;
}
