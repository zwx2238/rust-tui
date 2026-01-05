use crate::render::{RenderTheme, render_markdown_lines};
use crate::ui::selection::line_to_string;
use ratatui::text::Text;
use textwrap::Options;

pub(crate) fn build_patch_text(
    preview: &str,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let wrapped = wrap_text(preview, width);
    let md = patch_to_markdown(&wrapped);
    build_text(&md, width, height, scroll, theme)
}

pub(crate) fn patch_max_scroll(
    preview: &str,
    width: u16,
    height: u16,
    theme: &RenderTheme,
) -> usize {
    let wrapped = wrap_text(preview, width);
    let md = patch_to_markdown(&wrapped);
    max_scroll(&md, width, height, theme)
}

pub(crate) fn patch_plain_lines(
    preview: &str,
    width: u16,
    theme: &RenderTheme,
) -> Vec<String> {
    let wrapped = wrap_text(preview, width);
    let md = patch_to_markdown(&wrapped);
    let lines = render_markdown_lines(&md, width.max(1) as usize, theme, false, false);
    lines.into_iter().map(|line| line_to_string(&line)).collect()
}

fn build_text(
    md: &str,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let lines = render_markdown_lines(md, width as usize, theme, false, false);
    let view_height = height.saturating_sub(1) as usize;
    let start = scroll.min(lines.len());
    let end = (start + view_height).min(lines.len());
    let slice = lines[start..end].to_vec();
    (Text::from(slice), lines.len())
}

fn max_scroll(md: &str, width: u16, height: u16, theme: &RenderTheme) -> usize {
    let lines = render_markdown_lines(md, width as usize, theme, false, false);
    let view_height = height.saturating_sub(1) as usize;
    lines.len().saturating_sub(view_height)
}

fn wrap_text(input: &str, width: u16) -> String {
    let width = width.max(1) as usize;
    let options = Options::new(width).break_words(true);
    let mut out = String::new();
    for line in input.lines() {
        if line.is_empty() {
            out.push('\n');
            continue;
        }
        for wrapped in textwrap::wrap(line, &options) {
            out.push_str(&wrapped);
            out.push('\n');
        }
    }
    if !input.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

fn patch_to_markdown(diff: &str) -> String {
    let mut out = String::new();
    out.push_str("```diff\n");
    if diff.trim().is_empty() {
        out.push_str("(空)\n");
    } else {
        out.push_str(diff);
        if !diff.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("```\n");
    out
}
