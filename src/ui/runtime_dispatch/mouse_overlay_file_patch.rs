use crate::ui::file_patch_popup_layout::file_patch_popup_layout;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::ViewState;
use crate::ui::state::{FilePatchHover, PendingCommand};
use crossterm::event::MouseEvent;

use super::{apply_scroll, is_mouse_down, is_mouse_moved, point_in_rect, scroll_delta};

pub(crate) fn handle_file_patch_overlay_mouse(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
) -> bool {
    let theme = ctx.theme;
    let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) else {
        return handle_file_patch_fallback(m, ctx, layout, view);
    };
    let Some(pending) = tab_state.app.pending_file_patch.clone() else {
        return handle_file_patch_fallback(m, ctx, layout, view);
    };
    let popup = file_patch_popup_layout(layout.size);
    if handle_file_patch_popup_mouse(m, theme, view, tab_state, &pending, popup) {
        return true;
    }
    handle_file_patch_fallback(m, ctx, layout, view)
}

fn handle_file_patch_popup_mouse(
    m: MouseEvent,
    theme: &crate::render::RenderTheme,
    view: &mut ViewState,
    tab_state: &mut TabState,
    pending: &crate::ui::state::PendingFilePatch,
    popup: crate::ui::file_patch_popup_layout::FilePatchPopupLayout,
) -> bool {
    if handle_file_patch_hover(m, tab_state, popup) {
        return true;
    }
    if handle_file_patch_scroll(m, theme, tab_state, pending, popup) {
        return true;
    }
    if handle_file_patch_click(m, tab_state, view, popup) {
        return true;
    }
    false
}

fn handle_file_patch_hover(
    m: MouseEvent,
    tab_state: &mut TabState,
    popup: crate::ui::file_patch_popup_layout::FilePatchPopupLayout,
) -> bool {
    if !is_mouse_moved(m.kind) {
        return false;
    }
    tab_state.app.file_patch_hover = if point_in_rect(m.column, m.row, popup.apply_btn) {
        Some(FilePatchHover::Apply)
    } else if point_in_rect(m.column, m.row, popup.cancel_btn) {
        Some(FilePatchHover::Cancel)
    } else {
        None
    };
    true
}

fn handle_file_patch_scroll(
    m: MouseEvent,
    theme: &crate::render::RenderTheme,
    tab_state: &mut TabState,
    pending: &crate::ui::state::PendingFilePatch,
    popup: crate::ui::file_patch_popup_layout::FilePatchPopupLayout,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    let Some(delta) = scroll_delta(m.kind) else {
        return false;
    };
    let max_scroll = patch_max_scroll(
        &pending.preview,
        popup.preview_area.width,
        popup.preview_area.height,
        theme,
    );
    apply_scroll(&mut tab_state.app.file_patch_scroll, delta, max_scroll);
    true
}

fn handle_file_patch_click(
    m: MouseEvent,
    tab_state: &mut TabState,
    view: &mut ViewState,
    popup: crate::ui::file_patch_popup_layout::FilePatchPopupLayout,
) -> bool {
    if !is_mouse_down(m.kind) {
        return false;
    }
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    if point_in_rect(m.column, m.row, popup.apply_btn) {
        tab_state.app.pending_command = Some(PendingCommand::ApplyFilePatch);
        tab_state.app.file_patch_hover = None;
        view.overlay.close();
        return true;
    }
    if point_in_rect(m.column, m.row, popup.cancel_btn) {
        tab_state.app.pending_command = Some(PendingCommand::CancelFilePatch);
        tab_state.app.file_patch_hover = None;
        view.overlay.close();
        return true;
    }
    false
}

fn handle_file_patch_fallback(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
) -> bool {
    if is_mouse_down(m.kind) {
        if point_in_rect(m.column, m.row, layout.tabs_area)
            || point_in_rect(m.column, m.row, layout.category_area)
        {
            return false;
        }
        view.overlay.close();
    }
    if is_mouse_moved(m.kind)
        && let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
    {
        tab_state.app.file_patch_hover = None;
    }
    true
}
