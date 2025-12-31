use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ViewMode {
    Chat,
    Summary,
    Jump,
    Model,
}

pub(crate) struct ViewState {
    pub(crate) mode: ViewMode,
    pub(crate) summary_selected: usize,
    pub(crate) jump_selected: usize,
    pub(crate) jump_scroll: usize,
    pub(crate) model_selected: usize,
}

pub(crate) enum ViewAction {
    None,
    SwitchTab(usize),
    JumpTo(usize),
    SelectModel(usize),
    CycleModel,
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
        ViewAction::SelectModel(_) | ViewAction::CycleModel => false,
        ViewAction::None => false,
    }
}

impl ViewState {
    pub(crate) fn new() -> Self {
        Self {
            mode: ViewMode::Chat,
            summary_selected: 0,
            jump_selected: 0,
            jump_scroll: 0,
            model_selected: 0,
        }
    }

    pub(crate) fn is_chat(&self) -> bool {
        self.mode == ViewMode::Chat
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
        view.mode = if view.mode == ViewMode::Model {
            ViewMode::Chat
        } else {
            ViewMode::Model
        };
        return ViewAction::None;
    }

    match key.code {
        KeyCode::F(1) => {
            view.mode = if view.mode == ViewMode::Summary {
                ViewMode::Chat
            } else {
                view.summary_selected = active_tab.min(tabs_len.saturating_sub(1));
                ViewMode::Summary
            };
            return ViewAction::None;
        }
        KeyCode::F(2) => {
            view.mode = if view.mode == ViewMode::Jump {
                ViewMode::Chat
            } else {
                view.jump_selected = 0;
                view.jump_scroll = 0;
                ViewMode::Jump
            };
            return ViewAction::None;
        }
        _ => {}
    }

    match view.mode {
        ViewMode::Chat => ViewAction::None,
        ViewMode::Summary => match key.code {
            KeyCode::Esc => {
                view.mode = ViewMode::Chat;
                ViewAction::None
            }
            KeyCode::Up => {
                view.summary_selected = view.summary_selected.saturating_sub(1);
                ViewAction::None
            }
            KeyCode::Down => {
                view.summary_selected = view.summary_selected.saturating_add(1);
                ViewAction::None
            }
            KeyCode::Enter => {
                if view.summary_selected < tabs_len {
                    let idx = view.summary_selected;
                    view.mode = ViewMode::Chat;
                    ViewAction::SwitchTab(idx)
                } else {
                    ViewAction::None
                }
            }
            _ => ViewAction::None,
        },
        ViewMode::Jump => match key.code {
            KeyCode::Esc => {
                view.mode = ViewMode::Chat;
                ViewAction::None
            }
            KeyCode::Up => {
                view.jump_selected = view.jump_selected.saturating_sub(1);
                if view.jump_selected < view.jump_scroll {
                    view.jump_scroll = view.jump_selected;
                }
                ViewAction::None
            }
            KeyCode::Down => {
                view.jump_selected = view.jump_selected.saturating_add(1);
                ViewAction::None
            }
            KeyCode::PageUp => {
                view.jump_scroll = view.jump_scroll.saturating_sub(5);
                if view.jump_selected < view.jump_scroll {
                    view.jump_selected = view.jump_scroll;
                }
                ViewAction::None
            }
            KeyCode::PageDown => {
                view.jump_scroll = view.jump_scroll.saturating_add(5);
                if view.jump_selected < view.jump_scroll {
                    view.jump_selected = view.jump_scroll;
                }
                ViewAction::None
            }
            KeyCode::Enter => {
                if view.jump_selected < jump_len {
                    let idx = view.jump_selected;
                    view.mode = ViewMode::Chat;
                    ViewAction::JumpTo(idx)
                } else {
                    ViewAction::None
                }
            }
            _ => ViewAction::None,
        },
        ViewMode::Model => match key.code {
            KeyCode::Esc => {
                view.mode = ViewMode::Chat;
                ViewAction::None
            }
            KeyCode::Up => {
                view.model_selected = view.model_selected.saturating_sub(1);
                ViewAction::None
            }
            KeyCode::Down => {
                view.model_selected = view.model_selected.saturating_add(1);
                ViewAction::None
            }
            KeyCode::Enter => {
                view.mode = ViewMode::Chat;
                ViewAction::SelectModel(view.model_selected)
            }
            _ => ViewAction::None,
        },
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
    match view.mode {
        ViewMode::Summary => {
            view.summary_selected = row.min(tabs_len.saturating_sub(1));
            if matches!(kind, MouseEventKind::Down(_)) && row < tabs_len {
                view.mode = ViewMode::Chat;
                return ViewAction::SwitchTab(row);
            }
        }
        ViewMode::Jump => {
            view.jump_selected = row.min(jump_len.saturating_sub(1));
            if view.jump_selected < view.jump_scroll {
                view.jump_scroll = view.jump_selected;
            }
            if matches!(kind, MouseEventKind::Down(_)) && row < jump_len {
                view.mode = ViewMode::Chat;
                return ViewAction::JumpTo(row);
            }
        }
        ViewMode::Model => {
            view.model_selected = row;
            if matches!(kind, MouseEventKind::Down(_)) {
                view.mode = ViewMode::Chat;
                return ViewAction::SelectModel(row);
            }
        }
        ViewMode::Chat => {}
    }
    ViewAction::None
}
