use crate::render::markdown::code::render_code_block_lines;
use crate::render::markdown::list::{count_list_item_lines, render_list_item_lines};
use crate::render::markdown::table::TableBuild;
use crate::render::markdown::text::{render_heading_lines, render_paragraph_lines};
use crate::render::theme::RenderTheme;
use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, Options, Parser as MdParser, Tag, TagEnd,
};
use ratatui::text::Line;

mod code;
mod list;
mod table;
mod text;

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
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = MdParser::new_ext(text, options);
    let mut buf = String::new();
    let mut list_buf = String::new();
    let mut in_list = false;
    let mut in_item = false;
    let mut code_buf = String::new();
    let mut in_code = false;
    let mut count = 0usize;
    let mut list_index: u64 = 1;
    let mut table = TableBuild::default();

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {}
            Event::Start(Tag::List(start)) => {
                in_list = true;
                list_index = start.unwrap_or(1);
            }
            Event::End(TagEnd::List(_)) => {
                in_list = false;
            }
            Event::Start(Tag::Item) => {
                in_item = true;
                list_buf.clear();
            }
            Event::End(TagEnd::Item) => {
                if !list_buf.trim().is_empty() {
                    let prefix = if in_list {
                        let n = list_index;
                        list_index = list_index.saturating_add(1);
                        format!("{n}. ")
                    } else {
                        "- ".to_string()
                    };
                    count += count_list_item_lines(list_buf.trim(), &prefix, width);
                }
                list_buf.clear();
                in_item = false;
            }
            Event::End(TagEnd::Paragraph) => {
                if !buf.trim().is_empty() {
                    count += textwrap::wrap(buf.trim(), width.max(10)).len();
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
            Event::Start(Tag::Table(_)) => {
                table.start(false);
            }
            Event::End(TagEnd::Table) => {
                count += table.finish_count(width);
            }
            Event::Start(Tag::TableHead) => {
                table.start_head();
            }
            Event::End(TagEnd::TableHead) => {
                table.end_head();
            }
            Event::Start(Tag::TableRow) => {
                table.start_row();
            }
            Event::End(TagEnd::TableRow) => {
                table.end_row();
            }
            Event::Start(Tag::TableCell) => {
                table.start_cell();
            }
            Event::End(TagEnd::TableCell) => {
                table.end_cell();
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
                } else if table.in_cell {
                    table.push_text(&t);
                } else if in_item {
                    list_buf.push_str(&t);
                } else {
                    buf.push_str(&t);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_code {
                    code_buf.push('\n');
                } else if table.in_cell {
                    table.push_text(" ");
                } else if in_item {
                    list_buf.push('\n');
                } else {
                    buf.push('\n');
                }
            }
            _ => {}
        }
    }
    if !buf.trim().is_empty() {
        count += textwrap::wrap(buf.trim(), width.max(10)).len();
    }
    count
}

pub(crate) fn render_markdown_lines(
    text: &str,
    width: usize,
    theme: &RenderTheme,
    streaming: bool,
) -> Vec<Line<'static>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = MdParser::new_ext(text, options);
    let mut buf = String::new();
    let mut list_buf = String::new();
    let mut in_list = false;
    let mut in_item = false;
    let mut list_index: u64 = 1;
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut heading_level: Option<HeadingLevel> = None;
    let mut lines = Vec::new();
    let mut table = TableBuild::default();

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {}
            Event::Start(Tag::List(start)) => {
                in_list = true;
                list_index = start.unwrap_or(1);
            }
            Event::End(TagEnd::List(_)) => {
                in_list = false;
            }
            Event::Start(Tag::Item) => {
                in_item = true;
                list_buf.clear();
            }
            Event::End(TagEnd::Item) => {
                if !list_buf.trim().is_empty() {
                    let prefix = if in_list {
                        let n = list_index;
                        list_index = list_index.saturating_add(1);
                        format!("{n}. ")
                    } else {
                        "- ".to_string()
                    };
                    lines.extend(render_list_item_lines(list_buf.trim(), &prefix, width, theme));
                }
                list_buf.clear();
                in_item = false;
            }
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
            Event::Start(Tag::Table(_)) => {
                table.start(streaming);
            }
            Event::End(TagEnd::Table) => {
                lines.extend(table.finish_render(width, theme));
            }
            Event::Start(Tag::TableHead) => {
                table.start_head();
            }
            Event::End(TagEnd::TableHead) => {
                table.end_head();
            }
            Event::Start(Tag::TableRow) => {
                table.start_row();
            }
            Event::End(TagEnd::TableRow) => {
                table.end_row();
            }
            Event::Start(Tag::TableCell) => {
                table.start_cell();
            }
            Event::End(TagEnd::TableCell) => {
                table.end_cell();
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
                } else if table.in_cell {
                    table.push_text(&t);
                } else if in_item {
                    list_buf.push_str(&t);
                } else {
                    buf.push_str(&t);
                }
            }
            Event::SoftBreak => {
                if in_code {
                    code_buf.push('\n');
                } else if table.in_cell {
                    table.push_text(" ");
                } else if in_item {
                    list_buf.push('\n');
                } else {
                    buf.push('\n');
                }
            }
            Event::HardBreak => {
                if in_code {
                    code_buf.push('\n');
                } else if table.in_cell {
                    table.push_text(" ");
                } else if in_item {
                    list_buf.push('\n');
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
