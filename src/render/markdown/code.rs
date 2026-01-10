use crate::render::theme::RenderTheme;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub(crate) fn render_code_block_lines(
    text: &str,
    lang: &str,
    theme: &RenderTheme,
    show_line_numbers: bool,
) -> Vec<Line<'static>> {
    let (ss, syn_theme, syntax) = load_syntax(theme, lang);
    let mut highlighter = HighlightLines::new(syntax, syn_theme);
    let lines: Vec<&str> = text.lines().collect();
    let config = build_highlight_config(theme, lang, lines.len(), show_line_numbers);
    let mut out = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let spans = highlight_spans(raw, &mut highlighter, ss, &config, i);
        out.push(Line::from(spans));
    }
    out
}

fn load_syntax(
    theme: &RenderTheme,
    lang: &str,
) -> (&'static SyntaxSet, &'static Theme, &'static SyntaxReference) {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
    let ss = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
    let ts = THEME_SET.get_or_init(ThemeSet::load_defaults);
    let syn_theme = ts
        .themes
        .get(theme.code_theme)
        .unwrap_or_else(|| ts.themes.values().next().expect("theme set is empty"));
    let syntax = ss
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    (ss, syn_theme, syntax)
}

struct HighlightConfig {
    show_line_numbers: bool,
    max_digits: usize,
    code_fg: Color,
    code_bg: Color,
    bg_luma: u8,
}

fn highlight_spans(
    raw: &str,
    highlighter: &mut HighlightLines<'_>,
    ss: &SyntaxSet,
    config: &HighlightConfig,
    line_idx: usize,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    add_line_number(&mut spans, config, line_idx);
    append_highlighted_spans(
        &mut spans,
        highlight_line_ranges(raw, highlighter, ss),
        config,
    );
    spans
}

fn highlight_line_ranges(
    raw: &str,
    highlighter: &mut HighlightLines<'_>,
    ss: &SyntaxSet,
) -> Vec<(syntect::highlighting::Style, String)> {
    let mut line_with_nl = String::with_capacity(raw.len() + 1);
    line_with_nl.push_str(raw);
    line_with_nl.push('\n');
    highlighter
        .highlight_line(&line_with_nl, ss)
        .unwrap_or_default()
        .into_iter()
        .map(|(style, part)| (style, part.to_string()))
        .collect()
}

fn add_line_number(spans: &mut Vec<Span<'static>>, config: &HighlightConfig, line_idx: usize) {
    if !config.show_line_numbers {
        return;
    }
    spans.push(line_no_span(
        line_idx + 1,
        config.max_digits,
        config.code_fg,
        config.code_bg,
    ));
}

fn append_highlighted_spans(
    spans: &mut Vec<Span<'static>>,
    ranges: Vec<(syntect::highlighting::Style, String)>,
    config: &HighlightConfig,
) {
    for (style, part) in ranges {
        let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
        let span_fg = if (color_luma(fg) as i16 - config.bg_luma as i16).abs() < 80 {
            config.code_fg
        } else {
            fg
        };
        spans.push(Span::styled(
            part,
            Style::default().fg(span_fg).bg(config.code_bg),
        ));
    }
}

fn build_highlight_config(
    theme: &RenderTheme,
    lang: &str,
    lines_len: usize,
    show_numbers: bool,
) -> HighlightConfig {
    let show_line_numbers = show_numbers && lang != "math";
    let max_digits = if show_line_numbers {
        lines_len.max(1).to_string().len()
    } else {
        0
    };
    let code_fg = theme.fg.unwrap_or(Color::White);
    let code_bg = theme.bg;
    let bg_luma = color_luma(code_bg);
    HighlightConfig {
        show_line_numbers,
        max_digits,
        code_fg,
        code_bg,
        bg_luma,
    }
}

fn line_no_span(line_no: usize, width: usize, code_fg: Color, code_bg: Color) -> Span<'static> {
    let label = format!("{:>width$} | ", line_no, width = width);
    Span::styled(label, Style::default().fg(code_fg).bg(code_bg))
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
