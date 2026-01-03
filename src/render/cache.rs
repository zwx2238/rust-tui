use crate::render::MessageLayout;
use crate::render::markdown::{
    close_unbalanced_code_fence, count_markdown_lines, render_markdown_lines,
};
use crate::render::theme::{RenderTheme, theme_cache_key};
use crate::render::util::hash_message;
use crate::types::{Message, ROLE_ASSISTANT, ROLE_SYSTEM, ROLE_TOOL, ROLE_USER};
use ratatui::text::{Line, Text};
use std::borrow::Cow;

mod viewport;
use viewport::ViewportState;
#[derive(Clone)]
pub struct RenderCacheEntry {
    pub(crate) role: String,
    pub(crate) content_hash: u64,
    pub(crate) content_len: usize,
    pub(crate) width: usize,
    pub(crate) theme_key: u64,
    pub(crate) streaming: bool,
    pub(crate) lines: Vec<Line<'static>>,
    pub(crate) line_count: usize,
    pub(crate) rendered: bool,
}

fn empty_entry(theme_key: u64) -> RenderCacheEntry {
    RenderCacheEntry {
        role: String::new(),
        content_hash: 0,
        content_len: 0,
        width: 0,
        theme_key,
        streaming: false,
        lines: Vec::new(),
        line_count: 0,
        rendered: false,
    }
}
pub fn messages_to_text(
    messages: &[Message],
    width: usize,
    theme: &RenderTheme,
    label_suffixes: &[(usize, String)],
    streaming_idx: Option<usize>,
) -> Text<'static> {
    let mut cache = Vec::new();
    messages_to_text_cached(
        messages,
        width,
        theme,
        label_suffixes,
        streaming_idx,
        &mut cache,
    )
}
pub fn messages_to_text_cached(
    messages: &[Message],
    width: usize,
    theme: &RenderTheme,
    label_suffixes: &[(usize, String)],
    streaming_idx: Option<usize>,
    cache: &mut Vec<RenderCacheEntry>,
) -> Text<'static> {
    let (text, _) = messages_to_viewport_text_cached(
        messages,
        width,
        theme,
        label_suffixes,
        streaming_idx,
        0,
        u16::MAX,
        cache,
    );
    text
}
pub fn messages_to_plain_lines(
    messages: &[Message],
    width: usize,
    theme: &RenderTheme,
) -> Vec<String> {
    let mut out = Vec::new();
    let text = messages_to_text(messages, width, theme, &[], None);
    for line in text.lines {
        let mut s = String::new();
        for span in line.spans {
            s.push_str(&span.content);
        }
        out.push(s);
    }
    out
}
pub fn messages_to_viewport_text_cached(
    messages: &[Message],
    width: usize,
    theme: &RenderTheme,
    label_suffixes: &[(usize, String)],
    streaming_idx: Option<usize>,
    scroll: u16,
    height: u16,
    cache: &mut Vec<RenderCacheEntry>,
) -> (Text<'static>, usize) {
    let (text, total, _) = messages_to_viewport_text_cached_with_layout(
        messages,
        width,
        theme,
        label_suffixes,
        streaming_idx,
        scroll,
        height,
        cache,
    );
    (text, total)
}

pub fn messages_to_viewport_text_cached_with_layout(
    messages: &[Message],
    width: usize,
    theme: &RenderTheme,
    label_suffixes: &[(usize, String)],
    streaming_idx: Option<usize>,
    scroll: u16,
    height: u16,
    cache: &mut Vec<RenderCacheEntry>,
) -> (Text<'static>, usize, Vec<MessageLayout>) {
    let theme_key = theme_cache_key(theme);
    trim_cache(cache, messages.len());
    let start = scroll as usize;
    let end = start.saturating_add(height as usize);
    let mut state = ViewportState::new(
        width,
        theme,
        theme_key,
        label_suffixes,
        streaming_idx,
        start,
        end,
    );
    for (idx, msg) in messages.iter().enumerate() {
        state.process_message(idx, msg, cache);
    }
    state.finish()
}

