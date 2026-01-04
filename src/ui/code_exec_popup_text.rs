use crate::render::{RenderTheme, render_markdown_lines};
use ratatui::text::{Line, Text};
use crate::ui::selection::{line_to_string, line_width};
use unicode_width::UnicodeWidthChar;

pub(crate) fn build_code_text(
    code: &str,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let md = code_to_markdown(code);
    build_text(&md, width, height, scroll, theme)
}

pub(crate) fn code_max_scroll(code: &str, width: u16, height: u16, theme: &RenderTheme) -> usize {
    let md = code_to_markdown(code);
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
    let md = stdout_to_markdown(stdout);
    build_text(&md, width, height, scroll, theme)
}

pub(crate) fn stdout_max_scroll(
    output: &str,
    width: u16,
    height: u16,
    theme: &RenderTheme,
) -> usize {
    let md = stdout_to_markdown(output);
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
    let md = stderr_to_markdown(stderr);
    build_text(&md, width, height, scroll, theme)
}

pub(crate) fn stderr_max_scroll(
    output: &str,
    width: u16,
    height: u16,
    theme: &RenderTheme,
) -> usize {
    let md = stderr_to_markdown(output);
    max_scroll(&md, width, height, theme)
}

pub(crate) fn code_plain_lines(code: &str, width: u16, theme: &RenderTheme) -> Vec<String> {
    let md = code_to_markdown(code);
    render_plain_lines(&md, width, theme)
}

pub(crate) fn stdout_plain_lines(output: &str, width: u16, theme: &RenderTheme) -> Vec<String> {
    let md = stdout_to_markdown(output);
    render_plain_lines(&md, width, theme)
}

pub(crate) fn stderr_plain_lines(output: &str, width: u16, theme: &RenderTheme) -> Vec<String> {
    let md = stderr_to_markdown(output);
    render_plain_lines(&md, width, theme)
}

fn build_text(
    md: &str,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let lines =
        wrap_rendered_lines(render_markdown_lines(md, width as usize, theme, false, true), width);
    let view_height = height.saturating_sub(1) as usize;
    let start = scroll.min(lines.len());
    let end = (start + view_height).min(lines.len());
    let slice = lines[start..end].to_vec();
    (Text::from(slice), lines.len())
}

fn max_scroll(md: &str, width: u16, height: u16, theme: &RenderTheme) -> usize {
    let lines =
        wrap_rendered_lines(render_markdown_lines(md, width as usize, theme, false, true), width);
    let view_height = height.saturating_sub(1) as usize;
    lines.len().saturating_sub(view_height)
}

fn render_plain_lines(md: &str, width: u16, theme: &RenderTheme) -> Vec<String> {
    let lines: Vec<Line<'static>> =
        wrap_rendered_lines(render_markdown_lines(md, width as usize, theme, false, true), width);
    lines.into_iter().map(|line| line_to_string(&line)).collect()
}

fn wrap_rendered_lines(lines: Vec<Line<'static>>, width: u16) -> Vec<Line<'static>> {
    let width = width.max(1) as usize;
    let mut out = Vec::new();
    for line in lines {
        if line_width(&line) <= width {
            out.push(line);
            continue;
        }
        let mut text = line_to_string(&line);
        text = text.trim_end_matches(['\n', '\r']).to_string();
        if text.is_empty() {
            out.push(Line::from(""));
            continue;
        }
        if let Some((prefix, cont_prefix, content)) = split_numbered_prefix(&text) {
            for wrapped in wrap_with_prefix(&content, &prefix, &cont_prefix, width) {
                out.push(Line::from(wrapped));
            }
        } else {
            for wrapped in wrap_fixed_width(&text, width) {
                out.push(Line::from(wrapped));
            }
        }
    }
    out
}

fn split_numbered_prefix(text: &str) -> Option<(String, String, String)> {
    let pipe = text.find(" | ")?;
    let (left, rest) = text.split_at(pipe);
    let left_trim = left.trim();
    if left_trim.is_empty() || !left_trim.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let prefix = format!("{} | ", left_trim);
    let cont_prefix = format!("{} | ", " ".repeat(left_trim.len()));
    let content = rest.trim_start_matches(" | ").to_string();
    Some((prefix, cont_prefix, content))
}

fn wrap_with_prefix(content: &str, prefix: &str, cont_prefix: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let prefix_width = prefix.chars().map(char_width).sum::<usize>().max(1);
    if width <= prefix_width + 1 {
        for wrapped in wrap_fixed_width(&format!("{prefix}{content}"), width) {
            out.push(wrapped);
        }
        return out;
    }
    let avail = width.saturating_sub(prefix_width);
    let parts = wrap_fixed_width(content, avail);
    for (i, part) in parts.into_iter().enumerate() {
        if i == 0 {
            out.push(format!("{prefix}{part}"));
        } else {
            out.push(format!("{cont_prefix}{part}"));
        }
    }
    out
}

fn wrap_fixed_width(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut col = 0usize;
    for ch in text.chars() {
        let w = char_width(ch);
        if col + w > width && !buf.is_empty() {
            out.push(std::mem::take(&mut buf));
            col = 0;
        }
        buf.push(ch);
        col += w;
        if col >= width {
            out.push(std::mem::take(&mut buf));
            col = 0;
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

fn char_width(ch: char) -> usize {
    UnicodeWidthChar::width(ch).unwrap_or(0).max(1)
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
