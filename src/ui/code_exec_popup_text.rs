use crate::render::{RenderTheme, render_markdown_lines};
use ratatui::text::Text;
use textwrap::Options;

pub(crate) fn build_code_text(
    code: &str,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let wrapped = wrap_text(code, width);
    let md = code_to_markdown(&wrapped);
    build_text(&md, width, height, scroll, theme)
}

pub(crate) fn code_max_scroll(code: &str, width: u16, height: u16, theme: &RenderTheme) -> usize {
    let wrapped = wrap_text(code, width);
    let md = code_to_markdown(&wrapped);
    max_scroll(&md, width, height, theme)
}

pub(crate) fn build_stdout_text(
    output: Option<&str>,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let stdout = output.unwrap_or("");
    let wrapped = wrap_text(stdout, width);
    let md = stdout_to_markdown(&wrapped);
    build_text(&md, width, height, scroll, theme)
}

pub(crate) fn stdout_max_scroll(
    output: &str,
    width: u16,
    height: u16,
    theme: &RenderTheme,
) -> usize {
    let wrapped = wrap_text(output, width);
    let md = stdout_to_markdown(&wrapped);
    max_scroll(&md, width, height, theme)
}

pub(crate) fn build_stderr_text(
    output: Option<&str>,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let stderr = output.unwrap_or("");
    let wrapped = wrap_text(stderr, width);
    let md = stderr_to_markdown(&wrapped);
    build_text(&md, width, height, scroll, theme)
}

pub(crate) fn stderr_max_scroll(
    output: &str,
    width: u16,
    height: u16,
    theme: &RenderTheme,
) -> usize {
    let wrapped = wrap_text(output, width);
    let md = stderr_to_markdown(&wrapped);
    max_scroll(&md, width, height, theme)
}

fn build_text(
    md: &str,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let lines = render_markdown_lines(md, width as usize, theme, false);
    let view_height = height.saturating_sub(1) as usize;
    let start = scroll.min(lines.len());
    let end = (start + view_height).min(lines.len());
    let slice = lines[start..end].to_vec();
    (Text::from(slice), lines.len())
}

fn max_scroll(md: &str, width: u16, height: u16, theme: &RenderTheme) -> usize {
    let lines = render_markdown_lines(md, width as usize, theme, false);
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

fn code_to_markdown(code: &str) -> String {
    if code.trim().is_empty() {
        "```python\n(空)\n```".to_string()
    } else {
        let mut out = String::from("```python\n");
        out.push_str(code);
        if !code.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```");
        out
    }
}

fn stdout_to_markdown(stdout: &str) -> String {
    let mut out = String::new();
    out.push_str("```text\n");
    if stdout.trim().is_empty() {
        out.push_str("(空)\n");
    } else {
        out.push_str(stdout);
        if !stdout.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("```\n");
    out
}

fn stderr_to_markdown(stderr: &str) -> String {
    let mut out = String::new();
    out.push_str("```text\n");
    if stderr.trim().is_empty() {
        out.push_str("(空)\n");
    } else {
        out.push_str(stderr);
        if !stderr.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("```\n");
    out
}
