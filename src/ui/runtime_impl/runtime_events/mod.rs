mod runtime_events_key;
mod runtime_events_mouse;
mod runtime_events_tabs;

pub(crate) use runtime_events_key::handle_key_event;
pub(crate) use runtime_events_mouse::{MouseEventParams, handle_mouse_event};
pub(crate) use runtime_events_tabs::{TabCategoryClickParams, handle_tab_category_click};

use crate::ui::command_suggestions::refresh_command_suggestions;
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::Focus;

pub(crate) fn handle_paste_event(paste: &str, tabs: &mut [TabState], active_tab: usize) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let app = &mut tab_state.app;
        if app.focus == Focus::Input && !app.busy {
            let text = paste.replace("\r\n", "\n").replace('\r', "\n");
            app.input.insert_str(text);
            refresh_command_suggestions(app);
        }
    }
}
