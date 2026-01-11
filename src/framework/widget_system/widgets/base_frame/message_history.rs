use crate::render::label_for_role;
use crate::framework::widget_system::draw::style::{base_fg, base_style, selection_bg};
use crate::framework::widget_system::widgets::jump::JumpRow;
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::interaction::scroll::{SCROLL_STEP_I32, max_scroll};
use crate::framework::widget_system::interaction::text_utils::{collapse_text, truncate_to_width};
use crate::framework::widget_system::BoxConstraints;
use crate::framework::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
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
        _jump_rows: &[JumpRow],
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
        let len = history_len(frame);
        clamp_history_state_with_len(frame, rect, len);
        draw_history(frame, rect, preview_width, len);
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
    let len = history_len_event(ctx);
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
            app.message_history.scroll_by(delta, max, viewport);
            Ok(EventResult::handled())
        }
        MouseEventKind::Down(MouseButton::Left) => {
            handle_history_click(ctx, layout, update, rect, m.row, len)
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
    len: usize,
) -> Result<EventResult, Box<dyn Error>> {
    if len == 0 {
        return Ok(EventResult::ignored());
    }
    let Some(app) = ctx.tabs.get_mut(*ctx.active_tab).map(|tab| &mut tab.app) else {
        return Ok(EventResult::ignored());
    };
    let Some(row) = row_at(app, rect, mouse_y) else {
        return Ok(EventResult::ignored());
    };
    if row >= len {
        return Ok(EventResult::ignored());
    }
    select_row(app, rect, row);
    reset_detail_scroll(app);
    Ok(EventResult::handled())
}

fn select_row(app: &mut crate::framework::widget_system::runtime::state::App, rect: Rect, row: usize) {
    let viewport = viewport_rows(rect);
    app.message_history.select(row);
    app.message_history.ensure_visible(viewport);
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

fn history_len(frame: &WidgetFrame<'_, '_, '_, '_>) -> usize {
    frame
        .state
        .with_active_tab(|tab| tab.app.messages.len())
        .unwrap_or(0)
}

fn history_len_event(ctx: &EventCtx<'_>) -> usize {
    ctx.tabs
        .get(*ctx.active_tab)
        .map(|tab| tab.app.messages.len())
        .unwrap_or(0)
}

fn clamp_history_state_with_len(frame: &mut WidgetFrame<'_, '_, '_, '_>, rect: Rect, len: usize) {
    let viewport = viewport_rows(rect);
    if let Some(app) = frame.state.active_app_mut() {
        app.message_history.clamp_with_viewport(len, viewport);
    }
}

fn draw_history(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    rect: Rect,
    preview_width: usize,
    len: usize,
) {
    let theme = frame.state.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .title("历史")
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    let lines = history_lines(frame, rect, preview_width, len);
    let p = Paragraph::new(lines).block(block).style(base_style(theme));
    frame.frame.render_widget(p, rect);
}

fn history_lines(
    frame: &WidgetFrame<'_, '_, '_, '_>,
    rect: Rect,
    preview_width: usize,
    len: usize,
) -> Vec<Line<'static>> {
    let viewport = viewport_rows(rect);
    let Some(app) = frame.state.active_app() else {
        return Vec::new();
    };
    let start = app.message_history.scroll;
    let end = start.saturating_add(viewport).min(len);
    let selected = app.message_history.selected;
    let Some(lines) = frame.state.with_active_tab(|tab| {
        (start..end)
            .map(|i| format_line(tab, i, preview_width, i == selected, frame.state.theme))
            .collect()
    }) else {
        return Vec::new();
    };
    lines
}

fn format_line(
    tab: &crate::framework::widget_system::runtime::runtime_helpers::TabState,
    row: usize,
    preview_width: usize,
    selected: bool,
    theme: &crate::render::RenderTheme,
) -> Line<'static> {
    let idx = row;
    let role = tab
        .app
        .messages
        .get(idx)
        .map(|m| role_label(&m.role))
        .unwrap_or_default();
    let preview = preview_for(tab, idx, preview_width);
    let s = format!("{:>3} {} {}", idx + 1, role, preview);
    if selected {
        Line::from(s).style(Style::default().bg(selection_bg(theme.bg)))
    } else {
        Line::from(s)
    }
}

fn preview_for(tab: &crate::framework::widget_system::runtime::runtime_helpers::TabState, idx: usize, width: usize) -> String {
    let Some(msg) = tab.app.messages.get(idx) else {
        return String::new();
    };
    truncate_to_width(&collapse_text(&msg.content), width)
}

fn role_label(role: &str) -> String {
    label_for_role(role, None).unwrap_or_else(|| role.to_string())
}
