use crate::render::{RenderTheme, messages_to_plain_lines, messages_to_viewport_text_cached};
use crate::ui::clipboard;
use crate::ui::draw::layout::{PADDING_X, PADDING_Y};
use crate::ui::draw::{inner_area, scrollbar_area};
use crate::ui::input::handle_key;
use crate::ui::input_click::click_to_cursor;
use crate::ui::logic::{build_label_suffixes, point_in_rect, scroll_from_mouse, timer_text};
use crate::ui::runtime_helpers::{
    TabState, tab_index_at, tab_labels_for_category, visible_tab_indices,
};
use crate::ui::scroll::SCROLL_STEP_U16;
use crate::ui::selection::{Selection, chat_position_from_mouse, extract_selection};
use crate::ui::state::Focus;
use crossterm::event::{KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use tui_textarea::CursorMove;

pub(crate) fn handle_key_event(
    key: KeyEvent,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    msg_width: usize,
    theme: &RenderTheme,
) -> Result<bool, Box<dyn std::error::Error>> {
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && key.code == crossterm::event::KeyCode::Char('c')
    {
        if let Some(tab_state) = tabs.get_mut(active_tab) {
            let app = &mut tab_state.app;
            if app.focus == Focus::Input && app.input.is_selecting() {
                app.input.copy();
                let text = app.input.yank_text();
                clipboard::set(&text);
                return Ok(false);
            }
            if app.focus == Focus::Chat {
                if let Some(selection) = app.chat_selection {
                    let lines = messages_to_plain_lines(&app.messages, msg_width, theme);
                    let text = extract_selection(&lines, selection);
                    if !text.is_empty() {
                        clipboard::set(&text);
                    }
                    return Ok(false);
                }
            }
        }
        return Ok(true);
    }
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        if handle_key(key, &mut tab_state.app)? {
            return Ok(true);
        }
    }
    Ok(false)
}

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
        MouseEventKind::Down(_) => {
            if handle_tab_category_click(
                m.column,
                m.row,
                tabs,
                active_tab,
                categories,
                active_category,
                tabs_area,
                category_area,
            ) {
                return None;
            }
            let scroll_area = scrollbar_area(msg_area);
            if point_in_rect(m.column, m.row, scroll_area) && total_lines > view_height as usize {
                if let Some(tab_state) = tabs.get_mut(*active_tab) {
                    let app = &mut tab_state.app;
                    app.scrollbar_dragging = true;
                    app.follow = false;
                    app.scroll = scroll_from_mouse(total_lines, view_height, scroll_area, m.row);
                    app.focus = Focus::Chat;
                }
                return None;
            }
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                if point_in_rect(m.column, m.row, input_area) {
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
                } else if point_in_rect(m.column, m.row, msg_area) {
                    if let Some(msg_idx) = hit_test_edit_button(
                        tab_state,
                        msg_area,
                        msg_width,
                        theme,
                        view_height,
                        m.column,
                        m.row,
                    ) {
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
                    let inner = inner_area(msg_area, PADDING_X, PADDING_Y);
                    let pos = chat_position_from_mouse(&text, app.scroll, inner, m.column, m.row);
                    app.chat_selecting = true;
                    app.chat_selection = Some(Selection {
                        start: pos,
                        end: pos,
                    });
                }
            }
        }
        MouseEventKind::Up(_) => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
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
        MouseEventKind::Drag(_) => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                let dragging = tab_state.app.scrollbar_dragging;
                let input_selecting = tab_state.app.input_selecting;
                let chat_selecting = tab_state.app.chat_selecting;
                if dragging {
                    let scroll_area = scrollbar_area(msg_area);
                    let app = &mut tab_state.app;
                    app.follow = false;
                    app.scroll = scroll_from_mouse(total_lines, view_height, scroll_area, m.row);
                    app.focus = Focus::Chat;
                } else if input_selecting && point_in_rect(m.column, m.row, input_area) {
                    let app = &mut tab_state.app;
                    let (row, col) = click_to_cursor(app, input_area, m.column, m.row);
                    if !app.input.is_selecting() {
                        app.input.start_selection();
                    }
                    app.input
                        .move_cursor(CursorMove::Jump(row as u16, col as u16));
                } else if chat_selecting {
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
            }
        }
        MouseEventKind::ScrollUp => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                let app = &mut tab_state.app;
                app.scroll = app.scroll.saturating_sub(SCROLL_STEP_U16);
                app.follow = false;
                app.focus = Focus::Chat;
            }
        }
        MouseEventKind::ScrollDown => {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                let app = &mut tab_state.app;
                app.scroll = app.scroll.saturating_add(SCROLL_STEP_U16);
                app.follow = false;
                app.focus = Focus::Chat;
            }
        }
        _ => {}
    }
    None
}

