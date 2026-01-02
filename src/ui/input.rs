use crate::types::{Message, ROLE_ASSISTANT, ROLE_SYSTEM};
use crate::ui::clipboard;
use crate::ui::commands::{command_has_args, command_names, commands_help_text};
use crate::ui::state::{App, Focus, PendingCommand};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use tui_textarea::CursorMove;
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
            KeyCode::Tab => {
                if try_autocomplete_command(app) {
                    return Ok(false);
                }
                let _ = app.input.input(key);
                return Ok(false);
            }
            KeyCode::Enter => {
                let line = app.input.lines().join("\n");
                let line = line.trim_end().to_string();
                app.input = TextArea::default();
                if line.trim().is_empty() {
                    return Ok(false);
                }
                if line.starts_with('/') {
                    return handle_command(&line, app);
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

pub fn handle_command(line: &str, app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();
    match line {
        "/exit" | "/quit" => return Ok(true),
        "/reset" | "/clear" => {
            let system = app.messages.iter().find(|m| m.role == ROLE_SYSTEM).cloned();
            app.messages.clear();
            app.assistant_stats.clear();
            if let Some(sys) = system {
                app.messages.push(sys);
            }
            app.follow = true;
        }
        "/save" => {
            app.pending_command = Some(PendingCommand::SaveSession);
        }
        "/help" => {
            app.messages.push(Message {
                role: ROLE_ASSISTANT.to_string(),
                content: commands_help_text(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        _ if cmd == "/category" => {
            app.pending_category_name = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            app.pending_command = Some(PendingCommand::NewCategory);
        }
        _ if cmd == "/open" => {
            if arg.is_empty() {
                app.messages.push(Message {
                    role: ROLE_ASSISTANT.to_string(),
                    content: "用法：/open <conversation_id>".to_string(),
                    tool_call_id: None,
                    tool_calls: None,
                });
            } else {
                app.pending_open_conversation = Some(arg.to_string());
                app.pending_command = Some(PendingCommand::OpenConversation);
            }
        }
        _ if cmd == "/list-conv" => {
            match crate::conversation::conversations_dir()
                .and_then(|dir| std::fs::read_dir(dir).map_err(|e| e.into()))
            {
                Ok(entries) => {
                    let mut ids = Vec::new();
                    for entry in entries.flatten() {
                        if let Some(stem) = entry.path().file_stem() {
                            ids.push(stem.to_string_lossy().to_string());
                        }
                    }
                    ids.sort();
                    let content = if ids.is_empty() {
                        "暂无对话文件。".to_string()
                    } else {
                        format!("可用对话：\n{}", ids.join("\n"))
                    };
                    app.messages.push(Message {
                        role: ROLE_ASSISTANT.to_string(),
                        content,
                        tool_call_id: None,
                        tool_calls: None,
                    });
                }
                Err(e) => {
                    app.messages.push(Message {
                        role: ROLE_ASSISTANT.to_string(),
                        content: format!("读取对话目录失败：{e}"),
                        tool_call_id: None,
                        tool_calls: None,
                    });
                }
            }
        }
        _ => {
            app.messages.push(Message {
                role: ROLE_ASSISTANT.to_string(),
                content: format!("未知命令：{line}"),
                tool_call_id: None,
                tool_calls: None,
            });
        }
    }
    Ok(false)
}

fn try_autocomplete_command(app: &mut App) -> bool {
    let (row, col) = app.input.cursor();
    let mut lines = app.input.lines().to_vec();
    if row >= lines.len() {
        return false;
    }
    let line = lines[row].clone();
    if !line.starts_with('/') {
        return false;
    }
    let cursor = col.min(line.len());
    let token_end = line.find(char::is_whitespace).unwrap_or(line.len());
    if cursor > token_end {
        return false;
    }
    let token = &line[..token_end];
    if !token.starts_with('/') {
        return false;
    }
    let pattern = token.trim_start_matches('/');
    let matcher = SkimMatcherV2::default();
    let mut best: Option<(&str, i64)> = None;
    for name in command_names() {
        let candidate = name.trim_start_matches('/');
        let score = if pattern.is_empty() {
            Some(0)
        } else {
            matcher.fuzzy_match(candidate, pattern)
        };
        if let Some(score) = score {
            let should_replace = match best {
                None => true,
                Some((_, best_score)) => score > best_score,
            };
            if should_replace {
                best = Some((candidate, score));
            }
        }
    }
    let Some((best_name, _)) = best else {
        return false;
    };
    let mut new_line = String::new();
    new_line.push('/');
    new_line.push_str(best_name);
    let mut new_col = new_line.len();
    let needs_args = command_has_args(&format!("/{best_name}"));
    if token_end == line.len() && needs_args {
        new_line.push(' ');
        new_col += 1;
    }
    if token_end < line.len() {
        new_line.push_str(&line[token_end..]);
    }
    lines[row] = new_line;
    app.input = TextArea::from(lines);
    app.input
        .move_cursor(CursorMove::Jump(row as u16, new_col as u16));
    true
}
