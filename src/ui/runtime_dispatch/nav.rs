use crate::ui::state::{App, Focus};
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
    if app.message_layouts.is_empty() {
        return;
    }
    let current = app.scroll as usize;
    let mut target = None;
    for layout in &app.message_layouts {
        if layout.label_line > current {
            target = Some(layout.label_line);
            break;
        }
    }
    if let Some(line) = target {
        app.scroll = line.min(u16::MAX as usize) as u16;
        app.follow = false;
    }
}

fn nav_prev(app: &mut App) {
    if app.message_layouts.is_empty() {
        return;
    }
    let current = app.scroll as usize;
    let mut target = None;
    for layout in app.message_layouts.iter().rev() {
        if layout.label_line < current {
            target = Some(layout.label_line);
            break;
        }
    }
    if let Some(line) = target {
        app.scroll = line.min(u16::MAX as usize) as u16;
        app.follow = false;
    }
}
