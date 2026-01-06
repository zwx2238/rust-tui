use crate::render::{RenderTheme, count_message_lines, label_for_role};
use crate::types::Message;
use crate::ui::draw::{draw_categories, draw_footer, draw_header, draw_tabs};
use crate::ui::overlay_table::{OverlayTable, draw_overlay_table, header_style};
use crate::ui::text_utils::{collapse_text, truncate_to_width};
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};

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

pub(crate) struct JumpLayoutParams<'a> {
    pub(crate) theme: &'a RenderTheme,
    pub(crate) tab_labels: &'a [String],
    pub(crate) active_tab_pos: usize,
    pub(crate) categories: &'a [String],
    pub(crate) active_category: usize,
    pub(crate) header_note: Option<&'a str>,
    pub(crate) startup_text: Option<&'a str>,
    pub(crate) header_area: Rect,
    pub(crate) category_area: Rect,
    pub(crate) tabs_area: Rect,
    pub(crate) footer_area: Rect,
}

pub(crate) fn draw_jump_layout(f: &mut ratatui::Frame<'_>, params: JumpLayoutParams<'_>) {
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

pub(crate) fn draw_jump_table(
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
    rows: &'a [JumpRow],
    selected: usize,
    theme: &'a RenderTheme,
    scroll: usize,
) -> OverlayTable<'a> {
    OverlayTable {
        title: Line::from(jump_title()),
        header: jump_header(theme),
        rows: jump_body(rows),
        widths: jump_widths(),
        selected,
        scroll,
        theme,
    }
}

fn jump_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("序号"),
        Cell::from("角色"),
        Cell::from("内容"),
    ])
    .style(header_style(theme))
}

fn jump_body<'a>(rows: &'a [JumpRow]) -> Vec<Row<'a>> {
    rows.iter()
        .map(|row| {
            Row::new(vec![
                Cell::from(row.index.to_string()),
                Cell::from(row.role.clone()),
                Cell::from(row.preview.clone()),
            ])
        })
        .collect()
}

fn jump_widths() -> Vec<Constraint> {
    vec![
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Min(10),
    ]
}

fn jump_title() -> &'static str {
    "消息定位 · Enter/点击 跳转 · E 复制用户消息到新对话 · F2 退出"
}

// text utilities are centralized in text_utils
