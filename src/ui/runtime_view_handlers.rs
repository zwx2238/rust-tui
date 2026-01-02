use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::runtime_view::{ViewAction, ViewState};

const PAGE_STEP: usize = 5;

pub(crate) fn handle_summary_key(view: &mut ViewState, key: KeyEvent, tabs_len: usize) -> ViewAction {
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

pub(crate) fn handle_jump_key(view: &mut ViewState, key: KeyEvent, jump_len: usize) -> ViewAction {
    match key.code {
        KeyCode::Esc => {
            view.overlay.close();
            ViewAction::None
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if view.jump.selected < jump_len {
                ViewAction::ForkMessage(view.jump.selected)
            } else {
                ViewAction::None
            }
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
            view.jump.page_up(PAGE_STEP);
            ViewAction::None
        }
        KeyCode::PageDown => {
            view.jump.page_down(PAGE_STEP);
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

pub(crate) fn handle_model_key(view: &mut ViewState, key: KeyEvent) -> ViewAction {
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

pub(crate) fn handle_prompt_key(view: &mut ViewState, key: KeyEvent) -> ViewAction {
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

pub(crate) fn handle_help_key(view: &mut ViewState, key: KeyEvent) -> ViewAction {
    match key.code {
        KeyCode::Esc => {
            view.overlay.close();
            ViewAction::None
        }
        KeyCode::Up => {
            view.help.move_up();
            ViewAction::None
        }
        KeyCode::Down => {
            view.help.move_down();
            ViewAction::None
        }
        KeyCode::PageUp => {
            view.help.page_up(PAGE_STEP);
            ViewAction::None
        }
        KeyCode::PageDown => {
            view.help.page_down(PAGE_STEP);
            ViewAction::None
        }
        _ => ViewAction::None,
    }
}
