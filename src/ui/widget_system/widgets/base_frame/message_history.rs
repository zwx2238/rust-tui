use crate::render::label_for_role;
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::jump::{JumpRow, build_jump_rows};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::runtime_view::{ViewAction, apply_view_action};
use crate::ui::scroll::{SCROLL_STEP_I32, max_scroll};
use crate::ui::widget_system::bindings::bind_event;
use crate::ui::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crate::ui::widget_system::BoxConstraints;
use crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Rect, Size};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::error::Error;

use super::helpers::point_in_rect;

pub(super) struct MessageHistoryWidget;

impl Widget for MessageHistoryWidget {
    fn measure(&mut self, _ctx: &mut LayoutCtx<'_>, bc: BoxConstraints) -> Result<Size, Box<dyn Error>> {
        let width = crate::ui::runtime_layout::compute_history_width(bc.max.width);
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
        jump_rows: &[JumpRow],
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
        handle_history_mouse(ctx, layout, update, jump_rows, rect, *m)
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
        refresh_jump_rows(frame, preview_width);
        clamp_jump_state(frame, rect);
        draw_history(frame, rect);
        Ok(())
    }
}

fn handle_history_mouse(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[JumpRow],
    rect: Rect,
    m: MouseEvent,
) -> Result<EventResult, Box<dyn Error>> {
    let viewport = viewport_rows(rect);
    let len = jump_rows.len();
    match m.kind {
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            if viewport == 0 || len == 0 {
                return Ok(EventResult::handled());
            }
            let delta = if matches!(m.kind, MouseEventKind::ScrollUp) {
                -SCROLL_STEP_I32
            } else {
                SCROLL_STEP_I32
            };
            let max = max_scroll(len, viewport);
            ctx.view.jump.scroll_by(delta, max, viewport);
            Ok(EventResult::handled())
        }
        MouseEventKind::Down(MouseButton::Left) => handle_history_click(ctx, layout, update, jump_rows, rect, m.row),
        _ => Ok(EventResult::ignored()),
    }
}

fn handle_history_click(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[JumpRow],
    rect: Rect,
    mouse_y: u16,
) -> Result<EventResult, Box<dyn Error>> {
    if jump_rows.is_empty() {
        return Ok(EventResult::ignored());
    }
    let Some(row) = row_at(ctx, rect, mouse_y) else {
        return Ok(EventResult::ignored());
    };
    if row >= jump_rows.len() {
        return Ok(EventResult::ignored());
    }
    let viewport = viewport_rows(rect);
    ctx.view.jump.select(row);
    ctx.view.jump.ensure_visible(viewport);
    let binding = bind_event(ctx, layout, update);
    let _ = apply_view_action(
        ViewAction::JumpTo(row),
        jump_rows,
        binding.dispatch.tabs,
        binding.dispatch.active_tab,
        binding.dispatch.categories,
        binding.dispatch.active_category,
    );
    Ok(EventResult::handled())
}

fn row_at(ctx: &EventCtx<'_>, rect: Rect, mouse_y: u16) -> Option<usize> {
    if mouse_y <= rect.y || mouse_y >= rect.y + rect.height.saturating_sub(1) {
        return None;
    }
    let inner_y = mouse_y.saturating_sub(rect.y + 1) as usize;
    Some(ctx.view.jump.scroll.saturating_add(inner_y))
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

fn refresh_jump_rows(frame: &mut WidgetFrame<'_, '_, '_, '_>, preview_width: usize) {
    frame.jump_rows.clear();
    let rows = frame
        .state
        .with_active_tab(|tab| {
            build_jump_rows(
                &tab.app.messages,
                frame.state.msg_width,
                preview_width,
                tab.app.pending_assistant,
            )
        })
        .unwrap_or_default();
    frame.jump_rows.extend(rows);
}

fn clamp_jump_state(frame: &mut WidgetFrame<'_, '_, '_, '_>, rect: Rect) {
    let viewport = viewport_rows(rect);
    let len = frame.jump_rows.len();
    frame.view.jump.clamp_with_viewport(len, viewport);
}

fn draw_history(frame: &mut WidgetFrame<'_, '_, '_, '_>, rect: Rect) {
    let theme = frame.state.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .title("历史")
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    let lines = visible_lines(frame, rect);
    let p = Paragraph::new(lines).block(block).style(base_style(theme));
    frame.frame.render_widget(p, rect);
}

fn visible_lines(frame: &WidgetFrame<'_, '_, '_, '_>, rect: Rect) -> Vec<Line<'static>> {
    let viewport = viewport_rows(rect);
    let start = frame.view.jump.scroll;
    let end = start.saturating_add(viewport);
    let selected = frame.view.jump.selected;
    frame
        .jump_rows
        .iter()
        .enumerate()
        .skip(start)
        .take(end.saturating_sub(start))
        .map(|(i, row)| format_line(row, i == selected, frame.state.theme))
        .collect()
}

fn format_line(row: &JumpRow, selected: bool, theme: &crate::render::RenderTheme) -> Line<'static> {
    let role = role_label(&row.role);
    let s = format!("{:>3} {} {}", row.index, role, row.preview);
    if selected {
        Line::from(s).style(Style::default().bg(selection_bg(theme.bg)))
    } else {
        Line::from(s)
    }
}

fn role_label(role: &str) -> String {
    label_for_role(role, None).unwrap_or_else(|| role.to_string())
}

