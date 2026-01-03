use crate::render::{RenderTheme, count_message_lines, label_for_role};
use crate::types::Message;
use crate::ui::draw::{draw_categories, draw_footer, draw_header, draw_tabs};
use crate::ui::notice::draw_notice;
use crate::ui::overlay_table::{OverlayTable, draw_overlay_table, header_style};
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

fn build_jump_row(
    idx: usize,
    msg: &Message,
    width: usize,
    max_preview_width: usize,
    streaming_idx: Option<usize>,
    line_cursor: &mut usize,
) -> Option<JumpRow> {
    if label_for_role(&msg.role, None).is_none() {
        return None;
    }
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

pub fn redraw_jump(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>, theme: &RenderTheme,
    tabs: &mut [TabState], active_tab: usize, tab_labels: &[String], active_tab_pos: usize,
    categories: &[String], active_category: usize, startup_text: Option<&str>,
    header_note: Option<&str>, rows: &[JumpRow], selected: usize, area: Rect,
    header_area: Rect, category_area: Rect, tabs_area: Rect, footer_area: Rect, scroll: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        draw_jump_layout(
            f,
            theme,
            tab_labels,
            active_tab_pos,
            categories,
            active_category,
            header_note,
            startup_text,
            header_area,
            category_area,
            tabs_area,
            footer_area,
        );
        draw_jump_table(f, area, rows, selected, theme, scroll);
        if let Some(tab) = tabs.get_mut(active_tab) {
            draw_notice(f, f.area(), &mut tab.app, theme);
        }
    })?;
    Ok(())
}

fn draw_jump_layout(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    tab_labels: &[String],
    active_tab_pos: usize,
    categories: &[String],
    active_category: usize,
    header_note: Option<&str>,
    startup_text: Option<&str>,
    header_area: Rect,
    category_area: Rect,
    tabs_area: Rect,
    footer_area: Rect,
) {
    draw_header(f, header_area, theme, header_note);
    draw_categories(f, category_area, categories, active_category, theme);
    draw_tabs(
        f,
        tabs_area,
        tab_labels,
        active_tab_pos,
        theme,
        startup_text,
    );
    draw_footer(f, footer_area, theme, false);
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
    let popup = build_jump_table(rows, selected, theme, scroll);
    draw_overlay_table(f, area, popup);
}

fn build_jump_table<'a>(
    rows: &[JumpRow],
    selected: usize,
    theme: &'a RenderTheme,
    scroll: usize,
) -> OverlayTable<'a> {
    let header = Row::new(vec![Cell::from("序号"), Cell::from("角色"), Cell::from("内容")])
        .style(header_style(theme));
    let body = rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.index.to_string()),
            Cell::from(row.role.clone()),
            Cell::from(row.preview.clone()),
        ])
    });
    OverlayTable {
        title: Line::from("消息定位 · Enter/点击 跳转 · E 复制用户消息到新对话 · F2 退出"),
        header,
        rows: body.collect(),
        widths: vec![Constraint::Length(6), Constraint::Length(10), Constraint::Min(10)],
        selected,
        scroll,
        theme,
    }
}

// text utilities are centralized in text_utils
