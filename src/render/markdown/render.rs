use crate::render::markdown::code::render_code_block_lines;
use crate::render::markdown::list::render_list_item_lines;
use crate::render::markdown::shared::{
    ItemContext, ListState, append_text, list_indent, list_prefix, markdown_parser,
};
use crate::render::markdown::table::TableBuild;
use crate::render::markdown::text::{render_heading_lines, render_paragraph_lines};
use crate::render::markdown::preprocess_math;
use crate::render::theme::RenderTheme;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Tag, TagEnd};
use ratatui::text::Line;

pub fn render_markdown_lines(
    text: &str,
    width: usize,
    theme: &RenderTheme,
    streaming: bool,
) -> Vec<Line<'static>> {
    let text = preprocess_math(text);
    let parser = markdown_parser(&text);
    let mut buf = String::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut heading_level: Option<HeadingLevel> = None;
    let mut lines = Vec::new();
    let mut table = TableBuild::default();
    let mut list_stack: Vec<ListState> = Vec::new();
    let mut item_stack: Vec<ItemContext> = Vec::new();
    let mut pending_link: Option<String> = None;
    let mut pending_image: Option<String> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {}
            Event::Start(Tag::List(start)) => {
                if let Some(item) = item_stack.last_mut() {
                    if !item.buf.trim().is_empty() {
                        let prefix = list_prefix(item.ordered, item.index);
                        let indent = list_indent(item.depth);
                        lines.extend(render_list_item_lines(
                            item.buf.trim(),
                            &prefix,
                            &indent,
                            width,
                            theme,
                        ));
                        item.buf.clear();
                    }
                }
                list_stack.push(ListState {
                    ordered: start.is_some(),
                    index: start.unwrap_or(1),
                });
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
            }
            Event::Start(Tag::Item) => {
                if let Some(state) = list_stack.last_mut() {
                    let ordered = state.ordered;
                    let index = state.index;
                    if ordered {
                        state.index = state.index.saturating_add(1);
                    }
                    item_stack.push(ItemContext {
                        buf: String::new(),
                        depth: list_stack.len(),
                        ordered,
                        index,
                    });
                }
            }
            Event::End(TagEnd::Item) => {
                if let Some(item) = item_stack.pop() {
                    if !item.buf.trim().is_empty() {
                        let prefix = list_prefix(item.ordered, item.index);
                        let indent = list_indent(item.depth);
                        lines.extend(render_list_item_lines(
                            item.buf.trim(),
                            &prefix,
                            &indent,
                            width,
                            theme,
                        ));
                    }
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if !buf.trim().is_empty() {
                    lines.extend(render_paragraph_lines(buf.trim(), width, theme));
                }
                buf.clear();
            }
            Event::Start(Tag::Heading { level, .. }) => heading_level = Some(level),
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading_level.take() {
                    if !buf.trim().is_empty() {
                        lines.extend(render_heading_lines(buf.trim(), level, width, theme));
                    }
                    buf.clear();
                }
            }
            Event::Start(Tag::Table(_)) => table.start(streaming),
            Event::End(TagEnd::Table) => lines.extend(table.finish_render(width, theme)),
            Event::Start(Tag::TableHead) => table.start_head(),
            Event::End(TagEnd::TableHead) => table.end_head(),
            Event::Start(Tag::TableRow) => table.start_row(),
            Event::End(TagEnd::TableRow) => table.end_row(),
            Event::Start(Tag::TableCell) => table.start_cell(),
            Event::End(TagEnd::TableCell) => table.end_cell(),
            Event::Start(Tag::Link { dest_url, .. }) => pending_link = Some(dest_url.to_string()),
            Event::End(TagEnd::Link) => {
                if let Some(dest) = pending_link.take() {
                    if !dest.is_empty() {
                        append_text(&mut buf, &mut item_stack, &mut table, &format!(" ({dest})"));
                    }
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => pending_image = Some(dest_url.to_string()),
            Event::End(TagEnd::Image) => {
                if let Some(dest) = pending_image.take() {
                    if !dest.is_empty() {
                        append_text(&mut buf, &mut item_stack, &mut table, &format!(" ({dest})"));
                    }
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
                    append_text(&mut buf, &mut item_stack, &mut table, &t);
                }
            }
            Event::Code(t) | Event::Html(t) => {
                append_text(&mut buf, &mut item_stack, &mut table, &t);
            }
            Event::FootnoteReference(name) => {
                append_text(&mut buf, &mut item_stack, &mut table, &format!("[^{name}]"));
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked { "[x] " } else { "[ ] " };
                append_text(&mut buf, &mut item_stack, &mut table, marker);
            }
            Event::SoftBreak => {
                if in_code {
                    code_buf.push('\n');
                } else if table.in_cell {
                    table.push_text(" ");
                } else if let Some(item) = item_stack.last_mut() {
                    item.buf.push('\n');
                } else {
                    buf.push('\n');
                }
            }
            Event::HardBreak => {
                if in_code {
                    code_buf.push('\n');
                } else if table.in_cell {
                    table.push_text(" ");
                } else if let Some(item) = item_stack.last_mut() {
                    item.buf.push('\n');
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
