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
        crate::render::ViewportRenderParams {
            messages: &app.messages,
            width: msg_width,
            theme,
            label_suffixes: &label_suffixes,
            streaming_idx: app.pending_assistant,
            scroll: app.scroll,
            height: view_height,
        },
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
    if !mouse_in_rect(mouse_x, mouse_y, inner) {
        return None;
    }
    let text = selection_view_text(tab_state, msg_width, theme, view_height);
    let app = &tab_state.app;
    let (row, col) = chat_position_from_mouse(&text, app.scroll, inner, mouse_x, mouse_y);
    find_edit_button_at(app, row, col)
}

fn mouse_in_rect(mouse_x: u16, mouse_y: u16, rect: Rect) -> bool {
    mouse_x >= rect.x
        && mouse_x < rect.x + rect.width
        && mouse_y >= rect.y
        && mouse_y < rect.y + rect.height
}

fn find_edit_button_at(app: &crate::ui::state::App, row: usize, col: usize) -> Option<usize> {
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
