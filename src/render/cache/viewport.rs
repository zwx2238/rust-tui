use super::{
    RenderCacheEntry, ensure_cache_entry, render_message_content_lines, update_cache_entry,
};
use crate::render::theme::RenderTheme;
use crate::render::util::{label_for_role, ranges_overlap, suffix_for_index};
use crate::render::{MessageLayout, label_line_layout, label_line_with_button};
use crate::types::Message;
use ratatui::text::{Line, Text};

pub(super) struct ViewportState<'a> {
    width: usize,
    theme: &'a RenderTheme,
    theme_key: u64,
    label_suffixes: &'a [(usize, String)],
    streaming_idx: Option<usize>,
    start: usize,
    end: usize,
    out: Vec<Line<'static>>,
    layouts: Vec<MessageLayout>,
    line_cursor: usize,
}

impl<'a> ViewportState<'a> {
    pub(super) fn new(
        width: usize,
        theme: &'a RenderTheme,
        theme_key: u64,
        label_suffixes: &'a [(usize, String)],
        streaming_idx: Option<usize>,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            width,
            theme,
            theme_key,
            label_suffixes,
            streaming_idx,
            start,
            end,
            out: Vec::new(),
            layouts: Vec::new(),
            line_cursor: 0,
        }
    }

    pub(super) fn finish(self) -> (Text<'static>, usize, Vec<MessageLayout>) {
        (Text::from(self.out), self.line_cursor, self.layouts)
    }

    pub(super) fn process_message(
        &mut self,
        idx: usize,
        msg: &Message,
        cache: &mut Vec<RenderCacheEntry>,
    ) {
        let entry = ensure_cache_entry(cache, idx, self.theme_key);
        let suffix = suffix_for_index(self.label_suffixes, idx);
        let streaming = self.streaming_idx == Some(idx);
        update_cache_entry(entry, msg, self.width, self.theme_key, streaming);
        if let Some(label) = label_for_role(&msg.role, suffix) {
            self.push_label(idx, msg, &label);
            self.maybe_render_entry(entry, msg, streaming);
            self.push_content_lines(entry);
            self.push_spacing();
        }
    }

    fn push_label(&mut self, idx: usize, msg: &Message, label: &str) {
        let (button_range, label_line) = label_line_layout(&msg.role, label, self.line_cursor);
        self.layouts.push(MessageLayout {
            index: idx,
            label_line,
            button_range,
        });
        if self.line_cursor >= self.start && self.line_cursor < self.end {
            self.out
                .push(label_line_with_button(&msg.role, label, self.theme));
        }
        self.line_cursor += 1;
    }

    fn maybe_render_entry(&mut self, entry: &mut RenderCacheEntry, msg: &Message, streaming: bool) {
        if entry.rendered {
            return;
        }
        let within = ranges_overlap(
            self.start,
            self.end,
            self.line_cursor,
            self.line_cursor + entry.line_count,
        );
        if within {
            entry.lines = render_message_content_lines(msg, self.width, self.theme, streaming);
            entry.rendered = true;
            entry.line_count = entry.lines.len();
        }
    }

    fn push_content_lines(&mut self, entry: &RenderCacheEntry) {
        let content_len = entry.line_count;
        if content_len == 0 {
            return;
        }
        if self.line_cursor + content_len <= self.start || self.line_cursor >= self.end {
            self.line_cursor += content_len;
            return;
        }
        if entry.rendered {
            for line in &entry.lines {
                if self.line_cursor >= self.start && self.line_cursor < self.end {
                    self.out.push(line.clone());
                }
                self.line_cursor += 1;
            }
        } else {
            self.line_cursor += content_len;
        }
    }

    fn push_spacing(&mut self) {
        if self.line_cursor >= self.start && self.line_cursor < self.end {
            self.out.push(Line::from(""));
        }
        self.line_cursor += 1;
    }
}
