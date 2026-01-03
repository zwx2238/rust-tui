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
) -> Vec<Line<'static>> {
    let (ss, syn_theme, syntax) = load_syntax(theme, lang);
    let mut highlighter = HighlightLines::new(syntax, syn_theme);
    let lines: Vec<&str> = text.lines().collect();
    let show_line_numbers = lang != "math";
    let max_digits = if show_line_numbers {
        lines.len().max(1).to_string().len()
    } else {
        0
    };
    let code_fg = theme.fg.unwrap_or(Color::White);
    let code_bg = theme.bg;
    let bg_luma = color_luma(code_bg);
    let mut out = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let config = HighlightConfig {
            show_line_numbers,
            max_digits,
            line_idx: i,
            code_fg,
            code_bg,
            bg_luma,
        };
        let spans = highlight_spans(raw, &mut highlighter, ss, config);
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
    line_idx: usize,
    code_fg: Color,
    code_bg: Color,
    bg_luma: u8,
}

fn highlight_spans(
    raw: &str,
    highlighter: &mut HighlightLines<'_>,
    ss: &SyntaxSet,
    config: HighlightConfig,
) -> Vec<Span<'static>> {
    let mut line_with_nl = String::with_capacity(raw.len() + 1);
    line_with_nl.push_str(raw);
    line_with_nl.push('\n');
    let ranges = highlighter
        .highlight_line(&line_with_nl, ss)
        .unwrap_or_default();
    let mut spans = Vec::new();
    if config.show_line_numbers {
        spans.push(line_no_span(
            config.line_idx + 1,
            config.max_digits,
            config.code_fg,
            config.code_bg,
        ));
    }
    for (style, part) in ranges {
        let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
        let span_fg = if (color_luma(fg) as i16 - config.bg_luma as i16).abs() < 80 {
            config.code_fg
        } else {
            fg
        };
        spans.push(Span::styled(
            part.to_string(),
            Style::default().fg(span_fg).bg(config.code_bg),
        ));
    }
    spans
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::theme::RenderTheme;
    use ratatui::style::Color;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    #[test]
    fn renders_line_numbers_for_non_math() {
        let lines = render_code_block_lines("let x = 1;\nlet y = 2;", "rust", &theme());
        assert_eq!(lines.len(), 2);
        let first = lines[0].to_string();
        assert!(first.contains("|"));
    }

    #[test]
    fn omits_line_numbers_for_math() {
        let lines = render_code_block_lines("x^2 + 1", "math", &theme());
        assert_eq!(lines.len(), 1);
        let line = lines[0].to_string();
        assert!(!line.contains("|"));
    }
}
