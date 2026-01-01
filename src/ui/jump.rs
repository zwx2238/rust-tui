use crate::render::{count_message_lines, label_for_role, RenderTheme};
use crate::ui::draw::draw_tabs;
use crate::ui::popup_table::{draw_table_popup, TablePopup};
use crate::ui::summary::summary_row_at;
use crate::types::Message;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct JumpRow {
    pub index: usize,
    pub role: String,
    pub preview: String,
    pub scroll: u16,
}

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
    tabs_len: usize,
    active_tab: usize,
    startup_text: Option<&str>,
    rows: &[JumpRow],
    selected: usize,
    area: Rect,
    tabs_area: Rect,
    scroll: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        draw_tabs(f, tabs_area, tabs_len, active_tab, theme, startup_text);
        draw_jump_table(f, area, rows, selected, theme, scroll);
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
    summary_row_at(area, row_count, mouse_x, mouse_y).map(|r| r.saturating_add(scroll))
}

pub fn jump_visible_rows(area: Rect) -> usize {
    area.height.saturating_sub(2).saturating_sub(1) as usize
}

pub fn max_preview_width(area: Rect) -> usize {
    let inner_width = area.width.saturating_sub(2) as usize;
    inner_width.saturating_sub(20).max(10)
}

fn draw_jump_table(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    rows: &[JumpRow],
    selected: usize,
    theme: &RenderTheme,
    scroll: usize,
) {
    let header = Row::new(vec![Cell::from("序号"), Cell::from("角色"), Cell::from("内容")])
        .style(
            Style::default()
                .fg(theme.fg.unwrap_or(Color::White))
                .add_modifier(Modifier::BOLD),
        );
    let body = rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.index.to_string()),
            Cell::from(row.role.clone()),
            Cell::from(row.preview.clone()),
        ])
    });
    let popup = TablePopup {
        title: Line::from("消息定位 · F2 退出 · 点击行跳转"),
        header,
        rows: body.collect(),
        widths: vec![Constraint::Length(6), Constraint::Length(10), Constraint::Min(10)],
        selected,
        scroll,
        theme,
    };
    draw_table_popup(f, area, popup);
}

fn collapse_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if text.width() <= max_width {
        return text.to_string();
    }
    let ellipsis = "...";
    let mut out = String::new();
    let mut width = 0usize;
    let limit = max_width.saturating_sub(ellipsis.width());
    for ch in text.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(1);
        if width.saturating_add(w) > limit {
            break;
        }
        out.push(ch);
        width = width.saturating_add(w);
    }
    out.push_str(ellipsis);
    out
}
