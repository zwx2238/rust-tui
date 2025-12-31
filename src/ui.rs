use crate::args::Args;
use crate::render::{messages_to_text, RenderTheme};
use crate::session::save_session;
use crate::types::{ChatRequest, ChatResponse, Message};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{self};
use std::time::Duration;

struct App {
    input: String,
    cursor: usize,
    messages: Vec<Message>,
    scroll: u16,
    follow: bool,
    focus: Focus,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Focus {
    Chat,
    Input,
}

impl App {
    fn new(system_prompt: &str) -> Self {
        let mut messages = Vec::new();
        if !system_prompt.trim().is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            });
        }
        Self {
            input: String::new(),
            cursor: 0,
            messages,
            scroll: 0,
            follow: true,
            focus: Focus::Input,
        }
    }
}

pub fn run(
    args: Args,
    api_key: String,
    _cfg: Option<crate::config::Config>,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let base_url = args.base_url.trim_end_matches('/');
    let url = format!("{base_url}/chat/completions");
    let client = reqwest::blocking::Client::new();

    let mut app = App::new(&args.system);
    let mut last_session_id: Option<String> = None;

    loop {
        let size = terminal.size()?;
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
            .split(size);

        let (msg_area, input_area) = (layout[0], layout[1]);
        let msg_width = inner_width(msg_area, 1);
        let text = messages_to_text(&app.messages, msg_width, theme);
        let total_lines = text.lines.len();
        let view_height = inner_height(msg_area, 1) as usize;
        let max_scroll = total_lines.saturating_sub(view_height).min(u16::MAX as usize) as u16;

        if app.follow {
            app.scroll = max_scroll;
        } else if app.scroll > max_scroll {
            app.scroll = max_scroll;
        } else if app.scroll >= max_scroll {
            app.follow = true;
        }

        terminal.draw(|f| {
            draw_messages(
                f,
                msg_area,
                &text,
                app.scroll,
                theme,
                app.focus == Focus::Chat,
            );
            draw_input(
                f,
                input_area,
                &app.input,
                app.cursor,
                theme,
                app.focus == Focus::Input,
            );
        })?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(
                        key,
                        &mut app,
                        &client,
                        &url,
                        &api_key,
                        &args,
                        &mut last_session_id,
                    )? {
                        break;
                    }
                }
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::Down(_) => {
                        if point_in_rect(m.column, m.row, input_area) {
                            app.focus = Focus::Input;
                            app.cursor = app.input.len();
                        } else if point_in_rect(m.column, m.row, msg_area) {
                            app.focus = Focus::Chat;
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        app.scroll = app.scroll.saturating_sub(3);
                        app.follow = false;
                        app.focus = Focus::Chat;
                    }
                    MouseEventKind::ScrollDown => {
                        app.scroll = app.scroll.saturating_add(3);
                        app.focus = Focus::Chat;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if last_session_id.is_none() {
        if let Ok(id) = save_session(&app.messages) {
            last_session_id = Some(id);
        }
    }
    if let Some(id) = last_session_id {
        println!("回放指令：deepchat --resume {id}");
    }

    Ok(())
}

fn handle_key(
    key: KeyEvent,
    app: &mut App,
    client: &reqwest::blocking::Client,
    url: &str,
    api_key: &str,
    args: &Args,
    last_session_id: &mut Option<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
        KeyCode::Esc => return Ok(true),
        KeyCode::Enter => {
            if app.focus != Focus::Input {
                return Ok(false);
            }
            let line = app.input.trim().to_string();
            app.input.clear();
            app.cursor = 0;
            if line.is_empty() {
                return Ok(false);
            }
            if line.starts_with('/') {
                return handle_command(&line, app, last_session_id);
            }

            app.messages.push(Message {
                role: "user".to_string(),
                content: line,
            });
            app.follow = true;

            let req = ChatRequest {
                model: &args.model,
                messages: &app.messages,
                stream: false,
            };

            let resp = client
                .post(url)
                .bearer_auth(api_key)
                .json(&req)
                .send()?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                app.messages.push(Message {
                    role: "assistant".to_string(),
                    content: format!("请求失败：{status} {body}"),
                });
                return Ok(false);
            }

            let data: ChatResponse = resp.json()?;
            let Some(choice) = data.choices.into_iter().next() else {
                app.messages.push(Message {
                    role: "assistant".to_string(),
                    content: "响应中没有 choices。".to_string(),
                });
                return Ok(false);
            };

            if args.show_reasoning {
                if let Some(r) = choice.message.reasoning_content.as_deref() {
                    if !r.trim().is_empty() {
                        app.messages.push(Message {
                            role: "assistant".to_string(),
                            content: format!("推理> {r}"),
                        });
                    }
                }
            }

            let content = choice.message.content.unwrap_or_default();
            app.messages.push(Message {
                role: "assistant".to_string(),
                content,
            });
            app.follow = true;
        }
        KeyCode::Backspace => {
            if app.focus != Focus::Input {
                return Ok(false);
            }
            if app.cursor > 0 {
                let new_cursor = prev_char_boundary(&app.input, app.cursor);
                app.input.replace_range(new_cursor..app.cursor, "");
                app.cursor = new_cursor;
            }
        }
        KeyCode::Left => {
            if app.focus != Focus::Input {
                return Ok(false);
            }
            app.cursor = prev_char_boundary(&app.input, app.cursor);
        }
        KeyCode::Right => {
            if app.focus != Focus::Input {
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
                app.scroll = 0;
                app.follow = false;
            }
        }
        KeyCode::End => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if app.focus == Focus::Input {
                    app.cursor = app.input.len();
                }
            } else {
                app.follow = true;
            }
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
        KeyCode::Up => {
            app.scroll = app.scroll.saturating_sub(1);
            app.follow = false;
        }
        KeyCode::Down => {
            app.scroll = app.scroll.saturating_add(1);
        }
        KeyCode::PageUp => {
            app.scroll = app.scroll.saturating_sub(10);
            app.follow = false;
        }
        KeyCode::PageDown => {
            app.scroll = app.scroll.saturating_add(10);
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(false);
            }
            if app.focus != Focus::Input {
                return Ok(false);
            }
            app.input.insert(app.cursor, c);
            app.cursor = next_char_boundary(&app.input, app.cursor);
        }
        _ => {}
    }
    Ok(false)
}

