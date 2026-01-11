use crate::render::label_for_role;
use crate::framework::widget_system::draw::style::{base_fg, base_style, selection_bg};
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::interaction::scroll::{SCROLL_STEP_I32, max_scroll};
use crate::framework::widget_system::interaction::text_utils::{collapse_text, truncate_to_width};
use crate::framework::widget_system::BoxConstraints;
use crate::framework::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use crate::framework::widget_system::runtime_tick::{
    DisplayMessage, build_display_messages, select_visible_message,
};
use crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Rect, Size};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::error::Error;

use super::helpers::point_in_rect;

pub(super) struct MessageHistoryWidget;

impl Widget for MessageHistoryWidget {
    fn measure(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        let width = crate::framework::widget_system::layout::compute_history_width(bc.max.width);
        Ok(Size {
            width,
            height: bc.max.height,
        })
    }

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
        event: &Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        rect: Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if rect.width == 0 || rect.height == 0 {
            return Ok(EventResult::ignored());
        }
        if !point_in_rect(m.column, m.row, rect) {
            return Ok(EventResult::ignored());
        }
        handle_history_mouse(ctx, layout, update, rect, *m)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        if should_skip_render(rect) {
            return Ok(());
        }
        let preview_width = preview_width(rect);
        let messages = display_messages_for_frame(frame);
        update_history_scroll(frame, rect, &messages);
        draw_history(frame, rect, preview_width, &messages);
        Ok(())
    }
}

fn handle_history_mouse(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    rect: Rect,
    m: MouseEvent,
) -> Result<EventResult, Box<dyn Error>> {
    let viewport = viewport_rows(rect);
    let messages = display_messages_for_event(ctx);
    let len = messages.len();
    match m.kind {
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let Some(app) = ctx.tabs.get_mut(*ctx.active_tab).map(|tab| &mut tab.app) else {
                return Ok(EventResult::ignored());
            };
            if viewport == 0 || len == 0 {
                return Ok(EventResult::handled());
            }
            let delta = if matches!(m.kind, MouseEventKind::ScrollUp) {
                -SCROLL_STEP_I32
            } else {
                SCROLL_STEP_I32
            };
            let max = max_scroll(len, viewport);
            app.message_history.scroll = offset_scroll(app.message_history.scroll, delta);
            if app.message_history.scroll > max {
                app.message_history.scroll = max;
            }
            Ok(EventResult::handled())
        }
        MouseEventKind::Down(MouseButton::Left) => {
            handle_history_click(ctx, layout, update, rect, m.row, &messages)
        }
        _ => Ok(EventResult::ignored()),
    }
}

fn handle_history_click(
    ctx: &mut EventCtx<'_>,
    _layout: &FrameLayout,
    _update: &UpdateOutput,
    rect: Rect,
    mouse_y: u16,
    messages: &[DisplayMessage],
) -> Result<EventResult, Box<dyn Error>> {
    if messages.is_empty() {
        return Ok(EventResult::ignored());
    }
    let Some(app) = ctx.tabs.get_mut(*ctx.active_tab).map(|tab| &mut tab.app) else {
        return Ok(EventResult::ignored());
    };
    let Some(row) = row_at(app, rect, mouse_y) else {
        return Ok(EventResult::ignored());
    };
    let Some(index) = messages.get(row).map(|msg| msg.index) else {
        return Ok(EventResult::ignored());
    };
    select_row(app, rect, row, index);
    reset_detail_scroll(app);
    Ok(EventResult::handled())
}

fn select_row(
    app: &mut crate::framework::widget_system::runtime::state::App,
    rect: Rect,
    row: usize,
    index: usize,
) {
    let viewport = viewport_rows(rect);
    app.message_history.selected = index;
    ensure_row_visible(&mut app.message_history.scroll, row, viewport);
}

fn reset_detail_scroll(app: &mut crate::framework::widget_system::runtime::state::App) {
    app.scroll = 0;
    app.follow = false;
    app.focus = crate::framework::widget_system::runtime::state::Focus::Chat;
    app.chat_selection = None;
    app.chat_selecting = false;
}

fn row_at(app: &crate::framework::widget_system::runtime::state::App, rect: Rect, mouse_y: u16) -> Option<usize> {
    if mouse_y <= rect.y || mouse_y >= rect.y + rect.height.saturating_sub(1) {
        return None;
    }
    let inner_y = mouse_y.saturating_sub(rect.y + 1) as usize;
    Some(app.message_history.scroll.saturating_add(inner_y))
}

