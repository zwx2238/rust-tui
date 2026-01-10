use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::interaction::scroll::SCROLL_STEP_U16;
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

pub(crate) fn handle_mouse_scroll(tabs: &mut [TabState], active_tab: usize, down: bool) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let app = &mut tab_state.app;
        if down {
            app.scroll = app.scroll.saturating_add(SCROLL_STEP_U16);
        } else {
            app.scroll = app.scroll.saturating_sub(SCROLL_STEP_U16);
        }
        app.follow = false;
        app.focus = Focus::Chat;
    }
}
