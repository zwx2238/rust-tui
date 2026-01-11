use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::interaction::scroll::{SCROLL_STEP_U16, max_scroll_u16};
use crate::framework::widget_system::runtime::state::Focus;

pub(crate) fn handle_mouse_up(tabs: &mut [TabState], active_tab: usize) {
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

pub(crate) fn handle_mouse_scroll(
    tabs: &mut [TabState],
    active_tab: usize,
    down: bool,
    view_height: u16,
    total_lines: usize,
) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let app = &mut tab_state.app;
        let max_scroll = max_scroll_u16(total_lines, view_height);
        if down {
            if app.scroll >= max_scroll && select_next_message(app) {
                app.scroll = 0;
            } else {
                app.scroll = app.scroll.saturating_add(SCROLL_STEP_U16);
            }
        } else if app.scroll == 0 && select_prev_message(app) {
            app.scroll = u16::MAX;
        } else {
            app.scroll = app.scroll.saturating_sub(SCROLL_STEP_U16);
        }
        app.follow = false;
        app.focus = Focus::Chat;
    }
}

fn select_next_message(app: &mut crate::framework::widget_system::runtime::state::App) -> bool {
    let len = app.messages.len();
    if len == 0 {
        return false;
    }
    let next = app.message_history.selected.saturating_add(1);
    if next >= len {
        return false;
    }
    app.message_history.selected = next;
    app.chat_selection = None;
    app.chat_selecting = false;
    true
}

fn select_prev_message(app: &mut crate::framework::widget_system::runtime::state::App) -> bool {
    if app.message_history.selected == 0 {
        return false;
    }
    app.message_history.selected -= 1;
    app.chat_selection = None;
    app.chat_selecting = false;
    true
}