fn trim_cache(cache: &mut Vec<RenderCacheEntry>, len: usize) {
    if cache.len() > len {
        cache.truncate(len);
    }
}


fn ensure_cache_entry<'a>(
    cache: &'a mut Vec<RenderCacheEntry>,
    idx: usize,
    theme_key: u64,
) -> &'a mut RenderCacheEntry {
    if cache.len() <= idx {
        cache.push(empty_entry(theme_key));
    }
    &mut cache[idx]
}

fn update_cache_entry(
    entry: &mut RenderCacheEntry,
    msg: &Message,
    width: usize,
    theme_key: u64,
    streaming: bool,
) {
    let content_hash = hash_message(&msg.role, &msg.content);
    let content_len = msg.content.len();
    let needs_update = entry.role != msg.role
        || entry.content_hash != content_hash
        || entry.content_len != content_len
        || entry.width != width
        || entry.theme_key != theme_key
        || entry.streaming != streaming;
    if !needs_update {
        return;
    }
    entry.role = msg.role.clone();
    entry.content_hash = content_hash;
    entry.content_len = content_len;
    entry.width = width;
    entry.theme_key = theme_key;
    entry.streaming = streaming;
    entry.lines.clear();
    entry.rendered = false;
    entry.line_count = count_message_lines(msg, width, streaming);
}
pub fn insert_empty_cache_entry(
    cache: &mut Vec<RenderCacheEntry>,
    idx: usize,
    theme: &RenderTheme,
) {
    let theme_key = theme_cache_key(theme);
    let entry = empty_entry(theme_key);
    if idx > cache.len() {
        cache.resize_with(idx, || entry.clone());
    }
    cache.insert(idx, entry);
}
pub fn build_cache_entry(
    msg: &Message,
    width: usize,
    theme: &RenderTheme,
    streaming: bool,
) -> RenderCacheEntry {
    let theme_key = theme_cache_key(theme);
    let content_hash = hash_message(&msg.role, &msg.content);
    let content_len = msg.content.len();
    let lines = render_message_content_lines(msg, width, theme, streaming);
    let line_count = lines.len();
    RenderCacheEntry {
        role: msg.role.clone(),
        content_hash,
        content_len,
        width,
        theme_key,
        streaming,
        lines,
        line_count,
        rendered: true,
    }
}
pub fn set_cache_entry(cache: &mut Vec<RenderCacheEntry>, idx: usize, entry: RenderCacheEntry) {
    if cache.len() <= idx {
        cache.resize_with(idx + 1, || empty_entry(entry.theme_key));
    }
    cache[idx] = entry;
}
fn render_message_content_lines(
    msg: &Message,
    width: usize,
    theme: &RenderTheme,
    streaming: bool,
) -> Vec<Line<'static>> {
    match msg.role.as_str() {
        ROLE_USER | ROLE_ASSISTANT | ROLE_SYSTEM | ROLE_TOOL => {
            let content = message_content(msg, streaming);
            render_markdown_lines(content.as_ref(), width, theme, streaming)
        }
        _ => Vec::new(),
    }
}
pub(crate) fn count_message_lines(msg: &Message, width: usize, streaming: bool) -> usize {
    match msg.role.as_str() {
        ROLE_USER | ROLE_ASSISTANT | ROLE_SYSTEM | ROLE_TOOL => {
            let content = message_content(msg, streaming);
            count_markdown_lines(content.as_ref(), width)
        }
        _ => 0,
    }
}

fn message_content<'a>(msg: &'a Message, streaming: bool) -> Cow<'a, str> {
    if streaming {
        Cow::Owned(close_unbalanced_code_fence(&msg.content))
    } else {
        Cow::Borrowed(&msg.content)
    }
}
