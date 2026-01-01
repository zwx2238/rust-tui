use crate::render::{insert_empty_cache_entry, RenderCacheEntry, RenderTheme};
use crate::types::Message;
use crate::ui::net::{request_llm_stream, UiEvent};
use crate::ui::perf::seed_perf_messages;
use crate::ui::state::App;
use std::sync::mpsc;
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
    if !question.is_empty() {
        app.messages.push(Message {
            role: "user".to_string(),
            content: question.to_string(),
        });
    } else if let Some(line) = app.pending_send.take() {
        app.messages.push(Message {
            role: "user".to_string(),
            content: line,
        });
    } else {
        return;
    }
    if api_key.trim().is_empty() {
        app.messages.push(Message {
            role: "assistant".to_string(),
            content: "缺少 API Key，无法请求模型。".to_string(),
        });
        return;
    }
    let outbound_messages = app.messages.clone();
    let idx = app.messages.len();
    app.messages.push(Message {
        role: "assistant".to_string(),
        content: String::new(),
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
            tx,
            tab_id,
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
