use crate::config::Config;
use crate::types::Message;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser as MdParser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use textwrap::wrap;

pub struct RenderTheme {
    pub bg: Color,
    pub fg: Option<Color>,
    pub code_bg: Color,
    pub code_theme: &'static str,
    pub heading_fg: Option<Color>,
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
) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    for (idx, msg) in messages.iter().enumerate() {
        let suffix = suffix_for_index(label_suffixes, idx);
        let mut msg_lines = render_message_lines(msg, width, theme, suffix);
        lines.append(&mut msg_lines);
        lines.push(Line::from(""));
    }
    Text::from(lines)
}

pub fn messages_to_plain_lines(
    messages: &[Message],
    width: usize,
    theme: &RenderTheme,
) -> Vec<String> {
    let mut out = Vec::new();
    let text = messages_to_text(messages, width, theme, &[]);
    for line in text.lines {
        let mut s = String::new();
        for span in line.spans {
            s.push_str(&span.content);
        }
        out.push(s);
    }
    out
}

fn render_message_lines(
    msg: &Message,
    width: usize,
    theme: &RenderTheme,
    label_suffix: Option<&str>,
) -> Vec<Line<'static>> {
    match msg.role.as_str() {
        "user" => {
            let mut lines = vec![label_line("👤", theme)];
            lines.extend(render_markdown_lines(
                &msg.content,
                width,
                theme,
            ));
            lines
        }
        "assistant" => {
            let mut label = "🤖".to_string();
            if let Some(suffix) = label_suffix {
                if !suffix.is_empty() {
                    label.push(' ');
                    label.push_str(suffix);
                }
            }
            let mut lines = vec![label_line(&label, theme)];
            lines.extend(render_markdown_lines(
                &msg.content,
                width,
                theme,
            ));
            lines
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
    for (i, raw) in lines.iter().enumerate() {
        let ranges = highlighter
            .highlight_line(raw, &ss)
            .unwrap_or_default();
        let line_no = format!("{:>width$} | ", i + 1, width = max_digits);
        let mut spans = Vec::new();
        spans.push(Span::styled(
            line_no,
            Style::default().fg(Color::DarkGray),
        ));
        for (style, part) in ranges {
            let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
            let span_style = Style::default().fg(fg).bg(theme.code_bg);
            spans.push(Span::styled(part.to_string(), span_style));
        }
        out.push(Line::from(spans));
    }
    out
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
