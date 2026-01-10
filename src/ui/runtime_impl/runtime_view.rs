use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};
use crate::ui::overlay::{OverlayKind, OverlayState};
use crate::ui::runtime_view_handlers::{
    handle_help_key, handle_jump_key, handle_model_key, handle_prompt_key,
    handle_question_review_key, handle_summary_key,
};
use crate::ui::selection_state::SelectionState;
use crate::ui::summary::SummarySort;
pub(crate) struct ViewState {
    pub(crate) overlay: OverlayState,
    pub(crate) summary: SelectionState,
    pub(crate) summary_sort: SummarySort,
    pub(crate) summary_order: Vec<usize>,
    pub(crate) jump: SelectionState,
    pub(crate) model: SelectionState,
    pub(crate) prompt: SelectionState,
    pub(crate) question_review: SelectionState,
    pub(crate) question_review_detail_scroll: usize,
    pub(crate) help: SelectionState,
}
#[derive(Copy, Clone)]
pub(crate) enum ViewAction {
    None,
    SwitchTab(usize),
    JumpTo(usize),
    ForkMessage(usize),
    SelectModel(usize),
    CycleModel,
    SelectPrompt(usize),
    QuestionReviewToggle(usize), QuestionReviewApprove(usize), QuestionReviewReject(usize),
    QuestionReviewApproveAll, QuestionReviewRejectAll, QuestionReviewNextModel(usize),
    QuestionReviewSetAllModel(usize), QuestionReviewSubmit, QuestionReviewCancel,
}
pub(crate) fn apply_view_action(
    action: ViewAction,
    jump_rows: &[crate::ui::jump::JumpRow],
    tabs: &mut [crate::ui::runtime_helpers::TabState],
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
) -> bool {
    match action {
        ViewAction::SwitchTab(idx) => {
            apply_switch_tab(idx, tabs, active_tab, categories, active_category)
        }
        ViewAction::JumpTo(idx) => apply_jump_to(idx, jump_rows, tabs, *active_tab),
        ViewAction::ForkMessage(_) => false,
        ViewAction::SelectModel(_) | ViewAction::CycleModel | ViewAction::SelectPrompt(_) => false,
        ViewAction::QuestionReviewToggle(_) | ViewAction::QuestionReviewApprove(_)
        | ViewAction::QuestionReviewReject(_) | ViewAction::QuestionReviewApproveAll
        | ViewAction::QuestionReviewRejectAll | ViewAction::QuestionReviewNextModel(_)
        | ViewAction::QuestionReviewSetAllModel(_) | ViewAction::QuestionReviewSubmit
        | ViewAction::QuestionReviewCancel => false,
        ViewAction::None => false,
    }
}
impl ViewState {
    pub(crate) fn new() -> Self {
        Self {
            overlay: OverlayState::default(),
            summary: SelectionState::default(),
            summary_sort: SummarySort::TabOrder,
            summary_order: Vec::new(),
            jump: SelectionState::default(),
            model: SelectionState::default(),
            prompt: SelectionState::default(),
            question_review: SelectionState::default(),
            question_review_detail_scroll: 0,
            help: SelectionState::default(),
        }
    }

    pub(crate) fn is_chat(&self) -> bool {
        self.overlay.is_chat()
    }

    fn open_summary(&mut self, active_tab: usize, tabs_len: usize) {
        self.summary.selected = active_tab.min(tabs_len.saturating_sub(1));
        self.overlay.open(OverlayKind::Summary);
    }

    fn open_jump(&mut self) {
        self.jump = SelectionState::default();
        self.overlay.open(OverlayKind::Jump);
    }

    fn open_prompt(&mut self) {
        self.prompt.scroll = 0;
        self.overlay.open(OverlayKind::Prompt);
    }