fn should_skip_render(rect: Rect) -> bool {
    rect.width < 3 || rect.height < 3
}

fn viewport_rows(rect: Rect) -> usize {
    rect.height.saturating_sub(2) as usize
}

fn preview_width(rect: Rect) -> usize {
    let inner = rect.width.saturating_sub(2) as usize;
    let prefix = preview_prefix_width();
    inner.saturating_sub(prefix)
}

fn preview_prefix_width() -> usize {
    8
}

fn draw_history(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    rect: Rect,
    preview_width: usize,
    messages: &[DisplayMessage],
) {
    let theme = frame.state.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .title("历史")
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    let lines = history_lines(frame, rect, preview_width, messages);
    let p = Paragraph::new(lines).block(block).style(base_style(theme));
    frame.frame.render_widget(p, rect);
}

fn history_lines(
    frame: &WidgetFrame<'_, '_, '_, '_>,
    rect: Rect,
    preview_width: usize,
    messages: &[DisplayMessage],
) -> Vec<Line<'static>> {
    let viewport = viewport_rows(rect);
    let Some(app) = frame.state.active_app() else {
        return Vec::new();
    };
    let start = app.message_history.scroll;
    let end = start.saturating_add(viewport).min(messages.len());
    let selected = app.message_history.selected;
    (start..end)
        .filter_map(|row| {
            messages.get(row).map(|msg| {
                let selected_row = msg.index == selected;
                format_line(msg, row, preview_width, selected_row, frame.state.theme)
            })
        })
        .collect()
}

fn format_line(
    msg: &DisplayMessage,
    row: usize,
    preview_width: usize,
    selected: bool,
    theme: &crate::render::RenderTheme,
) -> Line<'static> {
    let role = role_label(&msg.message.role);
    let preview = preview_for(msg, preview_width);
    let s = format!("{:>3} {} {}", row + 1, role, preview);
    if selected {
        Line::from(s).style(Style::default().bg(selection_bg(theme.bg)))
    } else {
        Line::from(s)
    }
}

fn preview_for(msg: &DisplayMessage, width: usize) -> String {
    truncate_to_width(&collapse_text(&msg.message.content), width)
}

fn role_label(role: &str) -> String {
    label_for_role(role, None).unwrap_or_else(|| role.to_string())
}

fn display_messages_for_frame(frame: &WidgetFrame<'_, '_, '_, '_>) -> Vec<DisplayMessage> {
    frame
        .state
        .with_active_tab(|tab| build_display_messages(&tab.app, frame.state.args))
        .unwrap_or_default()
}

fn display_messages_for_event(ctx: &EventCtx<'_>) -> Vec<DisplayMessage> {
    ctx.tabs
        .get(*ctx.active_tab)
        .map(|tab| build_display_messages(&tab.app, ctx.args))
        .unwrap_or_default()
}

fn update_history_scroll(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    rect: Rect,
    messages: &[DisplayMessage],
) {
    let viewport = viewport_rows(rect);
    let Some(app) = frame.state.active_app_mut() else {
        return;
    };
    let selected = select_visible_message(app, messages);
    let selected_row = selected.and_then(|idx| selected_row(messages, idx));
    clamp_history_scroll(app, messages.len(), viewport, selected_row);
}

fn clamp_history_scroll(
    app: &mut crate::framework::widget_system::runtime::state::App,
    len: usize,
    viewport: usize,
    selected_row: Option<usize>,
) {
    if len == 0 {
        app.message_history.scroll = 0;
        return;
    }
    let max = max_scroll(len, viewport);
    if app.message_history.scroll > max {
        app.message_history.scroll = max;
    }
    if let Some(row) = selected_row {
        ensure_row_visible(&mut app.message_history.scroll, row, viewport);
    }
}

fn selected_row(messages: &[DisplayMessage], index: usize) -> Option<usize> {
    messages.iter().position(|msg| msg.index == index)
}

fn ensure_row_visible(scroll: &mut usize, row: usize, viewport: usize) {
    if viewport == 0 {
        return;
    }
    if row < *scroll {
        *scroll = row;
    } else if row >= *scroll + viewport {
        *scroll = row.saturating_sub(viewport.saturating_sub(1));
    }
}

fn offset_scroll(scroll: usize, delta: i32) -> usize {
    if delta.is_negative() {
        let step = delta.unsigned_abs() as usize;
        scroll.saturating_sub(step)
    } else {
        let step = delta as usize;
        scroll.saturating_add(step)
    }
}
