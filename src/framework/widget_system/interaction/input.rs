use crate::framework::widget_system::interaction::clipboard;
use crate::framework::widget_system::commands::command_input::handle_command_line;
use crate::framework::widget_system::commands::command_suggestions::{
    apply_command_suggestion, command_suggestions_active, refresh_command_suggestions,
};
use crate::framework::widget_system::runtime::state::{App, Focus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

pub fn handle_key(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    if app.pending_code_exec.is_some() || app.pending_file_patch.is_some() {
        return Ok(false);
    }
    if key.code == KeyCode::Esc {
        return Ok(true);
    }

    if is_input_ready(app) {
        return handle_input_key(key, app);
    }

    handle_scroll_key(key, app);
    Ok(false)
}

fn is_input_ready(app: &App) -> bool {
    app.focus == Focus::Input && !app.busy
}

fn handle_input_key(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    if handle_ctrl_shortcuts(key, app) {
        return Ok(false);
    }
    if handle_command_suggestion_nav(key, app) {
        return Ok(false);
    }
    handle_input_editing(key, app)
}

fn handle_ctrl_shortcuts(key: KeyEvent, app: &mut App) -> bool {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    match key.code {
        KeyCode::Char('a') => {
            app.input.select_all();
            true
        }
        KeyCode::Char('c') => {
            copy_selection(app);
            true
        }
        KeyCode::Char('x') => {
            cut_selection(app);
            true
        }
        KeyCode::Char('v') => {
            paste_clipboard(app);
            true
        }
        _ => false,
    }
}

fn copy_selection(app: &mut App) {
    if app.input.is_selecting() {
        app.input.copy();
        let text = app.input.yank_text();
        clipboard::set(&text);
    }
}

fn cut_selection(app: &mut App) {
    if app.input.is_selecting() && app.input.cut() {
        let text = app.input.yank_text();
        clipboard::set(&text);
    }
    refresh_command_suggestions(app);
}

fn paste_clipboard(app: &mut App) {
    if let Some(text) = clipboard::get() {
        app.input.set_yank_text(text);
        app.input.paste();
    } else {
        maybe_notice_clipboard_unavailable(app);
    }
    refresh_command_suggestions(app);
}

fn maybe_notice_clipboard_unavailable(app: &mut App) {
    if !is_ssh() {
        return;
    }
    crate::framework::widget_system::notice::push_notice(
        app,
        "无法读取系统剪贴板（可能是 SSH/无图形环境）。请使用终端粘贴：Ctrl+Shift+V 或 Shift+Insert",
    );
}

fn is_ssh() -> bool {
    std::env::var_os("SSH_CONNECTION").is_some() || std::env::var_os("SSH_TTY").is_some()
}

fn handle_command_suggestion_nav(key: KeyEvent, app: &mut App) -> bool {
    if !command_suggestions_active(app) {
        return false;
    }
    match key.code {
        KeyCode::Up => app.command_select.move_up(),
        KeyCode::Down => app.command_select.move_down(),
        KeyCode::PageUp => app.command_select.page_up(5),
        KeyCode::PageDown => app.command_select.page_down(5),
        KeyCode::BackTab => app.command_select.move_up(),
        _ => return false,
    }
    true
}

fn handle_input_editing(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Tab => handle_tab_key(key, app),
        KeyCode::Enter => handle_enter_key(app),
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input.insert_newline();
            Ok(false)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input = TextArea::default();
            refresh_command_suggestions(app);
            Ok(false)
        }
        _ => {
            let _ = app.input.input(key);
            refresh_command_suggestions(app);
            Ok(false)
        }
    }
}

fn handle_tab_key(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    if !command_suggestions_active(app) {
        refresh_command_suggestions(app);
    }
    if command_suggestions_active(app) && apply_command_suggestion(app) {
        refresh_command_suggestions(app);
        return Ok(false);
    }
    let _ = app.input.input(key);
    refresh_command_suggestions(app);
    Ok(false)
}

fn handle_enter_key(app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    let line = app.input.lines().join("\n");
    let line = line.trim_end().to_string();
    app.input = TextArea::default();
    refresh_command_suggestions(app);
    if line.trim().is_empty() {
        return Ok(false);
    }
    if line.starts_with('/') {
        return handle_command_line(&line, app);
    }
    app.pending_send = Some(line);
    Ok(false)
}

fn handle_scroll_key(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Home => scroll_home(app, key.modifiers.contains(KeyModifiers::CONTROL)),
        KeyCode::End => scroll_end(app, key.modifiers.contains(KeyModifiers::CONTROL)),
        KeyCode::F(12) => {
            app.scroll = u16::MAX;
            app.follow = true;
        }
        KeyCode::Up => scroll_by(app, -1),
        KeyCode::Down => scroll_by(app, 1),
        KeyCode::PageUp => scroll_by(app, -10),
        KeyCode::PageDown => scroll_by(app, 10),
        _ => {}
    }
}

fn scroll_home(app: &mut App, ctrl: bool) {
    if ctrl || app.focus == Focus::Chat {
        app.scroll = 0;
        app.follow = false;
    }
}

fn scroll_end(app: &mut App, ctrl: bool) {
    if ctrl || app.focus == Focus::Chat {
        app.scroll = u16::MAX;
        app.follow = ctrl;
    }
}

fn scroll_by(app: &mut App, delta: i32) {
    if delta < 0 {
        app.scroll = app.scroll.saturating_sub((-delta) as u16);
    } else {
        app.scroll = app.scroll.saturating_add(delta as u16);
    }
    app.follow = false;
}
