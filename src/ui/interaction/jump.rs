use crate::render::MessageLayout;
use crate::render::{count_message_lines, label_for_role};
use crate::types::Message;
use crate::ui::text_utils::{collapse_text, truncate_to_width};
use ratatui::layout::Rect;

pub struct JumpRow {
    pub index: usize,
    pub role: String,
    pub preview: String,
    pub scroll: u16,
}

const PREVIEW_GUTTER: usize = 20;

pub fn build_jump_rows(
    messages: &[Message],
    width: usize,
    max_preview_width: usize,
    streaming_idx: Option<usize>,
) -> Vec<JumpRow> {
    let mut rows = Vec::new();
    let mut line_cursor = 0usize;
    for (idx, msg) in messages.iter().enumerate() {
        if let Some(row) = build_jump_row(
            idx,
            msg,
            width,
            max_preview_width,
            streaming_idx,
            &mut line_cursor,
        ) {
            rows.push(row);
        }
    }
    rows
}

pub fn build_jump_rows_from_layouts(
    messages: &[Message],
    layouts: &[MessageLayout],
) -> Vec<JumpRow> {
    let mut rows = Vec::with_capacity(layouts.len());
    for layout in layouts {
        let Some(msg) = messages.get(layout.index) else {
            continue;
        };
        rows.push(JumpRow {
            index: layout.index + 1,
            role: msg.role.clone(),
            preview: String::new(),
            scroll: layout.label_line.min(u16::MAX as usize) as u16,
        });
    }
    rows
}

fn build_jump_row(
    idx: usize,
    msg: &Message,
    width: usize,
    max_preview_width: usize,
    streaming_idx: Option<usize>,
    line_cursor: &mut usize,
) -> Option<JumpRow> {
    label_for_role(&msg.role, None)?;
    let start_line = *line_cursor;
    *line_cursor += 1;
    let streaming = streaming_idx == Some(idx);
    let content_lines = count_message_lines(msg, width, streaming);
    *line_cursor += content_lines + 1;
    let preview = truncate_to_width(&collapse_text(&msg.content), max_preview_width);
    Some(JumpRow {
        index: idx + 1,
        role: msg.role.clone(),
        preview,
        scroll: start_line.min(u16::MAX as usize) as u16,
    })
}

pub fn max_preview_width(area: Rect) -> usize {
    let inner_width = area.width.saturating_sub(2) as usize;
    inner_width.saturating_sub(PREVIEW_GUTTER).max(10)
}
