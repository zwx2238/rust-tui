use crate::ui::clipboard;
use crate::ui::command_input::handle_command_line;
use crate::ui::command_suggestions::{
    apply_command_suggestion, command_suggestions_active, refresh_command_suggestions,
};
use crate::ui::state::{App, Focus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

pub fn handle_key(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    if app.pending_code_exec.is_some() || app.pending_file_patch.is_some() {
        return Ok(false);
    }
    if key.code == KeyCode::Esc {
        return Ok(true);
    }

    if app.focus == Focus::Input && !app.busy {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('a') => {
                    app.input.select_all();
                    return Ok(false);
                }
                KeyCode::Char('c') => {
                    if app.input.is_selecting() {
                        app.input.copy();
                        let text = app.input.yank_text();
                        clipboard::set(&text);
                    }
                    return Ok(false);
                }
                KeyCode::Char('x') => {
                    if app.input.is_selecting() && app.input.cut() {
                        let text = app.input.yank_text();
                        clipboard::set(&text);
                    }
                    refresh_command_suggestions(app);
                    return Ok(false);
                }
                KeyCode::Char('v') => {
                    if let Some(text) = clipboard::get() {
                        app.input.set_yank_text(text);
                        app.input.paste();
                    }
                    refresh_command_suggestions(app);
                    return Ok(false);
                }
                _ => {}
            }
        }
        if command_suggestions_active(app) {
            match key.code {
                KeyCode::Up => {
                    app.command_select.move_up();
                    return Ok(false);
                }
                KeyCode::Down => {
                    app.command_select.move_down();
                    return Ok(false);
                }
                KeyCode::PageUp => {
                    app.command_select.page_up(5);
                    return Ok(false);
                }
                KeyCode::PageDown => {
                    app.command_select.page_down(5);
                    return Ok(false);
                }
                KeyCode::BackTab => {
                    app.command_select.move_up();
                    return Ok(false);
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Tab => {
                if !command_suggestions_active(app) {
                    refresh_command_suggestions(app);
                }
                if command_suggestions_active(app) && apply_command_suggestion(app) {
                    refresh_command_suggestions(app);
                    return Ok(false);
                }
                let _ = app.input.input(key);
                refresh_command_suggestions(app);
                return Ok(false);
            }
            KeyCode::Enter => {
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
                return Ok(false);
            }
            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.input.insert_newline();
                return Ok(false);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.input = TextArea::default();
                refresh_command_suggestions(app);
                return Ok(false);
            }
            _ => {
                let _ = app.input.input(key);
                refresh_command_suggestions(app);
                return Ok(false);
            }
        }
    }

    match key.code {
        KeyCode::Home => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.scroll = 0;
                app.follow = false;
            } else if app.focus == Focus::Chat {
                app.scroll = 0;
                app.follow = false;
            }
        }
        KeyCode::End => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.scroll = u16::MAX;
                app.follow = true;
            } else if app.focus == Focus::Chat {
                app.scroll = u16::MAX;
                app.follow = false;
            }
        }
        KeyCode::F(12) => {
            app.scroll = u16::MAX;
            app.follow = true;
        }
        KeyCode::Up => {
            app.scroll = app.scroll.saturating_sub(1);
            app.follow = false;
        }
        KeyCode::Down => {
            app.scroll = app.scroll.saturating_add(1);
            app.follow = false;
        }
        KeyCode::PageUp => {
            app.scroll = app.scroll.saturating_sub(10);
            app.follow = false;
        }
        KeyCode::PageDown => {
            app.scroll = app.scroll.saturating_add(10);
            app.follow = false;
        }
        _ => {}
    }
    Ok(false)
}
