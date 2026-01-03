use crate::render::markdown::list::count_list_item_lines;
use crate::render::markdown::preprocess_math;
use crate::render::markdown::shared::{
    ItemContext, ListState, append_text, list_indent, list_prefix, markdown_parser,
};
use crate::render::markdown::table::TableBuild;
use pulldown_cmark::{Event, Tag, TagEnd};

pub(crate) fn count_markdown_lines(text: &str, width: usize) -> usize {
    let mut state = CountState::new();
    for event in markdown_parser(&preprocess_math(text)) {
        state.handle_event(event, width);
    }
    state.finish(width);
    state.count
}

struct CountState {
    buf: String,
    code_buf: String,
    in_code: bool,
    count: usize,
    list_stack: Vec<ListState>,
    item_stack: Vec<ItemContext>,
    table: TableBuild,
    pending_link: Option<String>,
    pending_image: Option<String>,
}

impl CountState {
    fn new() -> Self {
        Self {
            buf: String::new(),
            code_buf: String::new(),
            in_code: false,
            count: 0,
            list_stack: Vec::new(),
            item_stack: Vec::new(),
            table: TableBuild::default(),
            pending_link: None,
            pending_image: None,
        }
    }

    fn handle_event(&mut self, event: Event, width: usize) {
        match event {
            Event::Start(tag) => self.handle_start(tag, width),
            Event::End(tag) => self.handle_end(tag, width),
            Event::Text(t) => self.handle_text(t),
            Event::Code(t) | Event::Html(t) => append_text(&mut self.buf, &mut self.item_stack, &mut self.table, &t),
            Event::FootnoteReference(name) => append_text(&mut self.buf, &mut self.item_stack, &mut self.table, &format!("[^{name}]")),
            Event::TaskListMarker(checked) => self.handle_task_marker(checked),
            Event::SoftBreak | Event::HardBreak => self.handle_break(),
            _ => {}
        }
    }

    fn handle_start(&mut self, tag: Tag, width: usize) {
        match tag {
            Tag::Paragraph | Tag::Heading { .. } => {}
            Tag::List(start) => self.start_list(start, width),
            Tag::Item => self.start_item(),
            Tag::Table(_) => self.table.start(false),
            Tag::TableHead => self.table.start_head(),
            Tag::TableRow => self.table.start_row(),
            Tag::TableCell => self.table.start_cell(),
            Tag::Link { dest_url, .. } => self.pending_link = Some(dest_url.to_string()),
            Tag::Image { dest_url, .. } => self.pending_image = Some(dest_url.to_string()),
            Tag::CodeBlock(_) => self.start_code_block(),
            _ => {}
        }
    }

    fn handle_end(&mut self, tag: TagEnd, width: usize) {
        match tag {
            TagEnd::List(_) => { self.list_stack.pop(); }
            TagEnd::Item => self.end_item(width),
            TagEnd::Paragraph | TagEnd::Heading(_) => self.flush_paragraph(width),
            TagEnd::Table => self.count += self.table.finish_count(width),
            TagEnd::TableHead => self.table.end_head(),
            TagEnd::TableRow => self.table.end_row(),
            TagEnd::TableCell => self.table.end_cell(),
            TagEnd::Link => {
                let pending = self.pending_link.take();
                self.append_pending(pending);
            }
            TagEnd::Image => {
                let pending = self.pending_image.take();
                self.append_pending(pending);
            }
            TagEnd::CodeBlock => self.end_code_block(),
            _ => {}
        }
    }

    fn handle_text(&mut self, text: pulldown_cmark::CowStr<'_>) {
        if self.in_code { self.code_buf.push_str(&text); }
        else { append_text(&mut self.buf, &mut self.item_stack, &mut self.table, &text); }
    }

    fn handle_task_marker(&mut self, checked: bool) {
        let marker = if checked { "[x] " } else { "[ ] " };
        append_text(&mut self.buf, &mut self.item_stack, &mut self.table, marker);
    }

    fn handle_break(&mut self) {
        if self.in_code {
            self.code_buf.push('\n');
        } else if self.table.in_cell {
            self.table.push_text(" ");
        } else if let Some(item) = self.item_stack.last_mut() {
            item.buf.push('\n');
        } else {
            self.buf.push('\n');
        }
    }

    fn start_list(&mut self, start: Option<u64>, width: usize) {
        if let Some(item) = self.item_stack.last_mut() {
            if !item.buf.trim().is_empty() {
                let prefix = list_prefix(item.ordered, item.index);
                let indent = list_indent(item.depth);
                self.count += count_list_item_lines(item.buf.trim(), &prefix, &indent, width);
                item.buf.clear();
            }
        }
        self.list_stack.push(ListState { ordered: start.is_some(), index: start.unwrap_or(1) });
    }

    fn start_item(&mut self) {
        if let Some(state) = self.list_stack.last_mut() {
            let ordered = state.ordered; let index = state.index;
            if ordered { state.index = state.index.saturating_add(1); }
            self.item_stack.push(ItemContext { buf: String::new(), depth: self.list_stack.len(), ordered, index });
        }
    }

    fn end_item(&mut self, width: usize) {
        if let Some(item) = self.item_stack.pop() {
            if !item.buf.trim().is_empty() {
                let prefix = list_prefix(item.ordered, item.index);
                let indent = list_indent(item.depth);
                self.count += count_list_item_lines(item.buf.trim(), &prefix, &indent, width);
            }
        }
    }

    fn flush_paragraph(&mut self, width: usize) {
        if !self.buf.trim().is_empty() {
            self.count += textwrap::wrap(self.buf.trim(), width.max(10)).len();
        }
        self.buf.clear();
    }

    fn start_code_block(&mut self) {
        self.in_code = true;
        self.code_buf.clear();
    }

    fn end_code_block(&mut self) {
        self.in_code = false;
        self.count += self.code_buf.lines().count();
        self.code_buf.clear();
    }

    fn append_pending(&mut self, pending: Option<String>) {
        let Some(dest) = pending else {
            return;
        };
        if !dest.is_empty() {
            append_text(
                &mut self.buf,
                &mut self.item_stack,
                &mut self.table,
                &format!(" ({dest})"),
            );
        }
    }

    fn finish(&mut self, width: usize) {
        if !self.buf.trim().is_empty() {
            self.count += textwrap::wrap(self.buf.trim(), width.max(10)).len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_mixed_markdown() {
        let input = r#"
# Title

para text with [link](http://example.com) and ![img](http://img)

- item one
- item two

| A | B |
|---|---|
| 1 | 2 |

```rust
fn main() {}
```
"#;
        let count = count_markdown_lines(input, 20);
        assert!(count >= 6);
    }
}
