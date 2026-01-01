use crate::render::RenderTheme;
use crate::ui::draw::layout::{PADDING_X, PADDING_Y, inner_area, scrollbar_area};
use crate::ui::draw::style::{base_fg, base_style, focus_border_style};
use crate::ui::scroll::{max_scroll, max_scroll_u16};
use crate::ui::scroll_debug::{self, ScrollDebug};
use crate::ui::selection::{Selection, apply_selection_to_text};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::block::Padding;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

pub(crate) fn draw_messages(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    text: &Text<'_>,
    scroll: u16,
    theme: &RenderTheme,
    focused: bool,
    total_lines: usize,
    selection: Option<Selection>,
) {
    let style = base_style(theme);
    let border_style = focus_border_style(theme, focused);
    let inner = inner_area(area, PADDING_X, PADDING_Y);
    let content_height = inner.height;
    let lines_above = scroll as usize;
    let lines_below = total_lines.saturating_sub(lines_above + content_height as usize);
    let mut right_text = format!("{} {}", lines_above, lines_below);
    if scroll_debug::enabled() {
        let max_scroll = max_scroll_u16(total_lines, content_height);
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
    let mut display_text = text.clone();
    if let Some(selection) = selection {
        let select_style = Style::default().bg(Color::DarkGray);
        display_text =
            apply_selection_to_text(&display_text, scroll as usize, selection, select_style);
    }
    let paragraph = Paragraph::new(display_text)
        .block(block)
        .style(style)
        .wrap(Wrap { trim: false })
        .scroll((0, 0));
    f.render_widget(paragraph, area);

    if total_lines > content_height as usize {
        let viewport_len = content_height as usize;
        let max_scroll = max_scroll(total_lines, viewport_len);
        let scrollbar_content_len = max_scroll.saturating_add(1).max(1);
        let scroll_area = scrollbar_area(area);
        let mut state = ScrollbarState::new(scrollbar_content_len)
            .position(scroll as usize)
            .viewport_content_length(viewport_len);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(Color::Blue))
            .track_style(Style::default().fg(base_fg(theme)));
        f.render_stateful_widget(scrollbar, scroll_area, &mut state);
    }
}
