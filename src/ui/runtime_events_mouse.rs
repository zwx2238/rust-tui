use crate::render::RenderTheme;
use crate::ui::command_suggestions::{clear_command_suggestions, refresh_command_suggestions};
use crate::ui::draw::layout::{PADDING_X, PADDING_Y};
use crate::ui::draw::{inner_area, scrollbar_area};
use crate::ui::input_click::click_to_cursor;
use crate::ui::logic::{point_in_rect, scroll_from_mouse};
use crate::ui::runtime_events_helpers::{hit_test_edit_button, selection_view_text};
use crate::ui::runtime_helpers::TabState;
use crate::ui::scroll::SCROLL_STEP_U16;
use crate::ui::selection::{Selection, chat_position_from_mouse};
use crate::ui::state::Focus;
use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use tui_textarea::CursorMove;

use super::runtime_events_tabs::handle_tab_category_click;

pub(crate) fn handle_mouse_event(
    m: MouseEvent,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &[String],
    active_category: &mut usize,
    tabs_area: Rect,
    msg_area: Rect,
    input_area: Rect,
    category_area: Rect,
    msg_width: usize,
    view_height: u16,
    total_lines: usize,
    theme: &RenderTheme,
) -> Option<usize> {
    match m.kind {
        MouseEventKind::Down(_) => handle_mouse_down(
            m, tabs, active_tab, categories, active_category, tabs_area, msg_area, input_area,
            category_area, msg_width, view_height, total_lines, theme,
        ),
        MouseEventKind::Up(_) => { handle_mouse_up(tabs, *active_tab); None }
        MouseEventKind::Drag(_) => { handle_mouse_drag(m, tabs, *active_tab, msg_area, input_area, msg_width, view_height, total_lines, theme); None }
        MouseEventKind::ScrollUp => { handle_mouse_scroll(tabs, *active_tab, false); None }
        MouseEventKind::ScrollDown => { handle_mouse_scroll(tabs, *active_tab, true); None }
        _ => None,
    }
}

fn handle_mouse_down(
    m: MouseEvent,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &[String],
    active_category: &mut usize,
    tabs_area: Rect,
    msg_area: Rect,
    input_area: Rect,
    category_area: Rect,
    msg_width: usize,
    view_height: u16,
    total_lines: usize,
    theme: &RenderTheme,
) -> Option<usize> {
    if handle_tab_category_click(
        m.column, m.row, tabs, active_tab, categories, active_category, tabs_area, category_area,
    ) { return None; }
    if handle_scrollbar_click(m, tabs, *active_tab, msg_area, view_height, total_lines) { return None; }
    if let Some(tab_state) = tabs.get_mut(*active_tab) {
        if point_in_rect(m.column, m.row, input_area) { handle_input_click(tab_state, input_area, m); return None; }
        if point_in_rect(m.column, m.row, msg_area) {
            return handle_message_click(tab_state, msg_area, msg_width, view_height, m, theme);
        }
    }
    None
}

fn handle_scrollbar_click(
    m: MouseEvent,
    tabs: &mut Vec<TabState>,
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
    app.input.move_cursor(CursorMove::Jump(row as u16, col as u16));
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
    if let Some(msg_idx) = hit_test_edit_button(tab_state, msg_area, msg_width, theme, view_height, m.column, m.row) {
        let app = &mut tab_state.app;
        app.focus = Focus::Chat;
        app.follow = false;
        app.chat_selection = None;
        app.chat_selecting = false;
        app.input_selecting = false;
        return Some(msg_idx);
    }
    let text = selection_view_text(tab_state, msg_width, theme, view_height);
    let app = &mut tab_state.app;
    app.focus = Focus::Chat;
    app.follow = false;
    app.input_selecting = false;
    clear_command_suggestions(app);
    let inner = inner_area(msg_area, PADDING_X, PADDING_Y);
    let pos = chat_position_from_mouse(&text, app.scroll, inner, m.column, m.row);
    app.chat_selecting = true;
    app.chat_selection = Some(Selection { start: pos, end: pos });
    None
}

fn handle_mouse_up(tabs: &mut Vec<TabState>, active_tab: usize) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let app = &mut tab_state.app;
        app.scrollbar_dragging = false;
        app.input_selecting = false;
        if app.chat_selecting {
            app.chat_selecting = false;
            if app.chat_selection.map(|s| s.is_empty()).unwrap_or(false) {
                app.chat_selection = None;
            }
        }
    }
}

fn handle_mouse_drag(
    m: MouseEvent,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    msg_area: Rect,
    input_area: Rect,
    msg_width: usize,
    view_height: u16,
    total_lines: usize,
    theme: &RenderTheme,
) {
    let Some(tab_state) = tabs.get_mut(active_tab) else { return; };
    let dragging = tab_state.app.scrollbar_dragging;
    let input_selecting = tab_state.app.input_selecting;
    let chat_selecting = tab_state.app.chat_selecting;
    if dragging { drag_scrollbar(tab_state, msg_area, view_height, total_lines, m); return; }
    if input_selecting && point_in_rect(m.column, m.row, input_area) { drag_input_selection(tab_state, input_area, m); return; }
    if chat_selecting { drag_chat_selection(tab_state, msg_area, msg_width, view_height, m, theme); }
}

fn drag_scrollbar(tab_state: &mut TabState, msg_area: Rect, view_height: u16, total_lines: usize, m: MouseEvent) {
    let scroll_area = scrollbar_area(msg_area);
    let app = &mut tab_state.app;
    app.follow = false;
    app.scroll = scroll_from_mouse(total_lines, view_height, scroll_area, m.row);
    app.focus = Focus::Chat;
}

fn drag_input_selection(tab_state: &mut TabState, input_area: Rect, m: MouseEvent) {
    let app = &mut tab_state.app;
    let (row, col) = click_to_cursor(app, input_area, m.column, m.row);
    if !app.input.is_selecting() {
        app.input.start_selection();
    }
    app.input.move_cursor(CursorMove::Jump(row as u16, col as u16));
}

fn drag_chat_selection(
    tab_state: &mut TabState,
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
        app.chat_selection = Some(Selection { start: sel.start, end: pos });
    }
}

fn handle_mouse_scroll(tabs: &mut Vec<TabState>, active_tab: usize, down: bool) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let app = &mut tab_state.app;
        if down { app.scroll = app.scroll.saturating_add(SCROLL_STEP_U16); }
        else { app.scroll = app.scroll.saturating_sub(SCROLL_STEP_U16); }
        app.follow = false;
        app.focus = Focus::Chat;
    }
}
