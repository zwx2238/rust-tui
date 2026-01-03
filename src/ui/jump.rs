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

pub struct JumpRedrawParams<'a> {
    pub terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    pub theme: &'a RenderTheme,
    pub tabs: &'a mut [TabState],
    pub active_tab: usize,
    pub tab_labels: &'a [String],
    pub active_tab_pos: usize,
    pub categories: &'a [String],
    pub active_category: usize,
    pub startup_text: Option<&'a str>,
    pub header_note: Option<&'a str>,
    pub rows: &'a [JumpRow],
    pub selected: usize,
    pub area: Rect,
    pub header_area: Rect,
    pub category_area: Rect,
    pub tabs_area: Rect,
    pub footer_area: Rect,
    pub scroll: usize,
}

pub fn redraw_jump(params: JumpRedrawParams<'_>) -> Result<(), Box<dyn std::error::Error>> {
    params.terminal.draw(|f| {
        draw_jump_layout(
            f,
            JumpLayoutParams {
                theme: params.theme,
                tab_labels: params.tab_labels,
                active_tab_pos: params.active_tab_pos,
                categories: params.categories,
                active_category: params.active_category,
                header_note: params.header_note,
                startup_text: params.startup_text,
                header_area: params.header_area,
                category_area: params.category_area,
                tabs_area: params.tabs_area,
                footer_area: params.footer_area,
            },
        );
        draw_jump_table(
            f,
            params.area,
            params.rows,
            params.selected,
            params.theme,
            params.scroll,
        );
        if let Some(tab) = params.tabs.get_mut(params.active_tab) {
            draw_notice(f, f.area(), &mut tab.app, params.theme);
        }
    })?;
    Ok(())
}

struct JumpLayoutParams<'a> {
    theme: &'a RenderTheme,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    header_note: Option<&'a str>,
    startup_text: Option<&'a str>,
    header_area: Rect,
    category_area: Rect,
    tabs_area: Rect,
    footer_area: Rect,
}

fn draw_jump_layout(f: &mut ratatui::Frame<'_>, params: JumpLayoutParams<'_>) {
    draw_header(f, params.header_area, params.theme, params.header_note);
    draw_categories(
        f,
        params.category_area,
        params.categories,
        params.active_category,
        params.theme,
    );
    draw_tabs(
        f,
        params.tabs_area,
        params.tab_labels,
        params.active_tab_pos,
        params.theme,
        params.startup_text,
    );
    draw_footer(f, params.footer_area, params.theme, false);
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
    OverlayTable {
        title: Line::from("消息定位 · Enter/点击 跳转 · E 复制用户消息到新对话 · F2 退出"),
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
    }
}

// text utilities are centralized in text_utils
