use crate::framework::widget_system::runtime::state::{App, Focus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) fn handle_nav_key(app: &mut App, key: KeyEvent) -> bool {
    if !app.nav_mode {
        if app.focus == Focus::Chat
            && key.modifiers == KeyModifiers::NONE
            && key.code == KeyCode::Char('g')
        {
            app.nav_mode = true;
            app.focus = Focus::Chat;
            app.follow = false;
            return true;
        }
        return false;
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('g') => {
            app.nav_mode = false;
            true
        }
        KeyCode::Char('j') | KeyCode::Char('n') => {
            nav_next(app);
            true
        }
        KeyCode::Char('k') | KeyCode::Char('p') => {
            nav_prev(app);
            true
        }
        _ => true,
    }
}

fn nav_next(app: &mut App) {
    if app.messages.is_empty() {
        return;
    }
    let next = app.message_history.selected.saturating_add(1);
    if next >= app.messages.len() {
        return;
    }
    app.message_history.selected = next;
    app.scroll = 0;
    app.follow = false;
    app.focus = Focus::Chat;
    app.chat_selection = None;
    app.chat_selecting = false;
}

fn nav_prev(app: &mut App) {
    if app.messages.is_empty() {
        return;
    }
    if app.message_history.selected == 0 {
        return;
    }
    app.message_history.selected = app.message_history.selected.saturating_sub(1);
    app.scroll = 0;
    app.follow = false;
    app.focus = Focus::Chat;
    app.chat_selection = None;
    app.chat_selecting = false;
}
