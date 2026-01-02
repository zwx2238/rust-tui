use crate::render::{RenderTheme, messages_to_viewport_text_cached};
use crate::ui::draw::inner_area;
use crate::ui::draw::layout::{PADDING_X, PADDING_Y};
use crate::ui::logic::{build_label_suffixes, timer_text};
use crate::ui::runtime_helpers::TabState;
use crate::ui::selection::chat_position_from_mouse;
use ratatui::layout::Rect;

pub(crate) fn selection_view_text(
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

pub(crate) fn hit_test_edit_button(
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
