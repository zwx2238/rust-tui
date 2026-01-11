mod down;
mod drag;
mod scroll;
mod tabs_wheel;
mod types;

use crate::framework::widget_system::events::point_in_rect;
use crossterm::event::MouseEventKind;

pub(crate) use tabs_wheel::handle_tabs_wheel;
pub(crate) use types::MouseEventParams;
use types::{MouseDownParams, MouseDragParams};

pub(crate) fn handle_mouse_event(params: MouseEventParams<'_>) -> Option<usize> {
    match params.m.kind {
        MouseEventKind::Down(_) => down::handle_mouse_down(build_mouse_down_params(params)),
        MouseEventKind::Up(_) => {
            scroll::handle_mouse_up(params.tabs, *params.active_tab);
            None
        }
        MouseEventKind::Drag(_) => {
            drag::handle_mouse_drag(build_mouse_drag_params(params));
            None
        }
        MouseEventKind::ScrollUp => {
            if point_in_rect(params.m.column, params.m.row, params.tabs_area) {
                tabs_wheel::handle_tabs_wheel(
                    params.tabs,
                    params.active_tab,
                    params.categories,
                    *params.active_category,
                    false,
                );
                return None;
            }
            scroll::handle_mouse_scroll(
                params.tabs,
                *params.active_tab,
                false,
                params.view_height,
                params.total_lines,
            );
            None
        }
        MouseEventKind::ScrollDown => {
            if point_in_rect(params.m.column, params.m.row, params.tabs_area) {
                tabs_wheel::handle_tabs_wheel(
                    params.tabs,
                    params.active_tab,
                    params.categories,
                    *params.active_category,
                    true,
                );
                return None;
            }
            scroll::handle_mouse_scroll(
                params.tabs,
                *params.active_tab,
                true,
                params.view_height,
                params.total_lines,
            );
            None
        }
        _ => None,
    }
}

fn build_mouse_down_params(params: MouseEventParams<'_>) -> MouseDownParams<'_> {
    MouseDownParams {
        m: params.m,
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        tabs_area: params.tabs_area,
        msg_area: params.msg_area,
        input_area: params.input_area,
        category_area: params.category_area,
        msg_width: params.msg_width,
        view_height: params.view_height,
        total_lines: params.total_lines,
        theme: params.theme,
        args: params.args,
    }
}

fn build_mouse_drag_params(params: MouseEventParams<'_>) -> MouseDragParams<'_> {
    MouseDragParams {
        m: params.m,
        tabs: params.tabs,
        active_tab: *params.active_tab,
        msg_area: params.msg_area,
        input_area: params.input_area,
        msg_width: params.msg_width,
        view_height: params.view_height,
        total_lines: params.total_lines,
        theme: params.theme,
        args: params.args,
    }
}
