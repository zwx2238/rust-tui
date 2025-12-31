use crate::session::save_session;
use crate::types::Message;
use crate::ui::clipboard;
use crate::ui::state::{App, Focus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

pub fn handle_key(
    key: KeyEvent,
    app: &mut App,
    last_session_id: &mut Option<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
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
                    return Ok(false);
                }
                KeyCode::Char('v') => {
                    if let Some(text) = clipboard::get() {
                        app.input.set_yank_text(text);
                        app.input.paste();
                    }
                    return Ok(false);
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Enter => {
            let line = app.input.lines().join("\n");
                let line = line.trim_end().to_string();
                app.input = TextArea::default();
                if line.trim().is_empty() {
                    return Ok(false);
                }
                if line.starts_with('/') {
                    return handle_command(&line, app, last_session_id);
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
                return Ok(false);
            }
            _ => {
                let _ = app.input.input(key);
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

pub fn handle_command(
    line: &str,
    app: &mut App,
    last_session_id: &mut Option<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match line {
        "/exit" | "/quit" => return Ok(true),
        "/reset" | "/clear" => {
            let system = app
                .messages
                .iter()
                .find(|m| m.role == "system")
                .cloned();
            app.messages.clear();
            app.assistant_stats.clear();
            if let Some(sys) = system {
                app.messages.push(sys);
            }
            app.follow = true;
        }
        "/save" => {
            if let Ok(id) = save_session(&app.messages) {
                *last_session_id = Some(id.clone());
                app.messages.push(Message {
                    role: "assistant".to_string(),
                    content: format!("已保存会话：{id}"),
                });
            }
        }
        "/help" => {
            app.messages.push(Message {
                role: "assistant".to_string(),
                content: "命令：/help /save /reset /clear /exit /quit".to_string(),
            });
        }
        _ => {
            app.messages.push(Message {
                role: "assistant".to_string(),
                content: format!("未知命令：{line}"),
            });
        }
    }
    Ok(false)
}