    fn open_help(&mut self) {
        self.help.scroll = 0;
        self.overlay.open(OverlayKind::Help);
    }
}
pub(crate) fn handle_view_key(
    view: &mut ViewState,
    key: KeyEvent,
    tabs_len: usize,
    jump_len: usize,
    active_tab: usize,
) -> ViewAction {
    if let Some(action) = handle_function_keys(view, key, tabs_len, jump_len, active_tab) {
        return action;
    }
    match view.overlay.active {
        None => ViewAction::None,
        Some(OverlayKind::Summary) => handle_summary_key(view, key, tabs_len),
        Some(OverlayKind::Jump) => handle_jump_key(view, key, jump_len),
        Some(OverlayKind::Model) => handle_model_key(view, key),
        Some(OverlayKind::Prompt) => handle_prompt_key(view, key),
        Some(OverlayKind::QuestionReview) => handle_question_review_key(view, key),
        Some(OverlayKind::CodeExec | OverlayKind::FilePatch) => ViewAction::None,
        Some(OverlayKind::Terminal) => handle_terminal_key(view, key),
        Some(OverlayKind::Help) => handle_help_key(view, key),
    }
}
pub(crate) fn handle_view_mouse(
    view: &mut ViewState,
    row: Option<usize>,
    tabs_len: usize,
    jump_len: usize,
    kind: MouseEventKind,
) -> ViewAction {
    let Some(row) = row else {
        return ViewAction::None;
    };
    match view.overlay.active {
        Some(OverlayKind::Summary) => handle_summary_mouse(view, row, tabs_len, kind),
        Some(OverlayKind::Jump) => handle_jump_mouse(view, row, jump_len, kind),
        Some(OverlayKind::Model) => handle_model_mouse(view, row, kind),
        Some(OverlayKind::Prompt) => handle_prompt_mouse(view, row, kind),
        Some(OverlayKind::QuestionReview) => handle_question_review_mouse(view, row, kind),
        Some(OverlayKind::Help) => handle_help_mouse(view, row, kind),
        Some(OverlayKind::CodeExec | OverlayKind::FilePatch | OverlayKind::Terminal) | None => {
            ViewAction::None
        }
    }
}
fn handle_function_keys(
    view: &mut ViewState,
    key: KeyEvent,
    tabs_len: usize,
    _jump_len: usize,
    active_tab: usize,
) -> Option<ViewAction> {
    match key.code {
        KeyCode::F(3) => Some(ViewAction::CycleModel),
        KeyCode::F(4) => Some(toggle_overlay(view, OverlayKind::Model)),
        KeyCode::F(10) => Some(toggle_help(view)),
        KeyCode::F(1) => Some(toggle_summary(view, active_tab, tabs_len)),
        KeyCode::F(2) => Some(toggle_jump(view)),
        KeyCode::F(5) => Some(toggle_prompt(view)),
        KeyCode::F(7) => Some(toggle_overlay(view, OverlayKind::Terminal)),
        _ => None,
    }
}
fn handle_terminal_key(_view: &mut ViewState, _key: KeyEvent) -> ViewAction {
    ViewAction::None
}
fn toggle_overlay(view: &mut ViewState, kind: OverlayKind) -> ViewAction {
    view.overlay.toggle(kind);
    ViewAction::None
}
fn toggle_help(view: &mut ViewState) -> ViewAction {
    if view.overlay.is(OverlayKind::Help) {
        view.overlay.close();
    } else {
        view.open_help();
    }
    ViewAction::None
}
fn toggle_summary(view: &mut ViewState, active_tab: usize, tabs_len: usize) -> ViewAction {
    if view.overlay.is(OverlayKind::Summary) {
        view.overlay.close();
    } else {
        view.open_summary(active_tab, tabs_len);
    }
    ViewAction::None
}
fn toggle_jump(view: &mut ViewState) -> ViewAction {
    if view.overlay.is(OverlayKind::Jump) {
        view.overlay.close();
    } else {
        view.open_jump();
    }
    ViewAction::None
}
fn toggle_prompt(view: &mut ViewState) -> ViewAction {
    if view.overlay.is(OverlayKind::Prompt) {
        view.overlay.close();
    } else {
        view.open_prompt();
    }
    ViewAction::None
}
fn apply_switch_tab(
    idx: usize,
    tabs: &mut [crate::ui::runtime_helpers::TabState],
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
) -> bool {
    *active_tab = idx;
    if let Some(tab) = tabs.get(*active_tab) {
        if let Some(cat_idx) = categories.iter().position(|c| c == &tab.category) {
            *active_category = cat_idx;
        } else {
            categories.push(tab.category.clone());
            *active_category = categories.len().saturating_sub(1);
        }
    }
    true
}
fn apply_jump_to(
    idx: usize,
    jump_rows: &[crate::ui::jump::JumpRow],
    tabs: &mut [crate::ui::runtime_helpers::TabState],
    active_tab: usize,
) -> bool {
    if let Some(row) = jump_rows.get(idx)
        && let Some(tab_state) = tabs.get_mut(active_tab)
    {
        let app = &mut tab_state.app;
        app.scroll = row.scroll;
        app.follow = false;
        app.focus = crate::ui::state::Focus::Chat;
    }
    true
}
fn handle_summary_mouse(
    view: &mut ViewState,
    row: usize,
    tabs_len: usize,
    kind: MouseEventKind,
) -> ViewAction {
    view.summary.select(row.min(tabs_len.saturating_sub(1)));
    if matches!(kind, MouseEventKind::Down(_)) && row < tabs_len {
        view.overlay.close();
        return ViewAction::SwitchTab(row);
    }
    ViewAction::None
}
fn handle_jump_mouse(
    view: &mut ViewState,
    row: usize,
    jump_len: usize,
    kind: MouseEventKind,
) -> ViewAction {
    view.jump.select(row.min(jump_len.saturating_sub(1)));
    view.jump.ensure_visible(1);
    if matches!(kind, MouseEventKind::Down(_)) && row < jump_len {
        view.overlay.close();
        return ViewAction::JumpTo(row);
    }
    ViewAction::None
}
fn handle_model_mouse(view: &mut ViewState, row: usize, kind: MouseEventKind) -> ViewAction {
    view.model.select(row);
    if matches!(kind, MouseEventKind::Down(_)) {
        view.overlay.close();
        return ViewAction::SelectModel(row);
    }
    ViewAction::None
}
fn handle_prompt_mouse(view: &mut ViewState, row: usize, kind: MouseEventKind) -> ViewAction {
    view.prompt.select(row);
    if matches!(kind, MouseEventKind::Down(_)) {
        view.overlay.close();
        return ViewAction::SelectPrompt(row);
    }
    ViewAction::None
}
fn handle_question_review_mouse(
    view: &mut ViewState,
    row: usize,
    kind: MouseEventKind,
) -> ViewAction {
    if matches!(kind, MouseEventKind::Down(_)) {
        view.question_review.select(row);
        view.question_review_detail_scroll = 0;
    }
    ViewAction::None
}
fn handle_help_mouse(view: &mut ViewState, row: usize, kind: MouseEventKind) -> ViewAction {
    if matches!(kind, MouseEventKind::Moved) {
        view.help.select(row);
    }
    ViewAction::None
}
