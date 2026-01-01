use crate::render::{RenderTheme, count_message_lines, label_for_role};
use crate::types::Message;
use crate::ui::draw::draw_tabs;
use crate::ui::popup_table::{
    TablePopup, draw_table_popup, header_style, popup_row_at, popup_visible_rows,
};
use crate::ui::notice::draw_notice;
use crate::ui::runtime_helpers::TabState;
use crate::ui::text_utils::{collapse_text, truncate_to_width};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use std::io::Stdout;

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
        let label = label_for_role(&msg.role, None);
        if label.is_none() {
            continue;
        }
        let start_line = line_cursor;
        line_cursor += 1;
        let streaming = streaming_idx == Some(idx);
        let content_lines = count_message_lines(msg, width, streaming);
        line_cursor += content_lines;
        line_cursor += 1;
        let preview = truncate_to_width(&collapse_text(&msg.content), max_preview_width);
        rows.push(JumpRow {
            index: idx + 1,
            role: msg.role.clone(),
            preview,
            scroll: start_line.min(u16::MAX as usize) as u16,
        });
    }
    rows
}

pub fn redraw_jump(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    theme: &RenderTheme,
    tabs: &mut [TabState],
    active_tab: usize,
    startup_text: Option<&str>,
    rows: &[JumpRow],
    selected: usize,
    area: Rect,
    tabs_area: Rect,
    scroll: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        draw_tabs(f, tabs_area, tabs.len(), active_tab, theme, startup_text);
        draw_jump_table(f, area, rows, selected, theme, scroll);
        if let Some(tab) = tabs.get_mut(active_tab) {
            draw_notice(f, f.area(), &mut tab.app, theme);
        }
    })?;
    Ok(())
}

pub fn jump_row_at(
    area: Rect,
    row_count: usize,
    mouse_x: u16,
    mouse_y: u16,
    scroll: usize,
) -> Option<usize> {
    popup_row_at(area, row_count, scroll, mouse_x, mouse_y)
}

pub fn jump_visible_rows(area: Rect) -> usize {
    popup_visible_rows(area)
}

pub fn max_preview_width(area: Rect) -> usize {
    let inner_width = area.width.saturating_sub(2) as usize;
    inner_width.saturating_sub(PREVIEW_GUTTER).max(10)
}

fn draw_jump_table(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    rows: &[JumpRow],
    selected: usize,
    theme: &RenderTheme,
    scroll: usize,
) {
    let header = Row::new(vec![
        Cell::from("序号"),
        Cell::from("角色"),
        Cell::from("内容"),
    ])
    .style(header_style(theme));
    let body = rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.index.to_string()),
            Cell::from(row.role.clone()),
            Cell::from(row.preview.clone()),
        ])
    });
    let popup = TablePopup {
        title: Line::from("消息定位 · Enter/点击 跳转 · E 复制用户消息到新 tab · F2 退出"),
        header,
        rows: body.collect(),
        widths: vec![
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Min(10),
        ],
        selected,
        scroll,
        theme,
    };
    draw_table_popup(f, area, popup);
}

// text utilities are centralized in text_utils
