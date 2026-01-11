use crate::args::Args;
use crate::render::{RenderTheme, SingleMessageRenderParams, message_to_viewport_text_cached};
use crate::framework::widget_system::draw::inner_area;
use crate::framework::widget_system::draw::layout::{PADDING_X, PADDING_Y};
use crate::framework::widget_system::runtime::logic::{build_label_suffixes, timer_text};
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::runtime_tick::{
    DisplayMessage, build_display_messages, select_visible_message,
};
use crate::framework::widget_system::interaction::selection::chat_position_from_mouse;
use ratatui::layout::Rect;

pub(crate) fn selection_view_text(
    tab_state: &mut TabState,
    args: &Args,
    msg_width: usize,
    theme: &RenderTheme,
    view_height: u16,
) -> ratatui::text::Text<'static> {
    let app = &mut tab_state.app;
    let messages = build_display_messages(app, args);
    let Some((idx, msg)) = active_message(app, &messages) else {
        return ratatui::text::Text::default();
    };
    let label_suffixes = build_label_suffixes(app, &timer_text(app));
    let params = SingleMessageRenderParams {
        message: msg,
        message_index: idx,
        width: msg_width,
        theme,
        label_suffixes: &label_suffixes,
        streaming: app.pending_assistant == Some(idx),
        scroll: app.scroll,
        height: view_height,
    };
    let (text, _) = message_to_viewport_text_cached(params, &mut tab_state.render_cache);
    text
}

pub(crate) struct HitTestEditButtonParams<'a> {
    pub tab_state: &'a mut TabState,
    pub args: &'a Args,
    pub msg_area: Rect,
    pub msg_width: usize,
    pub theme: &'a RenderTheme,
    pub view_height: u16,
    pub mouse_x: u16,
    pub mouse_y: u16,
}

pub(crate) fn hit_test_edit_button(
    params: HitTestEditButtonParams<'_>,
) -> Option<usize> {
    if params.tab_state.app.message_layouts.is_empty() {
        return None;
    }
    let inner = inner_area(params.msg_area, PADDING_X, PADDING_Y);
    if !mouse_in_rect(params.mouse_x, params.mouse_y, inner) {
        return None;
    }
    let text = selection_view_text(
        params.tab_state,
        params.args,
        params.msg_width,
        params.theme,
        params.view_height,
    );
    let app = &params.tab_state.app;
    let (row, col) = chat_position_from_mouse(
        &text,
        app.scroll,
        inner,
        params.mouse_x,
        params.mouse_y,
    );
    find_edit_button_at(app, row, col)
}

fn mouse_in_rect(mouse_x: u16, mouse_y: u16, rect: Rect) -> bool {
    mouse_x >= rect.x
        && mouse_x < rect.x + rect.width
        && mouse_y >= rect.y
        && mouse_y < rect.y + rect.height
}

fn active_message<'a>(
    app: &mut crate::framework::widget_system::runtime::state::App,
    messages: &'a [DisplayMessage],
) -> Option<(usize, &'a crate::types::Message)> {
    let idx = select_visible_message(app, messages)?;
    messages
        .iter()
        .find(|msg| msg.index == idx)
        .map(|msg| (idx, &msg.message))
}

fn find_edit_button_at(app: &crate::framework::widget_system::runtime::state::App, row: usize, col: usize) -> Option<usize> {
    for layout in &app.message_layouts {
        if layout.label_line == row {
            return hit_test_layout_button(layout, col);
        }
    }
    None
}

fn hit_test_layout_button(layout: &crate::render::MessageLayout, col: usize) -> Option<usize> {
    let (start, end) = layout.button_range?;
    if col >= start && col < end {
        Some(layout.index)
    } else {
        None
    }
}
