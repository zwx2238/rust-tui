use crate::render::RenderTheme;
use crate::framework::widget_system::draw::layout::{PADDING_X, PADDING_Y, inner_area, scrollbar_area};
use crate::framework::widget_system::draw::style::{base_fg, base_style, focus_border_style};
use crate::framework::widget_system::interaction::scroll::{max_scroll, max_scroll_u16};
use crate::framework::widget_system::interaction::scroll_debug::{self, ScrollDebug};
use crate::framework::widget_system::interaction::selection::{Selection, apply_selection_to_text};
use crate::framework::widget_system::events::{MouseEventParams, handle_mouse_event};
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::runtime::state::Focus;
use crate::framework::widget_system::bindings::bind_event;
use crate::framework::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::block::Padding;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use super::helpers::{point_in_rect, scrollbar_hit};

pub(super) struct MessagesWidget;

impl Widget for MessagesWidget {
    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _jump_rows: &[crate::framework::widget_system::widgets::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !point_in_rect(m.column, m.row, rect)
            && !scrollbar_hit(layout.layout.msg_area, m.column, m.row)
        {
            return Ok(EventResult::ignored());
        }
        handle_messages_mouse(ctx, layout, update, *m);
        Ok(EventResult::handled())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(app) = frame.state.active_app() {
            draw_messages(
                frame.frame,
                MessagesDrawParams {
                    area: rect,
                    text: frame.state.text,
                    scroll: app.scroll,
                    theme: frame.state.theme,
                    focused: app.focus == Focus::Chat,
                    total_lines: frame.state.total_lines,
                    selection: app.chat_selection,
                },
            );
        }
        Ok(())
    }
}

struct MessagesDrawParams<'a> {
    area: Rect,
    text: &'a Text<'a>,
    scroll: u16,
    theme: &'a RenderTheme,
    focused: bool,
    total_lines: usize,
    selection: Option<Selection>,
}

fn draw_messages(f: &mut ratatui::Frame<'_>, params: MessagesDrawParams<'_>) {
    let style = base_style(params.theme);
    let content_height = inner_area(params.area, PADDING_X, PADDING_Y).height;
    let right_title = build_right_title(
        params.area,
        params.scroll,
        params.total_lines,
        content_height,
    );
    let block = build_block(params.theme, params.focused, right_title, style);
    let paragraph = build_paragraph(params.text, params.scroll, params.selection, block, style);
    f.render_widget(paragraph, params.area);
    render_scrollbar(
        f,
        params.area,
        params.theme,
        params.scroll,
        params.total_lines,
        content_height,
    );
}

fn build_right_title(
    area: Rect,
    scroll: u16,
    total_lines: usize,
    content_height: u16,
) -> Line<'static> {
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
    Line::from(right_text).right_aligned()
}

fn build_block(
    theme: &RenderTheme,
    focused: bool,
    right_title: Line<'static>,
    style: Style,
) -> Block<'static> {
    let border_style = focus_border_style(theme, focused);
    Block::default()
        .borders(Borders::ALL)
        .title_top("对话")
        .title_top(right_title)
        .padding(Padding::new(PADDING_X, PADDING_X, PADDING_Y, PADDING_Y))
        .style(style)
        .border_style(border_style)
}

fn build_paragraph<'a>(
    text: &'a Text<'a>,
    scroll: u16,
    selection: Option<Selection>,
    block: Block<'static>,
    style: Style,
) -> Paragraph<'a> {
    let display_text = apply_selection(text, scroll, selection);
    Paragraph::new(display_text)
        .block(block)
        .style(style)
        .wrap(Wrap { trim: false })
        .scroll((0, 0))
}

fn apply_selection<'a>(text: &'a Text<'a>, scroll: u16, selection: Option<Selection>) -> Text<'a> {
    let mut display_text = text.clone();
    if let Some(selection) = selection {
        let select_style = Style::default().bg(Color::DarkGray);
        display_text =
            apply_selection_to_text(&display_text, scroll as usize, selection, select_style);
    }
    display_text
}

fn render_scrollbar(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    theme: &RenderTheme,
    scroll: u16,
    total_lines: usize,
    content_height: u16,
) {
    if total_lines <= content_height as usize {
        return;
    }
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

fn handle_messages_mouse(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    m: crossterm::event::MouseEvent,
) {
    let binding = bind_event(ctx, layout, update);
    let _ = handle_mouse_event(MouseEventParams {
        m,
        tabs: binding.dispatch.tabs,
        active_tab: binding.dispatch.active_tab,
        categories: binding.dispatch.categories,
        active_category: binding.dispatch.active_category,
        tabs_area: binding.layout.tabs_area,
        msg_area: binding.layout.msg_area,
        input_area: binding.layout.input_area,
        category_area: binding.layout.category_area,
        msg_width: binding.dispatch.msg_width,
        view_height: binding.layout.view_height,
        total_lines: update.active_data.total_lines,
        theme: binding.dispatch.theme,
    });
}
