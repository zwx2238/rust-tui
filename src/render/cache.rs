use crate::render::markdown::{
    close_unbalanced_code_fence, count_markdown_lines, render_markdown_lines,
};
use crate::render::theme::{theme_cache_key, RenderTheme};
use crate::render::util::{
    hash_message, label_for_role, label_line, ranges_overlap, suffix_for_index,
};
use crate::types::Message;
use ratatui::text::{Line, Text};
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
    let theme_key = theme_cache_key(theme);
    if cache.len() > messages.len() {
        cache.truncate(messages.len());
    }
    let start = scroll as usize;
    let end = start.saturating_add(height as usize);
    let mut out: Vec<Line<'static>> = Vec::new();
    let mut line_cursor = 0usize;
    for (idx, msg) in messages.iter().enumerate() {
        if cache.len() <= idx {
            cache.push(RenderCacheEntry {
                role: String::new(),
                content_hash: 0,
                content_len: 0,
                width: 0,
                theme_key,
                streaming: false,
                lines: Vec::new(),
                line_count: 0,
                rendered: false,
            });
        }
        let suffix = suffix_for_index(label_suffixes, idx);
        let streaming = streaming_idx == Some(idx);
        let entry = &mut cache[idx];
        let content_hash = hash_message(&msg.role, &msg.content);
        let content_len = msg.content.len();
        if entry.role != msg.role
            || entry.content_hash != content_hash
            || entry.content_len != content_len
            || entry.width != width
            || entry.theme_key != theme_key
            || entry.streaming != streaming
        {
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
        if let Some(label) = label_for_role(&msg.role, suffix) {
            if line_cursor >= start && line_cursor < end {
                out.push(label_line(&label, theme));
            }
            line_cursor += 1;
            if !entry.rendered && ranges_overlap(start, end, line_cursor, line_cursor + entry.line_count)
            {
                entry.lines = render_message_content_lines(msg, width, theme, streaming);
                entry.rendered = true;
                entry.line_count = entry.lines.len();
            }
            let content_len = entry.line_count;
            if content_len > 0 {
                if line_cursor + content_len <= start || line_cursor >= end {
                    line_cursor += content_len;
                } else {
                    if entry.rendered {
                        for line in &entry.lines {
                            if line_cursor >= start && line_cursor < end {
                                out.push(line.clone());
                            }
                            line_cursor += 1;
                        }
                    } else {
                        line_cursor += content_len;
                    }
                }
            }
            if line_cursor >= start && line_cursor < end {
                out.push(Line::from(""));
            }
            line_cursor += 1;
        }
    }
    (Text::from(out), line_cursor)
}
pub fn insert_empty_cache_entry(cache: &mut Vec<RenderCacheEntry>, idx: usize, theme: &RenderTheme) {
    let theme_key = theme_cache_key(theme);
    let entry = RenderCacheEntry {
        role: String::new(),
        content_hash: 0,
        content_len: 0,
        width: 0,
        theme_key,
        streaming: false,
        lines: Vec::new(),
        line_count: 0,
        rendered: false,
    };
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
        cache.resize_with(idx + 1, || RenderCacheEntry {
            role: String::new(),
            content_hash: 0,
            content_len: 0,
            width: 0,
            theme_key: entry.theme_key,
            streaming: false,
            lines: Vec::new(),
            line_count: 0,
            rendered: false,
        });
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
        "user" | "assistant" => {
            let content = if streaming {
                close_unbalanced_code_fence(&msg.content)
            } else {
                msg.content.clone()
            };
            render_markdown_lines(&content, width, theme, streaming)
        }
        _ => Vec::new(),
    }
}
fn count_message_lines(msg: &Message, width: usize, streaming: bool) -> usize {
    match msg.role.as_str() {
        "user" | "assistant" => {
            let content = if streaming {
                close_unbalanced_code_fence(&msg.content)
            } else {
                msg.content.clone()
            };
            count_markdown_lines(&content, width)
        }
        _ => 0,
    }
}
