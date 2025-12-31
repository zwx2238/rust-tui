use crate::render::theme::RenderTheme;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub(crate) fn render_code_block_lines(text: &str, lang: &str, theme: &RenderTheme) -> Vec<Line<'static>> {
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