fn handle_command(
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

fn draw_messages(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    text: &Text<'_>,
    scroll: u16,
    theme: &RenderTheme,
    focused: bool,
) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.fg.unwrap_or(Color::White));
    let border_style = if focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(theme.fg.unwrap_or(Color::White))
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title("对话")
        .style(style)
        .border_style(border_style);
    let paragraph = Paragraph::new(text.clone())
        .block(block)
        .style(style)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(paragraph, area);
}

fn draw_input(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    input: &str,
    cursor: usize,
    theme: &RenderTheme,
    focused: bool,
) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.fg.unwrap_or(Color::White));
    let border_style = if focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(theme.fg.unwrap_or(Color::White))
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title("输入")
        .style(style)
        .border_style(border_style);
    let paragraph = Paragraph::new(input)
        .block(block)
        .style(style);
    f.render_widget(paragraph, area);

    let x = cursor_display_width(&input[..cursor]) as u16;
    let inner = inner_area(area, 0);
    let max_x = inner.x.saturating_add(inner.width.saturating_sub(1));
    let cursor_x = inner.x.saturating_add(x).min(max_x);
    let cursor_y = inner.y;
    f.set_cursor(cursor_x, cursor_y);
}

fn cursor_display_width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}

fn prev_char_boundary(s: &str, idx: usize) -> usize {
    s[..idx]
        .char_indices()
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn next_char_boundary(s: &str, idx: usize) -> usize {
    let mut iter = s[idx..].char_indices();
    iter.next();
    if let Some((i, _)) = iter.next() {
        idx + i
    } else {
        s.len()
    }
}

fn inner_area(area: Rect, padding: u16) -> Rect {
    Rect {
        x: area.x + 1 + padding,
        y: area.y + 1 + padding,
        width: area.width.saturating_sub(2 + padding * 2),
        height: area.height.saturating_sub(2 + padding * 2),
    }
}

fn inner_width(area: Rect, padding: u16) -> usize {
    area.width.saturating_sub(2 + padding * 2) as usize
}

fn inner_height(area: Rect, padding: u16) -> u16 {
    area.height.saturating_sub(2 + padding * 2)
}

fn point_in_rect(x: u16, y: u16, rect: Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}
