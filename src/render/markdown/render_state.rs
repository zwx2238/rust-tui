use crate::render::markdown::code::render_code_block_lines;
use crate::render::markdown::list::render_list_item_lines;
use crate::render::markdown::shared::{
    ItemContext, ListState, append_text, list_indent, list_prefix,
};
use crate::render::markdown::table::TableBuild;
use crate::render::markdown::text::{render_heading_lines, render_paragraph_lines};
use crate::render::theme::RenderTheme;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Tag, TagEnd};
use ratatui::text::Line;

pub(super) struct RenderState<'a> {
    width: usize,
    theme: &'a RenderTheme,
    streaming: bool,
    show_code_line_numbers: bool,
    buf: String,
    in_code: bool,
    code_lang: String,
    code_buf: String,
    heading_level: Option<HeadingLevel>,
    lines: Vec<Line<'static>>,
    table: TableBuild,
    list_stack: Vec<ListState>,
    item_stack: Vec<ItemContext>,
    pending_link: Option<String>,
    pending_image: Option<String>,
}

impl<'a> RenderState<'a> {
    pub(super) fn new(
        width: usize,
        theme: &'a RenderTheme,
        streaming: bool,
        show_code_line_numbers: bool,
    ) -> Self {
        Self {
            width,
            theme,
            streaming,
            show_code_line_numbers,
            buf: String::new(),
            in_code: false,
            code_lang: String::new(),
            code_buf: String::new(),
            heading_level: None,
            lines: Vec::new(),
            table: TableBuild::default(),
            list_stack: Vec::new(),
            item_stack: Vec::new(),
            pending_link: None,
            pending_image: None,
        }
    }

    pub(super) fn finish(&mut self) {
        if !self.buf.trim().is_empty() {
            let lines = render_paragraph_lines(self.buf.trim(), self.width, self.theme);
            self.lines.extend(lines);
        }
    }

    pub(super) fn into_lines(self) -> Vec<Line<'static>> {
        self.lines
    }

    pub(super) fn handle_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag),
            Event::End(tag) => self.handle_end_tag(tag),
            Event::Text(t) => self.handle_text(t),
            Event::Code(t) | Event::Html(t) => self.append_inline(&t),
            Event::FootnoteReference(name) => self.append_inline(&format!("[^{name}]")),
            Event::TaskListMarker(checked) => self.handle_task_marker(checked),
            Event::SoftBreak => self.handle_break(),
            Event::HardBreak => self.handle_break(),
            _ => {}
        }
    }

    fn handle_start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {}
            Tag::List(start) => self.start_list(start),
            Tag::Item => self.start_item(),
            Tag::Heading { level, .. } => self.heading_level = Some(level),
            Tag::Table(_) => self.table.start(self.streaming),
            Tag::TableHead => self.table.start_head(),
            Tag::TableRow => self.table.start_row(),
            Tag::TableCell => self.table.start_cell(),
            Tag::Link { dest_url, .. } => self.pending_link = Some(dest_url.to_string()),
            Tag::Image { dest_url, .. } => self.pending_image = Some(dest_url.to_string()),
            Tag::CodeBlock(kind) => self.start_code_block(kind),
            _ => {}
        }
    }

    fn handle_end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::List(_) => {
                self.list_stack.pop();
            }
            TagEnd::Item => self.end_item(),
            TagEnd::Paragraph => self.flush_paragraph(),
            TagEnd::Heading(_) => self.flush_heading(),
            TagEnd::Table => self.finish_table(),
            TagEnd::TableHead => self.table.end_head(),
            TagEnd::TableRow => self.table.end_row(),
            TagEnd::TableCell => self.table.end_cell(),
            TagEnd::Link => self.end_link(),
            TagEnd::Image => self.end_image(),
            TagEnd::CodeBlock => self.end_code_block(),
            _ => {}
        }
    }

    fn start_list(&mut self, start: Option<u64>) {
        if let Some(item) = self.item_stack.last_mut()
            && !item.buf.trim().is_empty()
        {
            let prefix = list_prefix(item.ordered, item.index);
            let indent = list_indent(item.depth);
            let lines =
                render_list_item_lines(item.buf.trim(), &prefix, &indent, self.width, self.theme);
            self.lines.extend(lines);
            item.buf.clear();
        }
        self.list_stack.push(ListState {
            ordered: start.is_some(),
            index: start.unwrap_or(1),
        });
    }

    fn start_item(&mut self) {
        if let Some(state) = self.list_stack.last_mut() {
            let ordered = state.ordered;
            let index = state.index;
            if ordered {
                state.index = state.index.saturating_add(1);
            }
            self.item_stack.push(ItemContext {
                buf: String::new(),
                depth: self.list_stack.len(),
                ordered,
                index,
            });
        }
    }

    fn end_item(&mut self) {
        if let Some(item) = self.item_stack.pop()
            && !item.buf.trim().is_empty()
        {
            let prefix = list_prefix(item.ordered, item.index);
            let indent = list_indent(item.depth);
            let lines =
                render_list_item_lines(item.buf.trim(), &prefix, &indent, self.width, self.theme);
            self.lines.extend(lines);
        }
    }

    fn flush_paragraph(&mut self) {
        if !self.buf.trim().is_empty() {
            let lines = render_paragraph_lines(self.buf.trim(), self.width, self.theme);
            self.lines.extend(lines);
        }
        self.buf.clear();
    }

    fn flush_heading(&mut self) {
        if let Some(level) = self.heading_level.take() {
            if !self.buf.trim().is_empty() {
                let lines = render_heading_lines(self.buf.trim(), level, self.width, self.theme);
                self.lines.extend(lines);
            }
            self.buf.clear();
        }
    }

    fn finish_table(&mut self) {
        let lines = self.table.finish_render(self.width, self.theme);
        self.lines.extend(lines);
    }

    fn start_code_block(&mut self, kind: CodeBlockKind) {
        self.in_code = true;
        self.code_buf.clear();
        self.code_lang.clear();
        if let CodeBlockKind::Fenced(lang) = kind {
            self.code_lang = lang.to_string();
        }
    }

    fn end_code_block(&mut self) {
        self.in_code = false;
        let lines = render_code_block_lines(
            &self.code_buf,
            self.code_lang.trim(),
            self.theme,
            self.show_code_line_numbers,
        );
        self.lines.extend(lines);
        self.code_buf.clear();
        self.code_lang.clear();
    }

    fn handle_text(&mut self, t: pulldown_cmark::CowStr<'_>) {
        if self.in_code {
            self.code_buf.push_str(&t);
            return;
        }
        append_text(&mut self.buf, &mut self.item_stack, &mut self.table, &t);
    }

    fn append_inline(&mut self, text: &str) {
        if self.in_code {
            self.code_buf.push_str(text);
            return;
        }
        append_text(&mut self.buf, &mut self.item_stack, &mut self.table, text);
    }

    fn end_link(&mut self) {
        if let Some(link) = self.pending_link.take() {
            self.append_inline(&format!(" ({link})"));
        }
    }

    fn end_image(&mut self) {
        if let Some(link) = self.pending_image.take() {
            self.append_inline(&format!(" ({link})"));
        }
    }

    fn handle_task_marker(&mut self, checked: bool) {
        let marker = if checked { "[x] " } else { "[ ] " };
        self.append_inline(marker);
    }

    fn handle_break(&mut self) {
        if self.in_code {
            self.code_buf.push('\n');
            return;
        }
        if let Some(item) = self.item_stack.last_mut() {
            item.buf.push('\n');
        } else {
            self.buf.push('\n');
        }
    }
}
