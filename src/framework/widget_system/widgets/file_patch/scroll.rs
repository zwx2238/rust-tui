use crate::ui::file_patch_popup_layout::FilePatchPopupLayout;
use crate::ui::file_patch_popup_text::patch_max_scroll;

use super::helpers::{apply_scroll, point_in_rect};

pub(super) fn handle_file_patch_scroll(
    m: crossterm::event::MouseEvent,
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    popup: FilePatchPopupLayout,
    delta: i32,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    let max_scroll = patch_max_scroll(
        &pending.preview,
        popup.preview_area.width,
        popup.preview_area.height,
        theme,
    );
    apply_scroll(&mut tab_state.app.file_patch_scroll, delta, max_scroll);
    true
}
