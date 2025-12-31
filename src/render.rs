use crate::config::Config;
use crate::types::Message;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser as MdParser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use textwrap::wrap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct RenderTheme {
    pub bg: Color,
    pub fg: Option<Color>,
    pub code_bg: Color,
    pub code_theme: &'static str,
    pub heading_fg: Option<Color>,
}

pub struct RenderCacheEntry {
    role: String,
    content_hash: u64,
    content_len: usize,
    width: usize,
    theme_key: u64,
    streaming: bool,
    lines: Vec<Line<'static>>,
}

pub fn theme_from_config(cfg: Option<&Config>) -> RenderTheme {
    let name = cfg
        .and_then(|c| c.theme.as_deref())
        .unwrap_or("light")
        .to_ascii_lowercase();
    if name == "light" {
        RenderTheme {
            bg: Color::White,
            fg: Some(Color::Black),
            code_bg: Color::White,
            code_theme: "base16-ocean.light",
            heading_fg: Some(Color::Blue),
        }
    } else {
        RenderTheme {
            bg: Color::Black,
            fg: None,
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
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
            entry.lines = render_message_content_lines(msg, width, theme, streaming);
        }
        if let Some(label) = label_for_role(&msg.role, suffix) {
            if line_cursor >= start && line_cursor < end {
                out.push(label_line(&label, theme));
            }
            line_cursor += 1;

            let content_len = entry.lines.len();
            if content_len > 0 {
                if line_cursor + content_len <= start || line_cursor >= end {
                    line_cursor += content_len;
                } else {
                    for line in &entry.lines {
                        if line_cursor >= start && line_cursor < end {
                            out.push(line.clone());
                        }
                        line_cursor += 1;
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

fn render_message_content_lines(
    msg: &Message,
    width: usize,
    theme: &RenderTheme,
    streaming: bool,
) -> Vec<Line<'static>> {
    match msg.role.as_str() {
        "user" => {
            let content = if streaming {
                close_unbalanced_code_fence(&msg.content)
            } else {
                msg.content.clone()
            };
            render_markdown_lines(&content, width, theme)
        }
        "assistant" => {
            let content = if streaming {
                close_unbalanced_code_fence(&msg.content)
            } else {
                msg.content.clone()
            };
            render_markdown_lines(&content, width, theme)
        }
        _ => Vec::new(),
    }
}

fn suffix_for_index<'a>(suffixes: &'a [(usize, String)], idx: usize) -> Option<&'a str> {
    suffixes
        .iter()
        .find(|(i, _)| *i == idx)
        .map(|(_, s)| s.as_str())
}

fn label_for_role(role: &str, suffix: Option<&str>) -> Option<String> {
    match role {
        "user" => Some("👤".to_string()),
        "assistant" => {
            let mut label = "🤖".to_string();
            if let Some(s) = suffix {
                if !s.is_empty() {
                    label.push(' ');
                    label.push_str(s);
                }
            }
            Some(label)
        }
        _ => None,
    }
}

fn theme_cache_key(theme: &RenderTheme) -> u64 {
    let mut hasher = DefaultHasher::new();
    theme.bg.hash(&mut hasher);
    theme.fg.hash(&mut hasher);
    theme.code_bg.hash(&mut hasher);
    theme.code_theme.hash(&mut hasher);
    theme.heading_fg.hash(&mut hasher);
    hasher.finish()
}

fn hash_message(role: &str, content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    role.hash(&mut hasher);
    content.hash(&mut hasher);
    hasher.finish()
}

fn close_unbalanced_code_fence(input: &str) -> String {
    let mut fence_count = 0usize;
    for line in input.lines() {
        if line.trim_start().starts_with("```") {
            fence_count += 1;
        }
    }
    if fence_count % 2 == 1 {
        let mut out = String::with_capacity(input.len() + 4);
        out.push_str(input);
        if !input.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```");
        out
    } else {
        input.to_string()
    }
}

fn label_line(text: &str, theme: &RenderTheme) -> Line<'static> {
    let style = Style::default()
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White))
        .add_modifier(Modifier::BOLD);
    Line::from(Span::styled(text.to_string(), style))
}

fn render_paragraph_lines(text: &str, width: usize, theme: &RenderTheme) -> Vec<Line<'static>> {
    let style = Style::default().fg(theme.fg.unwrap_or(Color::White));
    wrap(text, width.max(10))
        .into_iter()
        .map(|line| Line::from(Span::styled(line.to_string(), style)))
        .collect()
}

fn render_heading_lines(text: &str, level: HeadingLevel, width: usize, theme: &RenderTheme) -> Vec<Line<'static>> {
    let ch = match level {
        HeadingLevel::H1 => '=',
        HeadingLevel::H2 => '-',
        HeadingLevel::H3 => '~',
        _ => '.',
    };
    let rule = ch.to_string().repeat(width.max(10));
    let style = Style::default()
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White))
        .add_modifier(Modifier::BOLD);
    vec![
        Line::from(Span::styled(rule.clone(), style)),
        Line::from(Span::styled(text.to_string(), style)),
        Line::from(Span::styled(rule, style)),
    ]
}

fn render_code_block_lines(
    text: &str,
    lang: &str,
    theme: &RenderTheme,
) -> Vec<Line<'static>> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme_name = theme.code_theme;
    let syn_theme = ts
        .themes
        .get(theme_name)
        .unwrap_or_else(|| ts.themes.values().next().expect("theme set is empty"));
    let syntax = ss
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let mut highlighter = HighlightLines::new(syntax, syn_theme);

    let lines: Vec<&str> = text.lines().collect();
    let max_digits = lines.len().max(1).to_string().len();
    let mut out = Vec::new();
    let code_fg = theme.fg.unwrap_or(Color::White);
    let code_bg = theme.bg;
    let bg_luma = color_luma(code_bg);
    for (i, raw) in lines.iter().enumerate() {
        let mut line_with_nl = String::with_capacity(raw.len() + 1);
        line_with_nl.push_str(raw);
        line_with_nl.push('\n');
        let ranges = highlighter
            .highlight_line(&line_with_nl, &ss)
            .unwrap_or_default();
        let line_no = format!("{:>width$} | ", i + 1, width = max_digits);
        let mut spans = Vec::new();
        spans.push(Span::styled(
            line_no,
            Style::default().fg(code_fg).bg(code_bg),
        ));
        for (style, part) in ranges {
            let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
            let span_fg = if (color_luma(fg) as i16 - bg_luma as i16).abs() < 80 {
                code_fg
            } else {
                fg
            };
            let span_style = Style::default().fg(span_fg).bg(code_bg);
            spans.push(Span::styled(part.to_string(), span_style));
        }
        out.push(Line::from(spans));
    }
    out
}

fn color_luma(color: Color) -> u8 {
    match color {
        Color::Rgb(r, g, b) => {
            let l = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
            l.round().clamp(0.0, 255.0) as u8
        }
        Color::Black => 0,
        Color::White => 255,
        _ => 128,
    }
}

fn render_markdown_lines(text: &str, width: usize, theme: &RenderTheme) -> Vec<Line<'static>> {
    let parser = MdParser::new(text);
    let mut buf = String::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut heading_level: Option<HeadingLevel> = None;
    let mut lines = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if !buf.trim().is_empty() {
                    lines.extend(render_paragraph_lines(buf.trim(), width, theme));
                }
                buf.clear();
            }
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading_level.take() {
                    if !buf.trim().is_empty() {
                        lines.extend(render_heading_lines(buf.trim(), level, width, theme));
                    }
                    buf.clear();
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_buf.clear();
                code_lang.clear();
                if let CodeBlockKind::Fenced(lang) = kind {
                    code_lang = lang.to_string();
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                lines.extend(render_code_block_lines(&code_buf, &code_lang, theme));
                in_code = false;
            }
            Event::Text(t) => {
                if in_code {
                    code_buf.push_str(&t);
                } else {
                    buf.push_str(&t);
                }
            }
            Event::SoftBreak => {
                if in_code {
                    code_buf.push('\n');
                } else {
                    buf.push(' ');
                }
            }
            Event::HardBreak => {
                if in_code {
                    code_buf.push('\n');
                } else {
                    buf.push('\n');
                }
            }
            _ => {}
        }
    }

    if !buf.trim().is_empty() {
        lines.extend(render_paragraph_lines(buf.trim(), width, theme));
    }

    lines
}
