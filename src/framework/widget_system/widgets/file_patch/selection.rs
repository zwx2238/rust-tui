use super::popup_layout::{FilePatchPopupLayout, file_patch_popup_layout};
use super::popup_text::patch_max_scroll;
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::interaction::selection::{Selection, chat_position_from_mouse, extract_selection};
use crate::framework::widget_system::runtime::state::FilePatchHover;

use super::helpers::point_in_rect;

pub(super) fn handle_file_patch_selection_start(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingFilePatch,
    popup: FilePatchPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.preview_area) {
        return false;
    }
    let (text, _) = super::popup_text::build_patch_text(
        &pending.preview,
        popup.preview_area.width,
        popup.preview_area.height,
        tab_state.app.file_patch_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.file_patch_scroll,
        popup.preview_area,
        m,
    );
    tab_state.app.file_patch_selecting = true;
    tab_state.app.file_patch_selection = Some(Selection {
        start: pos,
        end: pos,
    });
    true
}

pub(super) fn handle_file_patch_selection_drag(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingFilePatch,
    popup: FilePatchPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    if !tab_state.app.file_patch_selecting {
        return false;
    }
    let (text, _) = super::popup_text::build_patch_text(
        &pending.preview,
        popup.preview_area.width,
        popup.preview_area.height,
        tab_state.app.file_patch_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.file_patch_scroll,
        popup.preview_area,
        m,
    );
    update_drag_selection(tab_state, pos);
    true
}

pub(super) fn clear_file_patch_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
) -> bool {
    if !tab_state.app.file_patch_selecting {
        return false;
    }
    tab_state.app.file_patch_selecting = false;
    if tab_state
        .app
        .file_patch_selection
        .map(|sel| sel.is_empty())
        .unwrap_or(false)
    {
        tab_state.app.file_patch_selection = None;
    }
    true
}

pub(super) fn selection_position_for_panel(
    text: &ratatui::text::Text<'static>,
    scroll: usize,
    area: ratatui::layout::Rect,
    m: crossterm::event::MouseEvent,
) -> (usize, usize) {
    let scroll_u16 = scroll.min(u16::MAX as usize) as u16;
    chat_position_from_mouse(text, scroll_u16, area, m.column, m.row)
}

fn update_drag_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pos: (usize, usize),
) {
    let next = match tab_state.app.file_patch_selection {
        Some(existing) => Selection {
            start: existing.start,
            end: pos,
        },
        None => Selection {
            start: pos,
            end: pos,
        },
    };
    tab_state.app.file_patch_selection = Some(next);
}

pub(super) fn hover_at(
    m: crossterm::event::MouseEvent,
    popup: FilePatchPopupLayout,
) -> Option<FilePatchHover> {
    if point_in_rect(m.column, m.row, popup.apply_btn) {
        Some(FilePatchHover::Apply)
    } else if point_in_rect(m.column, m.row, popup.cancel_btn) {
        Some(FilePatchHover::Cancel)
    } else {
        None
    }
}

pub(super) fn clamp_patch_scroll(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingFilePatch,
    layout: FilePatchPopupLayout,
) {
    let max_scroll = patch_max_scroll(
        &pending.preview,
        layout.preview_area.width,
        layout.preview_area.height,
        theme,
    );
    if tab_state.app.file_patch_scroll > max_scroll {
        tab_state.app.file_patch_scroll = max_scroll;
    }
}

pub(super) fn copy_file_patch_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingFilePatch,
    layout: &FrameLayout,
    theme: &crate::render::RenderTheme,
) -> bool {
    let Some(selection) = tab_state.app.file_patch_selection else {
        return false;
    };
    let popup = file_patch_popup_layout(layout.size);
    let lines = super::popup_text::patch_plain_lines(
        &pending.preview,
        popup.preview_area.width,
        theme,
    );
    let text = extract_selection(&lines, selection);
    if !text.is_empty() {
        crate::framework::widget_system::interaction::clipboard::set(&text);
    }
    true
}
