use crate::framework::widget_system::runtime::state::{App, Focus};
use crate::framework::widget_system::runtime_dispatch::DispatchContext;
use crate::framework::widget_system::runtime_dispatch::tabs::{next_tab, prev_tab};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) fn handle_nav_key(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if !is_nav_mode(ctx) {
        return try_enter_nav_mode(ctx, key);
    }
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => nav_prev_tab(ctx),
        KeyCode::Right | KeyCode::Char('l') => nav_next_tab(ctx),
        _ => handle_nav_key_in_mode(ctx, key),
    }
}

fn is_nav_mode(ctx: &DispatchContext<'_>) -> bool {
    ctx.tabs
        .get(*ctx.active_tab)
        .map(|tab| tab.app.nav_mode)
        .unwrap_or(false)
}

fn try_enter_nav_mode(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    let Some(app) = ctx.tabs.get_mut(*ctx.active_tab).map(|tab| &mut tab.app) else {
        return false;
    };
    if app.focus == Focus::Chat
        && key.modifiers == KeyModifiers::NONE
        && key.code == KeyCode::Char('g')
    {
        app.nav_mode = true;
        app.focus = Focus::Chat;
        app.follow = false;
        return true;
    }
    false
}

fn handle_nav_key_in_mode(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    let Some(app) = ctx.tabs.get_mut(*ctx.active_tab).map(|tab| &mut tab.app) else {
        return true;
    };
    match key.code {
        KeyCode::Esc | KeyCode::Char('g') => {
            app.nav_mode = false;
            true
        }
        KeyCode::Char('j') | KeyCode::Char('n') | KeyCode::Down => {
            nav_next(app);
            true
        }
        KeyCode::Char('k') | KeyCode::Char('p') | KeyCode::Up => {
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

fn nav_prev_tab(ctx: &mut DispatchContext<'_>) -> bool {
    prev_tab(ctx);
    ensure_nav_mode(ctx);
    true
}

fn nav_next_tab(ctx: &mut DispatchContext<'_>) -> bool {
    next_tab(ctx);
    ensure_nav_mode(ctx);
    true
}

fn ensure_nav_mode(ctx: &mut DispatchContext<'_>) {
    if let Some(app) = ctx.tabs.get_mut(*ctx.active_tab).map(|tab| &mut tab.app) {
        app.nav_mode = true;
        app.focus = Focus::Chat;
        app.follow = false;
    }
}