pub(crate) fn handle_paste_event(paste: &str, tabs: &mut Vec<TabState>, active_tab: usize) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let app = &mut tab_state.app;
        if app.focus == Focus::Input && !app.busy {
            let text = paste.replace("\r\n", "\n").replace('\r', "\n");
            app.input.insert_str(text);
        }
    }
}

pub(crate) fn handle_tab_category_click(
    mouse_x: u16,
    mouse_y: u16,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &[String],
    active_category: &mut usize,
    tabs_area: Rect,
    category_area: Rect,
) -> bool {
    if point_in_rect(mouse_x, mouse_y, category_area) {
        let row = mouse_y.saturating_sub(category_area.y) as usize;
        if row < categories.len() {
            *active_category = row;
            if let Some(category) = categories.get(*active_category) {
                let visible = visible_tab_indices(tabs, category);
                if let Some(tab_idx) = visible.first() {
                    *active_tab = *tab_idx;
                }
            }
        }
        return true;
    }
    if point_in_rect(mouse_x, mouse_y, tabs_area) {
        let category = categories
            .get(*active_category)
            .map(|s| s.as_str())
            .unwrap_or("默认");
        let labels = tab_labels_for_category(tabs, category);
        if let Some(pos) = tab_index_at(mouse_x, tabs_area, &labels) {
            let visible = visible_tab_indices(tabs, category);
            if let Some(idx) = visible.get(pos) {
                *active_tab = *idx;
            }
        }
        return true;
    }
    false
}

fn selection_view_text(
    tab_state: &mut TabState,
    msg_width: usize,
    theme: &RenderTheme,
    view_height: u16,
) -> ratatui::text::Text<'static> {
    let app = &tab_state.app;
    let label_suffixes = build_label_suffixes(app, &timer_text(app));
    let (text, _) = messages_to_viewport_text_cached(
        &app.messages,
        msg_width,
        theme,
        &label_suffixes,
        app.pending_assistant,
        app.scroll,
        view_height,
        &mut tab_state.render_cache,
    );
    text
}

fn hit_test_edit_button(
    tab_state: &mut TabState,
    msg_area: Rect,
    msg_width: usize,
    theme: &RenderTheme,
    view_height: u16,
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    if tab_state.app.message_layouts.is_empty() {
        return None;
    }
    let inner = inner_area(msg_area, PADDING_X, PADDING_Y);
    if mouse_x < inner.x
        || mouse_x >= inner.x + inner.width
        || mouse_y < inner.y
        || mouse_y >= inner.y + inner.height
    {
        return None;
    }
    let text = selection_view_text(tab_state, msg_width, theme, view_height);
    let app = &tab_state.app;
    let (row, col) = chat_position_from_mouse(&text, app.scroll, inner, mouse_x, mouse_y);
    for layout in &app.message_layouts {
        if layout.label_line == row {
            if let Some((start, end)) = layout.button_range {
                if col >= start && col < end {
                    return Some(layout.index);
                }
            }
            break;
        }
    }
    None
}
