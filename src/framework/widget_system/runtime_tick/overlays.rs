use crate::framework::widget_system::overlay::OverlayKind;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::runtime::runtime_view::ViewState;

pub fn sync_code_exec_overlay(tabs: &mut [TabState], active_tab: usize, view: &mut ViewState) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let has_pending = tab_state.app.pending_code_exec.is_some();
        if has_pending && view.overlay.is_chat() {
            view.overlay.open(OverlayKind::CodeExec);
        } else if !has_pending && view.overlay.is(OverlayKind::CodeExec) {
            view.overlay.close();
        }
    }
}

pub fn sync_file_patch_overlay(tabs: &mut [TabState], active_tab: usize, view: &mut ViewState) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let has_pending = tab_state.app.pending_file_patch.is_some();
        if has_pending && view.overlay.is_chat() {
            view.overlay.open(OverlayKind::FilePatch);
        } else if !has_pending && view.overlay.is(OverlayKind::FilePatch) {
            view.overlay.close();
        }
    }
}

pub fn sync_question_review_overlay(
    tabs: &mut [TabState],
    active_tab: usize,
    view: &mut ViewState,
) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let has_pending = tab_state.app.pending_question_review.is_some();
        if has_pending && view.overlay.is_chat() {
            view.overlay.open(OverlayKind::QuestionReview);
            view.question_review_detail_scroll = 0;
        } else if !has_pending && view.overlay.is(OverlayKind::QuestionReview) {
            view.overlay.close();
        }
    }
}
