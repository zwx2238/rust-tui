use crate::render::theme::RenderTheme;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser as MdParser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use textwrap::wrap;

pub(crate) fn close_unbalanced_code_fence(input: &str) -> String {
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

pub(crate) fn count_markdown_lines(text: &str, width: usize) -> usize {
    let parser = MdParser::new(text);
    let mut buf = String::new();
    let mut code_buf = String::new();
    let mut in_code = false;
    let mut count = 0usize;

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if !buf.trim().is_empty() {
                    count += wrap(buf.trim(), width.max(10)).len();
                }
                buf.clear();
            }
            Event::Start(Tag::Heading { .. }) => {}
            Event::End(TagEnd::Heading(_)) => {
                if !buf.trim().is_empty() {
                    count += 3;
                }
                buf.clear();
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code = true;
                code_buf.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code = false;
                count += code_buf.lines().count();
                code_buf.clear();
            }
            Event::Text(t) => {
                if in_code {
                    code_buf.push_str(&t);
                } else {
                    buf.push_str(&t);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
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
        count += wrap(buf.trim(), width.max(10)).len();
    }
    count
}

pub(crate) fn render_markdown_lines(
    text: &str,
    width: usize,
    theme: &RenderTheme,
) -> Vec<Line<'static>> {
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
                    buf.push('\n');
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

fn render_paragraph_lines(text: &str, width: usize, theme: &RenderTheme) -> Vec<Line<'static>> {
    let style = Style::default().fg(theme.fg.unwrap_or(Color::White));
    wrap(text, width.max(10))
        .into_iter()
        .map(|line| Line::from(Span::styled(line.to_string(), style)))
        .collect()
}

fn render_heading_lines(
    text: &str,
    level: HeadingLevel,
    width: usize,
    theme: &RenderTheme,
) -> Vec<Line<'static>> {
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

fn render_code_block_lines(text: &str, lang: &str, theme: &RenderTheme) -> Vec<Line<'static>> {
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
