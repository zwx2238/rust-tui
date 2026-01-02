use crate::render::markdown::list::count_list_item_lines;
use crate::render::markdown::shared::{
    ItemContext, ListState, append_text, list_indent, list_prefix, markdown_parser,
};
use crate::render::markdown::preprocess_math;
use crate::render::markdown::table::TableBuild;
use pulldown_cmark::{Event, Tag, TagEnd};

pub(crate) fn count_markdown_lines(text: &str, width: usize) -> usize {
    let text = preprocess_math(text);
    let parser = markdown_parser(&text);
    let mut buf = String::new();
    let mut code_buf = String::new();
    let mut in_code = false;
    let mut count = 0usize;
    let mut list_stack: Vec<ListState> = Vec::new();
    let mut item_stack: Vec<ItemContext> = Vec::new();
    let mut table = TableBuild::default();
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
                        count += count_list_item_lines(item.buf.trim(), &prefix, &indent, width);
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
                        count += count_list_item_lines(item.buf.trim(), &prefix, &indent, width);
                    }
                }
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
                    count += textwrap::wrap(buf.trim(), width.max(10)).len();
                }
                buf.clear();
            }
            Event::Start(Tag::Table(_)) => table.start(false),
            Event::End(TagEnd::Table) => count += table.finish_count(width),
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
            Event::SoftBreak | Event::HardBreak => {
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
        count += textwrap::wrap(buf.trim(), width.max(10)).len();
    }
    count
}
