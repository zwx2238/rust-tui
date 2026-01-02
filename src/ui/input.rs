use crate::types::{Message, ROLE_ASSISTANT, ROLE_SYSTEM};
use crate::ui::clipboard;
use crate::ui::state::{App, Focus, PendingCommand};
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
                content: "命令：/help /save /reset /clear /exit /quit /category [name] /open <id> /list-conv；快捷键：F6 终止生成，Shift+F6 终止并编辑上一问，F2 消息定位（E 复制用户消息到新对话），g 进入语义导航（j/k 或 n/p 上下消息，Esc 退出）"
                    .to_string(),
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
