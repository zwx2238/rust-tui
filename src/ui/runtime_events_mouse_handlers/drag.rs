use crate::render::RenderTheme;
use crate::ui::draw::layout::{PADDING_X, PADDING_Y};
use crate::ui::draw::{inner_area, scrollbar_area};
use crate::ui::input_click::click_to_cursor;
use crate::ui::logic::{point_in_rect, scroll_from_mouse};
use crate::ui::runtime_events_helpers::selection_view_text;
use crate::ui::selection::{Selection, chat_position_from_mouse};
use crate::ui::state::Focus;
use crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use tui_textarea::CursorMove;

use super::types::MouseDragParams;

pub(crate) fn handle_mouse_drag(params: MouseDragParams<'_>) {
    let Some(tab_state) = params.tabs.get_mut(params.active_tab) else {
        return;
    };
    let m = params.m;
    let msg_area = params.msg_area;
    let input_area = params.input_area;
    let msg_width = params.msg_width;
    let view_height = params.view_height;
    let total_lines = params.total_lines;
    let theme = params.theme;
    if drag_scrollbar_if_needed(tab_state, msg_area, view_height, total_lines, m) {
        return;
    }
    if drag_input_if_needed(tab_state, input_area, m) {
        return;
    }
    if tab_state.app.chat_selecting {
        drag_chat_selection(
            tab_state,
            msg_area,
            msg_width,
            view_height,
            m,
            theme,
        );
    }
}

fn drag_scrollbar_if_needed(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    msg_area: Rect,
    view_height: u16,
    total_lines: usize,
    m: MouseEvent,
) -> bool {
    if !tab_state.app.scrollbar_dragging {
        return false;
    }
    drag_scrollbar(
        tab_state,
        msg_area,
        view_height,
        total_lines,
        m,
    );
    true
}

fn drag_input_if_needed(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    input_area: Rect,
    m: MouseEvent,
) -> bool {
    if !tab_state.app.input_selecting || !point_in_rect(m.column, m.row, input_area) {
        return false;
    }
    drag_input_selection(tab_state, input_area, m);
    true
}

fn drag_scrollbar(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    msg_area: Rect,
    view_height: u16,
    total_lines: usize,
    m: MouseEvent,
) {
    let scroll_area = scrollbar_area(msg_area);
    let app = &mut tab_state.app;
    app.follow = false;
    app.scroll = scroll_from_mouse(total_lines, view_height, scroll_area, m.row);
    app.focus = Focus::Chat;
}

fn drag_input_selection(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    input_area: Rect,
    m: MouseEvent,
) {
    let app = &mut tab_state.app;
    let (row, col) = click_to_cursor(app, input_area, m.column, m.row);
    if !app.input.is_selecting() {
        app.input.start_selection();
    }
    app.input
        .move_cursor(CursorMove::Jump(row as u16, col as u16));
}

fn drag_chat_selection(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    msg_area: Rect,
    msg_width: usize,
    view_height: u16,
    m: MouseEvent,
    theme: &RenderTheme,
) {
    let text = selection_view_text(tab_state, msg_width, theme, view_height);
    let app = &mut tab_state.app;
    let inner = inner_area(msg_area, PADDING_X, PADDING_Y);
    let pos = chat_position_from_mouse(&text, app.scroll, inner, m.column, m.row);
    if let Some(sel) = app.chat_selection {
        app.chat_selection = Some(Selection {
            start: sel.start,
            end: pos,
        });
    }
}
