use crate::render::{messages_to_text, RenderTheme};
use crate::ui::input::cursor_position;
use crate::ui::state::{App, Focus};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::block::{Padding, Title};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;

pub fn layout_chunks(size: Rect) -> (Rect, Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(5)].as_ref())
        .split(size);
    (layout[0], layout[1])
}

pub fn redraw(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &App,
    theme: &RenderTheme,
    label_suffixes: &[(usize, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let (msg_area, input_area) = layout_chunks(size);
    let msg_width = inner_width(msg_area, PADDING_X);
    let text = messages_to_text(
        &app.messages,
        msg_width,
        theme,
        label_suffixes,
        app.pending_assistant,
    );
    terminal.draw(|f| {
        draw_messages(
            f,
            msg_area,
            &text,
            app.scroll,
            theme,
            app.focus == Focus::Chat,
        );
        draw_input(
            f,
            input_area,
            &app.input,
            app.cursor,
            theme,
            app.focus == Focus::Input,
            app.busy,
        );
    })?;
    Ok(())
}

const PADDING_X: u16 = 1;
const PADDING_Y: u16 = 0;

fn draw_messages(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    text: &Text<'_>,
    scroll: u16,
    theme: &RenderTheme,
    focused: bool,
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
    let total_lines = text.lines.len();
    let lines_above = scroll as usize;
    let lines_below = total_lines.saturating_sub(lines_above + content_height as usize);
    let right_title = Title::from(format!("{} {}", lines_above, lines_below))
        .alignment(Alignment::Right);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("对话")
        .title(right_title)
        .padding(Padding::new(PADDING_X, PADDING_X, PADDING_Y, PADDING_Y))
        .style(style)
        .border_style(border_style);
    let paragraph = Paragraph::new(text.clone())
        .block(block)
        .style(style)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(paragraph, area);
}

fn draw_input(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    input: &str,
    cursor: usize,
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
    let block = Block::default()
        .borders(Borders::ALL)
        .title(if busy { "输入(禁用)" } else { "输入" })
        .padding(Padding::new(PADDING_X, PADDING_X, PADDING_Y, PADDING_Y))
        .style(style)
        .border_style(border_style);
    let paragraph = Paragraph::new(input)
        .block(block)
        .style(style)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);

    let (line_idx, col) = cursor_position(input, cursor);
    let x = col as u16;
    let inner = inner_area(area, PADDING_X, PADDING_Y);
    let max_x = inner.x.saturating_add(inner.width.saturating_sub(1));
    let cursor_x = inner.x.saturating_add(x).min(max_x);
    let max_y = inner.y.saturating_add(inner.height.saturating_sub(1));
    let cursor_y = inner.y.saturating_add(line_idx as u16).min(max_y);
    if focused && !busy {
        f.set_cursor(cursor_x, cursor_y);
    }
}

pub fn inner_area(area: Rect, padding_x: u16, padding_y: u16) -> Rect {
    Rect {
        x: area.x + 1 + padding_x,
        y: area.y + 1 + padding_y,
        width: area.width.saturating_sub(2 + padding_x * 2),
        height: area.height.saturating_sub(2 + padding_y * 2),
    }
}

pub fn inner_width(area: Rect, padding_x: u16) -> usize {
    area.width.saturating_sub(2 + padding_x * 2) as usize
}

pub fn inner_height(area: Rect, padding_y: u16) -> u16 {
    area.height.saturating_sub(2 + padding_y * 2)
}
