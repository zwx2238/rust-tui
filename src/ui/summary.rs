use crate::render::RenderTheme;
use crate::types::ROLE_USER;
use crate::ui::draw::draw_tabs;
use crate::ui::popup_table::{TablePopup, draw_table_popup, header_style};
use crate::ui::runtime_helpers::TabState;
use crate::ui::text_utils::truncate_to_width;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use std::io::Stdout;

pub struct SummaryRow {
    pub tab_id: usize,
    pub message_count: usize,
    pub status: &'static str,
    pub latest_user: String,
}

pub fn build_summary_rows(tabs: &[TabState], max_latest_width: usize) -> Vec<SummaryRow> {
    tabs.iter()
        .enumerate()
        .map(|(idx, tab)| {
            let status = if tab.app.busy { "generating" } else { "done" };
            let latest_user = latest_user_question(&tab.app.messages)
                .map(|s| truncate_to_width(s, max_latest_width))
                .unwrap_or_else(|| "-".to_string());
            SummaryRow {
                tab_id: idx + 1,
                message_count: tab.app.messages.len(),
                status,
                latest_user,
            }
        })
        .collect()
}

pub fn redraw_summary(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &[TabState],
    active_tab: usize,
    theme: &RenderTheme,
    startup_text: Option<&str>,
    selected_row: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)].as_ref())
        .split(size);
    let tabs_area = layout[0];
    let body_area = layout[1];
    let max_latest_width = inner_area(body_area).width.saturating_sub(30) as usize;
    let rows = build_summary_rows(tabs, max_latest_width.max(10));
    terminal.draw(|f| {
        draw_tabs(f, tabs_area, tabs.len(), active_tab, theme, startup_text);
        draw_summary_table(f, body_area, &rows, selected_row, theme);
    })?;
    Ok(())
}

fn draw_summary_table(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    rows: &[SummaryRow],
    selected_row: usize,
    theme: &RenderTheme,
) {
    let header = Row::new(vec![
        Cell::from("Tab"),
        Cell::from("消息数"),
        Cell::from("状态"),
        Cell::from("最新提问"),
    ])
    .style(header_style(theme));

    let body = rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.tab_id.to_string()),
            Cell::from(row.message_count.to_string()),
            Cell::from(row.status),
            Cell::from(row.latest_user.clone()),
        ])
    });

    let popup = TablePopup {
        title: Line::from("汇总页 · F1 退出 · 点击行进入"),
        header,
        rows: body.collect(),
        widths: vec![
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Min(10),
        ],
        selected: selected_row,
        scroll: 0,
        theme,
    };
    draw_table_popup(f, area, popup);
}

fn inner_area(area: Rect) -> Rect {
    Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

fn latest_user_question(messages: &[crate::types::Message]) -> Option<&str> {
    messages
        .iter()
        .rev()
        .find(|m| m.role == ROLE_USER)
        .map(|m| m.content.as_str())
}

// text utilities are centralized in text_utils
