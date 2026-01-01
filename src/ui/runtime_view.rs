use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};

use crate::ui::overlay::{OverlayKind, OverlayState};
use crate::ui::selection_state::SelectionState;

const JUMP_PAGE_STEP: usize = 5;

pub(crate) struct ViewState {
    pub(crate) overlay: OverlayState,
    pub(crate) summary: SelectionState,
    pub(crate) jump: SelectionState,
    pub(crate) model: SelectionState,
    pub(crate) prompt: SelectionState,
}

pub(crate) enum ViewAction {
    None,
    SwitchTab(usize),
    JumpTo(usize),
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
        ViewAction::SelectModel(_) | ViewAction::CycleModel | ViewAction::SelectPrompt(_) => false,
        ViewAction::None => false,
    }
}

impl ViewState {
    pub(crate) fn new() -> Self {
        Self {
            overlay: OverlayState::default(),
            summary: SelectionState::default(),
            jump: SelectionState::default(),
            model: SelectionState::default(),
            prompt: SelectionState::default(),
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
        None => {}
    }
    ViewAction::None
}

fn handle_summary_key(view: &mut ViewState, key: KeyEvent, tabs_len: usize) -> ViewAction {
    match key.code {
        KeyCode::Esc => {
            view.overlay.close();
            ViewAction::None
        }
        KeyCode::Up => {
            view.summary.move_up();
            ViewAction::None
        }
        KeyCode::Down => {
            view.summary.move_down();
            ViewAction::None
        }
        KeyCode::Enter => {
            if view.summary.selected < tabs_len {
                let idx = view.summary.selected;
                view.overlay.close();
                ViewAction::SwitchTab(idx)
            } else {
                ViewAction::None
            }
        }
        _ => ViewAction::None,
    }
}

fn handle_jump_key(view: &mut ViewState, key: KeyEvent, jump_len: usize) -> ViewAction {
    match key.code {
        KeyCode::Esc => {
            view.overlay.close();
            ViewAction::None
        }
        KeyCode::Up => {
            view.jump.move_up();
            ViewAction::None
        }
        KeyCode::Down => {
            view.jump.move_down();
            ViewAction::None
        }
        KeyCode::PageUp => {
            view.jump.page_up(JUMP_PAGE_STEP);
            ViewAction::None
        }
        KeyCode::PageDown => {
            view.jump.page_down(JUMP_PAGE_STEP);
            ViewAction::None
        }
        KeyCode::Enter => {
            if view.jump.selected < jump_len {
                let idx = view.jump.selected;
                view.overlay.close();
                ViewAction::JumpTo(idx)
            } else {
                ViewAction::None
            }
        }
        _ => ViewAction::None,
    }
}

fn handle_model_key(view: &mut ViewState, key: KeyEvent) -> ViewAction {
    match key.code {
        KeyCode::Esc => {
            view.overlay.close();
            ViewAction::None
        }
        KeyCode::Up => {
            view.model.move_up();
            ViewAction::None
        }
        KeyCode::Down => {
            view.model.move_down();
            ViewAction::None
        }
        KeyCode::Enter => {
            view.overlay.close();
            ViewAction::SelectModel(view.model.selected)
        }
        _ => ViewAction::None,
    }
}

fn handle_prompt_key(view: &mut ViewState, key: KeyEvent) -> ViewAction {
    match key.code {
        KeyCode::Esc => {
            view.overlay.close();
            ViewAction::None
        }
        KeyCode::Up => {
            view.prompt.move_up();
            ViewAction::None
        }
        KeyCode::Down => {
            view.prompt.move_down();
            ViewAction::None
        }
        KeyCode::Enter => {
            view.overlay.close();
            ViewAction::SelectPrompt(view.prompt.selected)
        }
        _ => ViewAction::None,
    }
}
