use crate::render::RenderTheme;
use crate::ui::logic::tab_label;
use unicode_width::UnicodeWidthStr;
use crate::ui::state::{App, Focus};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::block::Padding;
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;
use tui_textarea::TextArea;
use crate::ui::scroll_debug::{self, ScrollDebug};

pub fn layout_chunks(size: Rect, input_height: u16) -> (Rect, Rect, Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(input_height.max(1)),
            ]
            .as_ref(),
        )
        .split(size);
    (layout[0], layout[1], layout[2])
}

pub fn redraw(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    tabs_len: usize,
    active_tab: usize,
    startup_text: Option<&str>,
    input_height: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let (tabs_area, msg_area, input_area) = layout_chunks(size, input_height);
    terminal.draw(|f| {
        draw_tabs(f, tabs_area, tabs_len, active_tab, theme, startup_text);
        draw_messages(
            f,
            msg_area,
            text,
            app.scroll,
            theme,
            app.focus == Focus::Chat,
            total_lines,
        );
        draw_input(
            f,
            input_area,
            &mut app.input,
            theme,
            app.focus == Focus::Input,
            app.busy,
        );
    })?;
    Ok(())
}

fn draw_tabs(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    tabs_len: usize,
    active_tab: usize,
    theme: &RenderTheme,
    startup_text: Option<&str>,
) {
    let mut label = String::new();
    for i in 0..tabs_len {
        let tab = tab_label(i);
        label.push_str(&tab);
        if i + 1 < tabs_len {
            label.push('│');
        }
    }
    let mut spans = Vec::new();
    let mut cursor = 0usize;
    for (i, part) in label.split('│').enumerate() {
        let style = if i == active_tab {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(theme.fg.unwrap_or(Color::White))
        };
        spans.push(ratatui::text::Span::styled(part.to_string(), style));
        cursor += part.width();
        if i + 1 < tabs_len {
            spans.push(ratatui::text::Span::styled(
                "│",
                Style::default().fg(theme.fg.unwrap_or(Color::White)),
            ));
            cursor += 1;
        }
    }
    if let Some(text) = startup_text {
        let width = area.width as usize;
        let text_width = text.width();
        if width > cursor + text_width {
            let pad = width.saturating_sub(cursor + text_width);
            spans.push(ratatui::text::Span::raw(" ".repeat(pad)));
            spans.push(ratatui::text::Span::styled(
                text.to_string(),
                Style::default().fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White)),
            ));
        }
    }
    let line = ratatui::text::Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.bg));
    f.render_widget(paragraph, area);
}
const PADDING_X: u16 = 1;
const PADDING_Y: u16 = 0;
pub const SCROLLBAR_WIDTH: u16 = 2;

pub fn scrollbar_area(area: Rect) -> Rect {
    let width = SCROLLBAR_WIDTH.min(area.width);
    Rect {
        x: area.x.saturating_add(area.width.saturating_sub(width)),
        y: area.y.saturating_add(1),
        width,
        height: area.height.saturating_sub(2),
    }
}

fn draw_messages(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    text: &Text<'_>,
    scroll: u16,
    theme: &RenderTheme,
    focused: bool,
    total_lines: usize,
) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.fg.unwrap_or(Color::White));
    let border_style = if focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(theme.fg.unwrap_or(Color::White))
    };
    let inner = inner_area(area, PADDING_X, PADDING_Y);
    let content_height = inner.height;
    let lines_above = scroll as usize;
    let lines_below = total_lines.saturating_sub(lines_above + content_height as usize);
    let mut right_text = format!("{} {}", lines_above, lines_below);
    if scroll_debug::enabled() {
        let max_scroll = total_lines
            .saturating_sub(content_height as usize)
            .min(u16::MAX as usize) as u16;
        let info = ScrollDebug {
            total_lines,
            scroll,
            content_height,
            max_scroll,
            viewport_len: content_height as usize,
            scroll_area_height: scrollbar_area(area).height,
        };
        right_text.push_str(" | ");
        right_text.push_str(&scroll_debug::format(&info));
    }
    let right_title = ratatui::text::Line::from(right_text).right_aligned();
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top("对话")
        .title_top(right_title)
        .padding(Padding::new(PADDING_X, PADDING_X, PADDING_Y, PADDING_Y))
        .style(style)
        .border_style(border_style);
    let _ = scroll;
    let paragraph = Paragraph::new(text.clone())
        .block(block)
        .style(style)
        .wrap(Wrap { trim: false })
        .scroll((0, 0));
    f.render_widget(paragraph, area);

    if total_lines > content_height as usize {
        let viewport_len = content_height as usize;
        let max_scroll = total_lines.saturating_sub(viewport_len);
        let scrollbar_content_len = max_scroll.saturating_add(1).max(1);
        let scroll_area = scrollbar_area(area);
        let mut state = ScrollbarState::new(scrollbar_content_len)
            .position(scroll as usize)
            .viewport_content_length(viewport_len);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(Color::Blue))
            .track_style(Style::default().fg(theme.fg.unwrap_or(Color::White)));
        f.render_stateful_widget(scrollbar, scroll_area, &mut state);
    }
}

fn draw_input(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    input: &mut TextArea<'_>,
    theme: &RenderTheme,
    focused: bool,
    busy: bool,
) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.fg.unwrap_or(Color::White));
    let border_style = if focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(theme.fg.unwrap_or(Color::White))
    };
    let (line_idx, col) = input.cursor();
    let total_lines = input.lines().len().max(1);
    let status = format!(
        "{} · 行 {}/{} 列 {}",
        if busy { "输入(禁用)" } else { "输入" },
        line_idx + 1,
        total_lines,
        col + 1
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(status)
        .title_top(ratatui::text::Line::from("Enter 发送 · Ctrl+J 换行").right_aligned())
        .padding(Padding::new(PADDING_X, PADDING_X, PADDING_Y, PADDING_Y))
        .style(style)
        .border_style(border_style);
    input.set_block(block);
    input.set_style(style);
    input.set_cursor_style(if focused && !busy {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    });
    input.set_placeholder_text(if busy {
        "正在生成回复，输入已禁用"
    } else {
        "输入内容后按 Enter 发送"
    });
    input.set_placeholder_style(Style::default().fg(Color::DarkGray));
    f.render_widget(&*input, area);
}

pub fn inner_area(area: Rect, padding_x: u16, padding_y: u16) -> Rect {
    Rect {
        x: area.x + 1 + padding_x,
        y: area.y + 1 + padding_y,
        width: area.width.saturating_sub(2 + padding_x * 2),
        height: area.height.saturating_sub(2 + padding_y * 2),
    }
}

pub fn input_inner_area(area: Rect) -> Rect {
    inner_area(area, PADDING_X, PADDING_Y)
}

pub fn inner_width(area: Rect, padding_x: u16) -> usize {
    area.width.saturating_sub(2 + padding_x * 2) as usize
}

pub fn inner_height(area: Rect, padding_y: u16) -> u16 {
    area.height.saturating_sub(2 + padding_y * 2)
}
