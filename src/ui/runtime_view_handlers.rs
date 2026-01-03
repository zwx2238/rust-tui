use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::runtime_view::{ViewAction, ViewState};

const PAGE_STEP: usize = 5;

pub(crate) fn handle_summary_key(
    view: &mut ViewState,
    key: KeyEvent,
    tabs_len: usize,
) -> ViewAction {
    match key.code {
        KeyCode::Esc => close_overlay(view),
        KeyCode::Up => move_summary_up(view),
        KeyCode::Down => move_summary_down(view),
        KeyCode::Enter => handle_summary_enter(view, tabs_len),
        KeyCode::Char('s') | KeyCode::Char('S') => toggle_summary_sort(view),
        _ => ViewAction::None,
    }
}

pub(crate) fn handle_jump_key(view: &mut ViewState, key: KeyEvent, jump_len: usize) -> ViewAction {
    match key.code {
        KeyCode::Esc => close_overlay(view),
        KeyCode::Char('e') | KeyCode::Char('E') => handle_jump_fork(view, jump_len),
        KeyCode::Up => move_jump_up(view),
        KeyCode::Down => move_jump_down(view),
        KeyCode::PageUp => page_jump_up(view),
        KeyCode::PageDown => page_jump_down(view),
        KeyCode::Enter => handle_jump_enter(view, jump_len),
        _ => ViewAction::None,
    }
}

fn close_overlay(view: &mut ViewState) -> ViewAction {
    view.overlay.close();
    ViewAction::None
}

fn move_summary_up(view: &mut ViewState) -> ViewAction {
    view.summary.move_up();
    ViewAction::None
}

fn move_summary_down(view: &mut ViewState) -> ViewAction {
    view.summary.move_down();
    ViewAction::None
}

fn handle_summary_enter(view: &mut ViewState, tabs_len: usize) -> ViewAction {
    if view.summary.selected < tabs_len {
        let idx = view
            .summary_order
            .get(view.summary.selected)
            .copied()
            .unwrap_or(view.summary.selected);
        view.overlay.close();
        ViewAction::SwitchTab(idx)
    } else {
        ViewAction::None
    }
}

fn toggle_summary_sort(view: &mut ViewState) -> ViewAction {
    view.summary_sort = match view.summary_sort {
        crate::ui::summary::SummarySort::TabOrder => crate::ui::summary::SummarySort::ExecTime,
        crate::ui::summary::SummarySort::ExecTime => crate::ui::summary::SummarySort::TabOrder,
    };
    view.summary.selected = 0;
    view.summary.scroll = 0;
    ViewAction::None
}

fn handle_jump_fork(view: &mut ViewState, jump_len: usize) -> ViewAction {
    if view.jump.selected < jump_len {
        ViewAction::ForkMessage(view.jump.selected)
    } else {
        ViewAction::None
    }
}

fn move_jump_up(view: &mut ViewState) -> ViewAction {
    view.jump.move_up();
    ViewAction::None
}

fn move_jump_down(view: &mut ViewState) -> ViewAction {
    view.jump.move_down();
    ViewAction::None
}

fn page_jump_up(view: &mut ViewState) -> ViewAction {
    view.jump.page_up(PAGE_STEP);
    ViewAction::None
}

fn page_jump_down(view: &mut ViewState) -> ViewAction {
    view.jump.page_down(PAGE_STEP);
    ViewAction::None
}

fn handle_jump_enter(view: &mut ViewState, jump_len: usize) -> ViewAction {
    if view.jump.selected < jump_len {
        let idx = view.jump.selected;
        view.overlay.close();
        ViewAction::JumpTo(idx)
    } else {
        ViewAction::None
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
