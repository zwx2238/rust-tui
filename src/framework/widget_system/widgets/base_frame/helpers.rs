use crate::framework::widget_system::events::{
    TabCategoryClickParams, handle_tab_category_click, handle_tabs_wheel,
};
use crate::framework::widget_system::bindings::bind_event;
use crate::framework::widget_system::context::{EventCtx, UpdateOutput};
use crate::framework::widget_system::lifecycle::EventResult;
use crate::framework::widget_system::widget_pod::WidgetPod;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

pub(super) fn point_in_rect(column: u16, row: u16, rect: ratatui::layout::Rect) -> bool {
    column >= rect.x && column < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

pub(super) fn scrollbar_hit(area: ratatui::layout::Rect, column: u16, row: u16) -> bool {
    let scroll = crate::ui::draw::scrollbar_area(area);
    point_in_rect(column, row, scroll)
}

pub(super) fn handle_tab_category_mouse_down(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    rect: ratatui::layout::Rect,
    m: crossterm::event::MouseEvent,
) -> Result<EventResult, Box<dyn Error>> {
    if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
        return Ok(EventResult::ignored());
    }
    if !point_in_rect(m.column, m.row, rect) {
        return Ok(EventResult::ignored());
    }
    let binding = bind_event(ctx, layout, update);
    let handled = handle_tab_category_click(TabCategoryClickParams {
        mouse_x: m.column,
        mouse_y: m.row,
        tabs: binding.dispatch.tabs,
        active_tab: binding.dispatch.active_tab,
        categories: binding.dispatch.categories,
        active_category: binding.dispatch.active_category,
        tabs_area: binding.layout.tabs_area,
        category_area: binding.layout.category_area,
    });
    Ok(if handled {
        EventResult::handled()
    } else {
        EventResult::ignored()
    })
}

pub(super) fn handle_tab_category_wheel(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    rect: ratatui::layout::Rect,
    m: crossterm::event::MouseEvent,
) -> Result<EventResult, Box<dyn Error>> {
    let down = match m.kind {
        crossterm::event::MouseEventKind::ScrollUp => false,
        crossterm::event::MouseEventKind::ScrollDown => true,
        _ => return Ok(EventResult::ignored()),
    };
    if !point_in_rect(m.column, m.row, rect) {
        return Ok(EventResult::ignored());
    }
    let binding = bind_event(ctx, layout, update);
    handle_tabs_wheel(
        binding.dispatch.tabs,
        binding.dispatch.active_tab,
        binding.dispatch.categories,
        *binding.dispatch.active_category,
        down,
    );
    Ok(EventResult::handled())
}

pub(super) fn pod_event_handled<T: crate::framework::widget_system::lifecycle::Widget>(
    pod: &mut WidgetPod<T>,
    ctx: &mut EventCtx<'_>,
    event: &crossterm::event::Event,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn Error>> {
    Ok(pod.event(ctx, event, layout, update, jump_rows)?.handled)
}
