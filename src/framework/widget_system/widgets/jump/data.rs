use crate::render::label_for_role;
use crate::types::Message;
use crate::framework::widget_system::interaction::text_utils::{collapse_text, truncate_to_width};
use ratatui::layout::Rect;

pub struct JumpRow {
    pub index: usize,
    pub role: String,
    pub preview: String,
}

const PREVIEW_GUTTER: usize = 20;

pub fn build_jump_rows(
    messages: &[Message],
    max_preview_width: usize,
) -> Vec<JumpRow> {
    let mut rows = Vec::new();
    for (idx, msg) in messages.iter().enumerate() {
        if let Some(row) = build_jump_row(idx, msg, max_preview_width) {
            rows.push(row);
        }
    }
    rows
}

fn build_jump_row(
    idx: usize,
    msg: &Message,
    max_preview_width: usize,
) -> Option<JumpRow> {
    if !is_jump_candidate(msg) {
        return None;
    }
    let preview = truncate_to_width(&collapse_text(&msg.content), max_preview_width);
    Some(JumpRow {
        index: idx + 1,
        role: msg.role.clone(),
        preview,
    })
}

pub fn jump_len(messages: &[Message]) -> usize {
    messages.iter().filter(|msg| is_jump_candidate(msg)).count()
}

pub fn jump_message_index(messages: &[Message], row: usize) -> Option<usize> {
    let mut seen = 0;
    for (idx, msg) in messages.iter().enumerate() {
        if !is_jump_candidate(msg) {
            continue;
        }
        if seen == row {
            return Some(idx);
        }
        seen += 1;
    }
    None
}

pub fn max_preview_width(area: Rect) -> usize {
    let inner_width = area.width.saturating_sub(2) as usize;
    inner_width.saturating_sub(PREVIEW_GUTTER).max(10)
}

fn is_jump_candidate(msg: &Message) -> bool {
    label_for_role(&msg.role, None).is_some()
}
