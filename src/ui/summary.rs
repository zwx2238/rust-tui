use crate::render::RenderTheme;
use crate::ui::draw::draw_tabs;
use crate::ui::popup_table::{draw_table_popup, popup_row_at, TablePopup};
use crate::ui::runtime_helpers::TabState;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use ratatui::Terminal;
use std::io::Stdout;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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

pub fn summary_row_at(
    area: Rect,
    row_count: usize,
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    popup_row_at(area, row_count, 0, mouse_x, mouse_y)
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
    .style(
        Style::default()
            .fg(theme.fg.unwrap_or(Color::White))
            .add_modifier(Modifier::BOLD),
    );

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
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
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
