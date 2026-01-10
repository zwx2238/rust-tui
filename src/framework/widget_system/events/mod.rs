mod geometry;
mod helpers;
mod key;
mod mouse;
mod mouse_handlers;
mod tabs;

pub(crate) use geometry::{point_in_rect, scroll_from_mouse};
pub(crate) use helpers::{hit_test_edit_button, selection_view_text};
pub(crate) use key::handle_key_event;
pub(crate) use mouse::{MouseEventParams, handle_mouse_event};
pub(crate) use mouse_handlers::handle_tabs_wheel;
pub(crate) use tabs::{TabCategoryClickParams, handle_tab_category_click};

use crate::framework::widget_system::commands::command_suggestions::refresh_command_suggestions;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::runtime::state::Focus;

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
