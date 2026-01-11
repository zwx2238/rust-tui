use crate::render::MessageLayout;
use crate::render::layout::{label_line_layout, label_line_with_button};
use crate::render::markdown::{
    close_unbalanced_code_fence, count_markdown_lines, render_markdown_lines,
};
use crate::render::theme::{RenderTheme, theme_cache_key};
use crate::render::util::{hash_message, label_for_role, ranges_overlap, suffix_for_index};
use crate::types::{Message, ROLE_ASSISTANT, ROLE_REASONING, ROLE_SYSTEM, ROLE_TOOL, ROLE_USER};
use ratatui::text::{Line, Text};
use std::borrow::Cow;

pub struct SingleMessageRenderParams<'a> {
    pub message: &'a Message,
    pub message_index: usize,
    pub width: usize,
    pub theme: &'a RenderTheme,
    pub label_suffixes: &'a [(usize, String)],
    pub streaming: bool,
    pub scroll: u16,
    pub height: u16,
}

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
pub fn message_to_viewport_text_cached(
    params: SingleMessageRenderParams<'_>,
    cache: &mut Vec<RenderCacheEntry>,
) -> (Text<'static>, usize) {
    let (text, total, _) = message_to_viewport_text_cached_with_layout(params, cache);
    (text, total)
}

pub fn message_to_viewport_text_cached_with_layout(
    params: SingleMessageRenderParams<'_>,
    cache: &mut Vec<RenderCacheEntry>,
) -> (Text<'static>, usize, Vec<MessageLayout>) {
    let theme_key = theme_cache_key(params.theme);
    let Some(label) = message_label(&params) else {
        return (Text::default(), 0, Vec::new());
    };
    let entry = ensure_cache_entry(cache, params.message_index, theme_key);
    update_cache_entry(entry, params.message, params.width, theme_key, params.streaming);
    let start = params.scroll as usize;
    let end = start.saturating_add(params.height as usize);
    let layout = build_single_layout(params.message_index, &params.message.role, &label);
    let mut out = Vec::new();
    let mut cursor = 0usize;
    push_line_if_visible(&mut out, label_line_with_button(&params.message.role, &label, params.theme), cursor, start, end);
    cursor += 1;
    cursor = push_message_content(&mut out, entry, cursor, start, end, &params);
    let _ = push_spacing_line(&mut out, cursor, start, end);
    let total = message_total_lines(entry.line_count);
    (Text::from(out), total, vec![layout])
}

pub fn message_to_plain_lines(
    params: SingleMessageRenderParams<'_>,
    cache: &mut Vec<RenderCacheEntry>,
) -> Vec<String> {
    let (text, _) = message_to_viewport_text_cached(params, cache);
    text_lines(&text)
}

fn ensure_cache_entry(
    cache: &mut Vec<RenderCacheEntry>,
    idx: usize,
    theme_key: u64,
) -> &mut RenderCacheEntry {
    if cache.len() <= idx {
        cache.resize_with(idx + 1, || empty_entry(theme_key));
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

fn message_label(params: &SingleMessageRenderParams<'_>) -> Option<String> {
    let suffix = suffix_for_index(params.label_suffixes, params.message_index);
    label_for_role(&params.message.role, suffix)
}

fn build_single_layout(index: usize, role: &str, label: &str) -> MessageLayout {
    let (button_range, label_line) = label_line_layout(role, label, 0);
    MessageLayout {
        index,
        label_line,
        button_range,
    }
}

fn message_total_lines(content_lines: usize) -> usize {
    content_lines.saturating_add(2)
}

fn push_line_if_visible(
    out: &mut Vec<Line<'static>>,
    line: Line<'static>,
    cursor: usize,
    start: usize,
    end: usize,
) {
    if cursor >= start && cursor < end {
        out.push(line);
    }
}

fn push_message_content(
    out: &mut Vec<Line<'static>>,
    entry: &mut RenderCacheEntry,
    cursor: usize,
    start: usize,
    end: usize,
    params: &SingleMessageRenderParams<'_>,
) -> usize {
    let content_len = entry.line_count;
    if content_len == 0 {
        return cursor;
    }
    if ranges_overlap(start, end, cursor, cursor + content_len) && !entry.rendered {
        entry.lines =
            render_message_content_lines(params.message, params.width, params.theme, params.streaming);
        entry.rendered = true;
        entry.line_count = entry.lines.len();
    }
    if cursor + entry.line_count <= start || cursor >= end {
        return cursor + entry.line_count;
    }
    if !entry.rendered {
        return cursor + entry.line_count;
    }
    let mut pos = cursor;
    for line in &entry.lines {
        if pos >= start && pos < end {
            out.push(line.clone());
        }
        pos += 1;
    }
    pos
}

fn push_spacing_line(
    out: &mut Vec<Line<'static>>,
    cursor: usize,
    start: usize,
    end: usize,
) -> usize {
    if cursor >= start && cursor < end {
        out.push(Line::from(""));
    }
    cursor + 1
}

fn text_lines(text: &Text<'_>) -> Vec<String> {
    let mut out = Vec::new();
    for line in &text.lines {
        let mut s = String::new();
        for span in &line.spans {
            s.push_str(&span.content);
        }
        out.push(s);
    }
    out
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
        ROLE_USER | ROLE_ASSISTANT | ROLE_REASONING | ROLE_SYSTEM | ROLE_TOOL => {
            let content = message_content(msg, streaming);
            render_markdown_lines(content.as_ref(), width, theme, streaming, false)
        }
        _ => Vec::new(),
    }
}
pub(crate) fn count_message_lines(msg: &Message, width: usize, streaming: bool) -> usize {
    match msg.role.as_str() {
        ROLE_USER | ROLE_ASSISTANT | ROLE_REASONING | ROLE_SYSTEM | ROLE_TOOL => {
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
