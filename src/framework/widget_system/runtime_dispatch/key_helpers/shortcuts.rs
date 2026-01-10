use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::tabs::{
    close_all_tabs, close_other_tabs, close_tab, new_tab, next_category, next_tab, prev_category,
    prev_tab,
};
use crate::framework::widget_system::runtime_dispatch::DispatchContext;

pub(crate) fn is_quit_key(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q')
}

pub(crate) fn handle_global_shortcuts(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if handle_ctrl_shortcuts(ctx, key) {
        return true;
    }
    handle_function_tab_shortcuts(ctx, key)
}

fn handle_ctrl_shortcuts(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    if handle_ctrl_category(ctx, key) || handle_ctrl_tabs(ctx, key) {
        return true;
    }
    handle_ctrl_tab_actions(ctx, key)
}

fn handle_ctrl_category(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Up => {
            prev_category(ctx);
            true
        }
        KeyCode::Down => {
            next_category(ctx);
            true
        }
        _ => false,
    }
}

fn handle_ctrl_tabs(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Char('w') {
        close_all_tabs(ctx);
        return true;
    }
    false
}

fn handle_ctrl_tab_actions(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('o') => {
            close_other_tabs(ctx);
            true
        }
        KeyCode::Char('t') => {
            new_tab(ctx);
            true
        }
        KeyCode::Char('w') => {
            close_tab(ctx);
            true
        }
        _ => false,
    }
}

fn handle_function_tab_shortcuts(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::F(8) => {
            prev_tab(ctx);
            true
        }
        KeyCode::F(9) => {
            next_tab(ctx);
            true
        }
        _ => false,
    }
}
