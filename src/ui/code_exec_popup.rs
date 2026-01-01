use crate::render::RenderTheme;
use crate::ui::draw::style::{base_fg, base_style};
use crate::ui::state::PendingCodeExec;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

const MIN_POPUP_WIDTH: u16 = 40;
const MIN_POPUP_HEIGHT: u16 = 8;
const OUTER_MARGIN: u16 = 2;
#[derive(Copy, Clone)]
pub(crate) struct CodeExecPopupLayout {
    pub(crate) popup: Rect,
    pub(crate) code_area: Rect,
    pub(crate) approve_btn: Rect,
    pub(crate) deny_btn: Rect,
}

pub(crate) fn code_exec_popup_layout(area: Rect) -> CodeExecPopupLayout {
    let safe = Rect {
        x: area.x.saturating_add(OUTER_MARGIN),
        y: area.y.saturating_add(OUTER_MARGIN),
        width: area.width.saturating_sub(OUTER_MARGIN.saturating_mul(2)),
        height: area.height.saturating_sub(OUTER_MARGIN.saturating_mul(2)),
    };
    let width = (safe.width * 80 / 100)
        .max(MIN_POPUP_WIDTH)
        .min(safe.width.saturating_sub(2).max(MIN_POPUP_WIDTH));
    let height = (safe.height * 70 / 100)
        .max(MIN_POPUP_HEIGHT)
        .min(safe.height.saturating_sub(2).max(MIN_POPUP_HEIGHT));
    let x = safe.x + (safe.width.saturating_sub(width)) / 2;
    let y = safe.y + (safe.height.saturating_sub(height)) / 2;
    let popup = Rect {
        x,
        y,
        width,
        height,
    };
    let inner = Rect {
        x: popup.x.saturating_add(1),
        y: popup.y.saturating_add(1),
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(3)])
        .split(inner);
    let actions_area = chunks[1];
    let gap = 2u16;
    let btn_width = actions_area
        .width
        .saturating_sub(gap)
        .saturating_div(2)
        .max(6);
    let approve_btn = Rect {
        x: actions_area.x,
        y: actions_area.y,
        width: btn_width,
        height: actions_area.height,
    };
    let deny_btn = Rect {
        x: actions_area.x.saturating_add(btn_width + gap),
        y: actions_area.y,
        width: actions_area
            .width
            .saturating_sub(btn_width + gap)
            .max(btn_width),
        height: actions_area.height,
    };
    CodeExecPopupLayout {
        popup,
        code_area: chunks[0],
        approve_btn,
        deny_btn,
    }
}

pub(crate) fn code_exec_max_scroll(code: &str, width: u16, height: u16) -> usize {
    let max_width = width.saturating_sub(2).max(10) as usize;
    let mut lines = 0usize;
    if code.trim().is_empty() {
        lines = 1;
    } else {
        for raw in code.lines() {
            if raw.is_empty() {
                lines += 1;
                continue;
            }
            let wrapped = wrap(raw, max_width);
            let count = if wrapped.is_empty() { 1 } else { wrapped.len() };
            lines += count;
        }
    }
    let view_height = height.saturating_sub(1) as usize;
    lines.saturating_sub(view_height)
}

pub(crate) fn draw_code_exec_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingCodeExec,
    scroll: usize,
    theme: &RenderTheme,
) {
    let layout = code_exec_popup_layout(area);
    f.render_widget(Clear, layout.popup);
    let mask = Block::default().style(base_style(theme));
    f.render_widget(mask, layout.popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(vec![
            Span::styled("代码执行确认", Style::default().fg(base_fg(theme)).add_modifier(Modifier::BOLD)),
        ]))
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    f.render_widget(block, layout.popup);

    let code_text = build_code_text(
        &pending.code,
        layout.code_area.width,
        layout.code_area.height,
        scroll,
    );
    let code_para = Paragraph::new(code_text)
        .style(base_style(theme))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(code_para, layout.code_area);

    let approve_style = base_style(theme);
    let deny_style = base_style(theme);
    let approve_block = Block::default().borders(Borders::ALL).style(approve_style);
    let deny_block = Block::default().borders(Borders::ALL).style(deny_style);
    f.render_widget(approve_block, layout.approve_btn);
    f.render_widget(
        Paragraph::new(Line::from("确认执行")).style(approve_style).alignment(ratatui::layout::Alignment::Center),
        layout.approve_btn,
    );
    f.render_widget(deny_block, layout.deny_btn);
    f.render_widget(
        Paragraph::new(Line::from("取消拒绝")).style(deny_style).alignment(ratatui::layout::Alignment::Center),
        layout.deny_btn,
    );
}

fn build_code_text(code: &str, width: u16, height: u16, scroll: usize) -> Text<'static> {
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
    let view_height = height.saturating_sub(1) as usize;
    let start = scroll.min(lines.len());
    let end = (start + view_height).min(lines.len());
    let slice = &lines[start..end];
    let text = slice
        .iter()
        .map(|l| {
            let pad = max_width.saturating_sub(UnicodeWidthStr::width(l.as_str()));
            let mut s = l.clone();
            if pad > 0 {
                s.push_str(&" ".repeat(pad));
            }
            Line::from(s)
        })
        .collect::<Vec<_>>();
    Text::from(text)
}
