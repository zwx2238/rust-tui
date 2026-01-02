use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};

use crate::ui::overlay::{OverlayKind, OverlayState};
use crate::ui::selection_state::SelectionState;
use crate::ui::summary::SummarySort;
use crate::ui::runtime_view_handlers::{
    handle_help_key, handle_jump_key, handle_model_key, handle_prompt_key, handle_summary_key,
};

pub(crate) struct ViewState {
    pub(crate) overlay: OverlayState,
    pub(crate) summary: SelectionState,
    pub(crate) summary_sort: SummarySort,
    pub(crate) summary_order: Vec<usize>,
    pub(crate) jump: SelectionState,
    pub(crate) model: SelectionState,
    pub(crate) prompt: SelectionState,
    pub(crate) help: SelectionState,
}

pub(crate) enum ViewAction {
    None,
    SwitchTab(usize),
    JumpTo(usize),
    ForkMessage(usize),
    SelectModel(usize),
    CycleModel,
    SelectPrompt(usize),
}

pub(crate) fn apply_view_action(
    action: ViewAction,
    jump_rows: &[crate::ui::jump::JumpRow],
    tabs: &mut Vec<crate::ui::runtime_helpers::TabState>,
    active_tab: &mut usize,
) -> bool {
    match action {
        ViewAction::SwitchTab(idx) => {
            *active_tab = idx;
            true
        }
        ViewAction::JumpTo(idx) => {
            if let Some(row) = jump_rows.get(idx) {
                if let Some(tab_state) = tabs.get_mut(*active_tab) {
                    let app = &mut tab_state.app;
                    app.scroll = row.scroll;
                    app.follow = false;
                    app.focus = crate::ui::state::Focus::Chat;
                }
            }
            true
        }
        ViewAction::ForkMessage(_) => false,
        ViewAction::SelectModel(_) | ViewAction::CycleModel | ViewAction::SelectPrompt(_) => false,
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
    if key.code == KeyCode::F(3) {
        return ViewAction::CycleModel;
    }
    if key.code == KeyCode::F(4) {
        view.overlay.toggle(OverlayKind::Model);
        return ViewAction::None;
    }
    if key.code == KeyCode::F(10) {
        if view.overlay.is(OverlayKind::Help) {
            view.overlay.close();
        } else {
            view.open_help();
        }
        return ViewAction::None;
    }

    match key.code {
        KeyCode::F(1) => {
            if view.overlay.is(OverlayKind::Summary) {
                view.overlay.close();
            } else {
                view.open_summary(active_tab, tabs_len);
            }
            return ViewAction::None;
        }
        KeyCode::F(2) => {
            if view.overlay.is(OverlayKind::Jump) {
                view.overlay.close();
            } else {
                view.open_jump();
            }
            return ViewAction::None;
        }
        KeyCode::F(5) => {
            if view.overlay.is(OverlayKind::Prompt) {
                view.overlay.close();
            } else {
                view.open_prompt();
            }
            return ViewAction::None;
        }
        _ => {}
    }

    match view.overlay.active {
        None => ViewAction::None,
        Some(OverlayKind::Summary) => handle_summary_key(view, key, tabs_len),
        Some(OverlayKind::Jump) => handle_jump_key(view, key, jump_len),
        Some(OverlayKind::Model) => handle_model_key(view, key),
        Some(OverlayKind::Prompt) => handle_prompt_key(view, key),
        Some(OverlayKind::CodeExec) => ViewAction::None,
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
        Some(OverlayKind::Summary) => {
            view.summary.select(row.min(tabs_len.saturating_sub(1)));
            if matches!(kind, MouseEventKind::Down(_)) && row < tabs_len {
                view.overlay.close();
                return ViewAction::SwitchTab(row);
            }
        }
        Some(OverlayKind::Jump) => {
            view.jump.select(row.min(jump_len.saturating_sub(1)));
            view.jump.ensure_visible(1);
            if matches!(kind, MouseEventKind::Down(_)) && row < jump_len {
                view.overlay.close();
                return ViewAction::JumpTo(row);
            }
        }
        Some(OverlayKind::Model) => {
            view.model.select(row);
            if matches!(kind, MouseEventKind::Down(_)) {
                view.overlay.close();
                return ViewAction::SelectModel(row);
            }
        }
        Some(OverlayKind::Prompt) => {
            view.prompt.select(row);
            if matches!(kind, MouseEventKind::Down(_)) {
                view.overlay.close();
                return ViewAction::SelectPrompt(row);
            }
        }
        Some(OverlayKind::Help) => {
            view.help.select(row);
            view.help.ensure_visible(1);
            if matches!(kind, MouseEventKind::Down(_)) {
                view.overlay.close();
                return ViewAction::None;
            }
        }
        Some(OverlayKind::CodeExec) => {}
        None => {}
    }
    ViewAction::None
}
