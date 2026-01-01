use crate::render::RenderTheme;
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::state::PendingCodeExec;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState};
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

const MIN_POPUP_WIDTH: u16 = 40;
const MIN_POPUP_HEIGHT: u16 = 8;
const ACTION_ROWS: usize = 2;

pub(crate) fn draw_code_exec_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingCodeExec,
    selected: usize,
    theme: &RenderTheme,
) {
    f.render_widget(Clear, area);
    let mask = Block::default().style(base_style(theme));
    f.render_widget(mask, area);

    let width = (area.width * 80 / 100)
        .max(MIN_POPUP_WIDTH)
        .min(area.width.saturating_sub(2).max(MIN_POPUP_WIDTH));
    let height = (area.height * 70 / 100)
        .max(MIN_POPUP_HEIGHT)
        .min(area.height.saturating_sub(2).max(MIN_POPUP_HEIGHT));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let popup = Rect {
        x,
        y,
        width,
        height,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(vec![
            Span::styled("代码执行确认", Style::default().fg(base_fg(theme)).add_modifier(Modifier::BOLD)),
        ]))
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(3),
    ])
    .margin(1)
    .split(inner);

    let code_text = build_code_text(&pending.code, chunks[0].width);
    let code_para = Paragraph::new(code_text).style(base_style(theme));
    f.render_widget(code_para, chunks[0]);

    let rows = vec![
        Row::new(vec![Cell::from("执行 (Enter/Y)")]),
        Row::new(vec![Cell::from("拒绝 (Esc/N)")]),
    ];
    let mut state = TableState::default();
    state.select(Some(selected.min(ACTION_ROWS - 1)));
    let table = Table::new(rows, vec![Constraint::Percentage(100)])
        .style(base_style(theme))
        .row_highlight_style(Style::default().bg(selection_bg(theme.bg)))
        .header(Row::new(vec![Cell::from("操作")]).style(
            Style::default().fg(base_fg(theme)).add_modifier(Modifier::BOLD),
        ))
        .block(Block::default().borders(Borders::TOP));
    f.render_stateful_widget(table, chunks[1], &mut state);
}

fn build_code_text(code: &str, width: u16) -> Text<'static> {
    let max_width = width.saturating_sub(2).max(10) as usize;
    let mut lines: Vec<String> = Vec::new();
    if code.trim().is_empty() {
        lines.push("<空代码>".to_string());
    } else {
        for raw in code.lines() {
            if raw.is_empty() {
                lines.push(String::new());
                continue;
            }
            let wrapped = wrap(raw, max_width);
            if wrapped.is_empty() {
                lines.push(String::new());
            } else {
                for item in wrapped {
                    lines.push(item.into_owned());
                }
            }
        }
    }
    let max_lines = 12usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        lines[max_lines - 1].push_str(" ...");
    }
    let text = lines
        .into_iter()
        .map(|l| {
            let pad = max_width.saturating_sub(UnicodeWidthStr::width(l.as_str()));
            let mut s = l;
            if pad > 0 {
                s.push_str(&" ".repeat(pad));
            }
            Line::from(s)
        })
        .collect::<Vec<_>>();
    Text::from(text)
}
