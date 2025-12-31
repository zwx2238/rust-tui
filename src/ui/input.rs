use crate::session::save_session;
use crate::types::Message;
use crate::ui::state::{App, Focus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(
    key: KeyEvent,
    app: &mut App,
    last_session_id: &mut Option<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
        KeyCode::Esc => return Ok(true),
        KeyCode::Enter => {
            if app.focus != Focus::Input {
                return Ok(false);
            }
            if app.busy {
                return Ok(false);
            }
            let line = app.input.trim_end().to_string();
            app.input.clear();
            app.cursor = 0;
            if line.trim().is_empty() {
                return Ok(false);
            }
            if line.starts_with('/') {
                return handle_command(&line, app, last_session_id);
            }
            app.pending_send = Some(line);
        }
        KeyCode::Backspace => {
            if app.focus != Focus::Input || app.busy {
                return Ok(false);
            }
            if app.cursor > 0 {
                let new_cursor = prev_char_boundary(&app.input, app.cursor);
                app.input.replace_range(new_cursor..app.cursor, "");
                app.cursor = new_cursor;
            }
        }
        KeyCode::Left => {
            if app.focus != Focus::Input || app.busy {
                return Ok(false);
            }
            app.cursor = prev_char_boundary(&app.input, app.cursor);
        }
        KeyCode::Right => {
            if app.focus != Focus::Input || app.busy {
                return Ok(false);
            }
            app.cursor = next_char_boundary(&app.input, app.cursor);
        }
        KeyCode::Home => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if app.focus == Focus::Input {
                    app.cursor = 0;
                }
            } else {
                if app.focus == Focus::Chat {
                    app.scroll = 0;
                    app.follow = false;
                } else if app.focus == Focus::Input {
                    app.cursor = 0;
                }
            }
        }
        KeyCode::End => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if app.focus == Focus::Input {
                    app.cursor = app.input.len();
                }
            } else {
                if app.focus == Focus::Chat {
                    app.scroll = u16::MAX;
                    app.follow = false;
                } else if app.focus == Focus::Input {
                    app.cursor = app.input.len();
                }
            }
        }
        KeyCode::F(12) => {
            app.scroll = u16::MAX;
            app.follow = true;
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.focus == Focus::Input {
                app.cursor = 0;
            }
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.focus == Focus::Input {
                app.cursor = app.input.len();
            }
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.focus == Focus::Input && !app.busy {
                app.input.insert(app.cursor, '\n');
                app.cursor = next_char_boundary(&app.input, app.cursor);
            }
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
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(false);
            }
            if app.focus != Focus::Input || app.busy {
                return Ok(false);
            }
            app.input.insert(app.cursor, c);
            app.cursor = next_char_boundary(&app.input, app.cursor);
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

pub fn prev_char_boundary(s: &str, idx: usize) -> usize {
    s[..idx]
        .char_indices()
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

pub fn next_char_boundary(s: &str, idx: usize) -> usize {
    let mut iter = s[idx..].char_indices();
    iter.next();
    if let Some((i, _)) = iter.next() {
        idx + i
    } else {
        s.len()
    }
}

pub fn cursor_position(s: &str, idx: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;
    let mut i = 0usize;
    for ch in s.chars() {
        if i >= idx {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        }
        i += ch.len_utf8();
    }
    (line, col)
}
