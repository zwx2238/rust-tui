use crate::framework::widget_system::events::{
    handle_tab_category_click, hit_test_edit_button, point_in_rect, scroll_from_mouse,
    selection_view_text,
};
use crate::render::RenderTheme;
use crate::framework::widget_system::commands::command_suggestions::{clear_command_suggestions, refresh_command_suggestions};
use crate::framework::widget_system::draw::inner_area;
use crate::framework::widget_system::draw::layout::{PADDING_X, PADDING_Y};
use crate::framework::widget_system::draw::scrollbar_area;
use crate::framework::widget_system::interaction::input_click::click_to_cursor;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::interaction::selection::{Selection, chat_position_from_mouse};
use crate::framework::widget_system::runtime::state::Focus;
use crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use tui_textarea::CursorMove;

use super::types::MouseDownParams;

pub(crate) fn handle_mouse_down(params: MouseDownParams<'_>) -> Option<usize> {
    let mut params = params;
    if handle_tab_click(&mut params) {
        return None;
    }
    if handle_scrollbar_click(
        params.m,
        params.tabs,
        *params.active_tab,
        params.msg_area,
        params.view_height,
        params.total_lines,
    ) {
        return None;
    }
    handle_active_tab_click(&mut params)
}

fn handle_tab_click(params: &mut MouseDownParams<'_>) -> bool {
    handle_tab_category_click(crate::framework::widget_system::events::TabCategoryClickParams {
        mouse_x: params.m.column,
        mouse_y: params.m.row,
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        tabs_area: params.tabs_area,
        category_area: params.category_area,
    })
}

fn handle_active_tab_click(params: &mut MouseDownParams<'_>) -> Option<usize> {
    let tab_state = params.tabs.get_mut(*params.active_tab)?;
    if point_in_rect(params.m.column, params.m.row, params.input_area) {
        handle_input_click(tab_state, params.input_area, params.m);
        return None;
    }
    if point_in_rect(params.m.column, params.m.row, params.msg_area) {
        return handle_message_click(
            tab_state,
            params.msg_area,
            params.msg_width,
            params.view_height,
            params.m,
            params.theme,
        );
    }
    None
}

fn handle_scrollbar_click(
    m: MouseEvent,
    tabs: &mut [TabState],
    active_tab: usize,
    msg_area: Rect,
    view_height: u16,
    total_lines: usize,
) -> bool {
    let scroll_area = scrollbar_area(msg_area);
    if !point_in_rect(m.column, m.row, scroll_area) || total_lines <= view_height as usize {
        return false;
    }
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let app = &mut tab_state.app;
        app.scrollbar_dragging = true;
        app.follow = false;
        app.scroll = scroll_from_mouse(total_lines, view_height, scroll_area, m.row);
        app.focus = Focus::Chat;
    }
    true
}

fn handle_input_click(tab_state: &mut TabState, input_area: Rect, m: MouseEvent) {
    let app = &mut tab_state.app;
    app.focus = Focus::Input;
    app.nav_mode = false;
    app.chat_selection = None;
    app.chat_selecting = false;
    app.input_selecting = true;
    let (row, col) = click_to_cursor(app, input_area, m.column, m.row);
    app.input.cancel_selection();
    app.input
        .move_cursor(CursorMove::Jump(row as u16, col as u16));
    refresh_command_suggestions(app);
}

fn handle_message_click(
    tab_state: &mut TabState,
    msg_area: Rect,
    msg_width: usize,
    view_height: u16,
    m: MouseEvent,
    theme: &RenderTheme,
) -> Option<usize> {
    if let Some(msg_idx) =
        handle_message_edit_click(tab_state, msg_area, msg_width, view_height, m, theme)
    {
        return Some(msg_idx);
    }
    start_chat_selection(tab_state, msg_area, msg_width, view_height, m, theme);
    None
}

fn handle_message_edit_click(
    tab_state: &mut TabState,
    msg_area: Rect,
    msg_width: usize,
    view_height: u16,
    m: MouseEvent,
    theme: &RenderTheme,
) -> Option<usize> {
    let msg_idx = hit_test_edit_button(
        tab_state,
        msg_area,
        msg_width,
        theme,
        view_height,
        m.column,
        m.row,
    )?;
    let app = &mut tab_state.app;
    app.focus = Focus::Chat;
    app.follow = false;
    app.chat_selection = None;
    app.chat_selecting = false;
    app.input_selecting = false;
    Some(msg_idx)
}

fn start_chat_selection(
    tab_state: &mut TabState,
    msg_area: Rect,
    msg_width: usize,
    view_height: u16,
    m: MouseEvent,
    theme: &RenderTheme,
) {
    let text = selection_view_text(tab_state, msg_width, theme, view_height);
    let app = &mut tab_state.app;
    app.focus = Focus::Chat;
    app.follow = false;
    app.input_selecting = false;
    clear_command_suggestions(app);
    let inner = inner_area(msg_area, PADDING_X, PADDING_Y);
    let pos = chat_position_from_mouse(&text, app.scroll, inner, m.column, m.row);
    app.chat_selecting = true;
    app.chat_selection = Some(Selection {
        start: pos,
        end: pos,
    });
}
