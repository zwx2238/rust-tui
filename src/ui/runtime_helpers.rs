use crate::render::{RenderCacheEntry, RenderTheme, insert_empty_cache_entry};
use crate::session::SessionTab;
use crate::types::{Message, ROLE_ASSISTANT, ROLE_USER};
use crate::ui::net::{UiEvent, request_llm_stream};
use crate::ui::perf::seed_perf_messages;
use crate::ui::state::{App, Focus, RequestHandle};
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::sync::{
    Arc,
    atomic::AtomicBool,
};
use std::thread;
use std::time::Instant;

pub(crate) struct TabState {
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
    pub(crate) fn new(system: &str, perf: bool, default_model: &str, default_prompt: &str) -> Self {
        let mut app = App::new(system, default_model, default_prompt);
        if perf {
            seed_perf_messages(&mut app);
            app.dirty_indices = (0..app.messages.len()).collect();
        }
        Self {
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

pub(crate) const PERF_QUESTIONS: [&str; 10] = [
    "用一句话解释什么是借用检查。",
    "用三点说明 async/await 的优势。",
    "写一个最小的 TCP echo 服务器示例。",
    "解释什么是零成本抽象。",
    "给出一个 Rust 中的错误处理最佳实践。",
    "简述 trait 和泛型的关系。",
    "解释生命周期标注的用途。",
    "提供一个并发安全的计数器示例。",
    "列出 3 个常用的性能分析工具。",
    "Rust 在系统编程中的典型应用场景有哪些？",
];

pub(crate) fn start_tab_request(
    tab_state: &mut TabState,
    question: &str,
    base_url: &str,
    api_key: &str,
    model: &str,
    show_reasoning: bool,
    tx: &mpsc::Sender<UiEvent>,
    tab_id: usize,
) {
    let app = &mut tab_state.app;
    if let Some(handle) = &app.active_request {
        handle.cancel();
        app.active_request = None;
    }
    if !question.is_empty() {
        app.messages.push(Message {
            role: ROLE_USER.to_string(),
            content: question.to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
    } else if let Some(line) = app.pending_send.take() {
        app.messages.push(Message {
            role: ROLE_USER.to_string(),
            content: line,
            tool_call_id: None,
            tool_calls: None,
        });
    } else {
        return;
    }
    if api_key.trim().is_empty() {
        app.messages.push(Message {
            role: ROLE_ASSISTANT.to_string(),
            content: "缺少 API Key，无法请求模型。".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        return;
    }
    let outbound_messages = app.messages.clone();
    let idx = app.messages.len();
    app.messages.push(Message {
        role: ROLE_ASSISTANT.to_string(),
        content: String::new(),
        tool_call_id: None,
        tool_calls: None,
    });
    let request_id = app.next_request_id;
    app.next_request_id = app.next_request_id.saturating_add(1);
    let cancel = Arc::new(AtomicBool::new(false));
    app.active_request = Some(RequestHandle {
        id: request_id,
        cancel: Arc::clone(&cancel),
    });
    app.busy = true;
    app.busy_since = Some(Instant::now());
    app.pending_assistant = Some(idx);
    app.pending_reasoning = None;
    app.stream_buffer.clear();
    app.follow = true;
    app.dirty_indices.push(idx);
    let messages = outbound_messages;
    let base_url = base_url.trim_end_matches('/').to_string();
    let url = format!("{base_url}/chat/completions");
    let api_key = api_key.to_string();
    let model = model.to_string();
    let tx = tx.clone();
    thread::spawn(move || {
        request_llm_stream(
            &url,
            &api_key,
            &model,
            show_reasoning,
            &messages,
            cancel,
            tx,
            tab_id,
            request_id,
        );
    });
}

pub(crate) fn tab_index_at(x: u16, area: ratatui::layout::Rect, tabs_len: usize) -> Option<usize> {
    let mut cursor = area.x;
    for i in 0..tabs_len {
        let w = crate::ui::logic::tab_label_width(i);
        let next = cursor.saturating_add(w);
        if x >= cursor && x < next {
            return Some(i);
        }
        cursor = next;
        if i + 1 < tabs_len {
            cursor = cursor.saturating_add(1);
        }
    }
    None
}

pub(crate) fn collect_session_tabs(tabs: &[TabState]) -> Vec<SessionTab> {
    tabs.iter()
        .map(|tab| SessionTab {
            messages: tab.app.messages.clone(),
            model_key: Some(tab.app.model_key.clone()),
            prompt_key: Some(tab.app.prompt_key.clone()),
        })
        .collect()
}

pub(crate) fn stop_and_edit(tab_state: &mut TabState) -> bool {
    let app = &mut tab_state.app;
    let Some(handle) = app.active_request.take() else {
        return false;
    };
    handle.cancel();
    let assistant_idx = app.pending_assistant.take();
    let reasoning_idx = app.pending_reasoning.take();
    let search_end = assistant_idx.unwrap_or(app.messages.len());
    let user_idx = app.messages[..search_end]
        .iter()
        .rposition(|m| m.role == ROLE_USER);
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
    if remove.is_empty() {
        app.busy = false;
        app.busy_since = None;
        app.stream_buffer.clear();
        return false;
    }
    remove.sort_unstable();
    remove.dedup();
    let user_text = user_idx
        .and_then(|idx| app.messages.get(idx).map(|m| m.content.clone()))
        .unwrap_or_default();
    for idx in remove.iter().rev() {
        if *idx < app.messages.len() {
            app.messages.remove(*idx);
        }
        if *idx < tab_state.render_cache.len() {
            tab_state.render_cache.remove(*idx);
        }
    }
    shift_stats_after_removals(&mut app.assistant_stats, &remove);
    app.stream_buffer.clear();
    app.busy = false;
    app.busy_since = None;
    app.follow = true;
    app.dirty_indices = (0..app.messages.len()).collect();
    app.input = tui_textarea::TextArea::default();
    if !user_text.is_empty() {
        app.input.insert_str(user_text);
    }
    app.focus = Focus::Input;
    true
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
