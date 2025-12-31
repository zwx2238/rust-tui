use clap::Parser;
use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser as MdParser, Tag, TagEnd};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// 模型名称
    #[arg(long, default_value = "deepseek-chat")]
    model: String,

    /// 系统提示词
    #[arg(long, default_value = "你是一个有帮助的助手。")]
    system: String,

    /// API Base URL
    #[arg(long, default_value = "https://api.deepseek.com")]
    base_url: String,

    /// 显示 reasoning_content（如果返回）
    #[arg(long, default_value_t = false)]
    show_reasoning: bool,

    /// 配置文件路径（JSON），默认：~/.config/deepseek/config.json
    #[arg(long)]
    config: Option<String>,

    /// 回放会话 ID（不触发真实 API）
    #[arg(long, alias = "resume")]
    replay: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: Option<String>,
    #[serde(rename = "reasoning_content")]
    reasoning_content: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    theme: Option<String>,
}

fn default_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("deepseek")
        .join("config.json"))
}

fn load_config(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let cfg: Config = serde_json::from_str(&text)?;
    Ok(cfg)
}

fn load_config_optional(path: &PathBuf) -> Option<Config> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
}

#[derive(Serialize, Deserialize)]
struct Session {
    id: String,
    messages: Vec<Message>,
}

fn sessions_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("deepseek")
        .join("sessions"))
}

fn save_session(messages: &[Message]) -> Result<String, Box<dyn std::error::Error>> {
    let dir = sessions_dir()?;
    fs::create_dir_all(&dir)?;
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间异常")?
        .as_secs()
        .to_string();
    let path = dir.join(format!("{id}.json"));
    let session = Session {
        id: id.clone(),
        messages: messages.to_vec(),
    };
    let text = serde_json::to_string_pretty(&session)?;
    fs::write(path, text)?;
    Ok(id)
}

fn load_session(id: &str) -> Result<Session, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(id);
    if path.extension().is_none() {
        let dir = sessions_dir()?;
        path = dir.join(format!("{id}.json"));
    }
    let text = fs::read_to_string(&path)?;
    let session: Session = serde_json::from_str(&text)?;
    Ok(session)
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if matches!(chars.peek(), Some('[')) {
                let _ = chars.next();
                while let Some(ch) = chars.next() {
                    if ch == 'm' {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

fn term_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
        .max(40)
}

fn print_padded_line(
    line: &str,
    pad: usize,
    width: usize,
    bg: Option<Color>,
    fg: Option<Color>,
) {
    let mut out = String::new();
    out.push_str(&" ".repeat(pad));
    out.push_str(line);

    let visible = UnicodeWidthStr::width(strip_ansi(&out).as_str());
    if visible < width {
        out.push_str(&" ".repeat(width - visible));
    }

    match (bg, fg) {
        (Some(bg), Some(fg)) => {
            print!(
                "{}{}{}{}",
                SetBackgroundColor(bg),
                SetForegroundColor(fg),
                out,
                ResetColor
            );
            println!();
        }
        (Some(bg), None) => {
            print!("{}{}{}", SetBackgroundColor(bg), out, ResetColor);
            println!();
        }
        (None, Some(fg)) => {
            print!("{}{}{}", SetForegroundColor(fg), out, ResetColor);
            println!();
        }
        (None, None) => {
            println!("{out}");
        }
    }
}

fn render_paragraph(text: &str, pad: usize, width: usize, bg: Color, fg: Option<Color>) {
    let inner_width = width.saturating_sub(pad * 2).max(10);
    for line in wrap(text, inner_width) {
        print_padded_line(&line, pad, width, Some(bg), fg);
    }
}

fn render_code_block(
    text: &str,
    lang: &str,
    pad: usize,
    width: usize,
    bg: Color,
    theme_name: &str,
) {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = ts
        .themes
        .get(theme_name)
        .unwrap_or_else(|| ts.themes.values().next().expect("theme set is empty"));
    let syntax = ss
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let mut highlighter = HighlightLines::new(syntax, theme);

    let lines: Vec<&str> = text.lines().collect();
    let max_digits = lines.len().max(1).to_string().len();
    for (i, raw) in lines.iter().enumerate() {
        let ranges = highlighter
            .highlight_line(raw, &ss)
            .unwrap_or_default();
        let highlighted = as_24_bit_terminal_escaped(&ranges[..], false);
        let line_no = format!("{:>width$} | ", i + 1, width = max_digits);
        let content = format!("{line_no}{highlighted}");
        print_padded_line(&content, pad, width, Some(bg), None);
    }
}

fn render_markdown(
    text: &str,
    pad: usize,
    width: usize,
    bg: Color,
    fg: Option<Color>,
    code_bg: Color,
    code_theme: &str,
    theme: &RenderTheme,
) {
    let parser = MdParser::new(text);
    let mut buf = String::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut heading_level: Option<HeadingLevel> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if !buf.trim().is_empty() {
                    render_paragraph(buf.trim(), pad, width, bg, fg);
                }
                buf.clear();
            }
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading_level.take() {
                    if !buf.trim().is_empty() {
                        render_heading(buf.trim(), level, pad, width, theme);
                    }
                    buf.clear();
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_buf.clear();
                code_lang.clear();
                if let CodeBlockKind::Fenced(lang) = kind {
                    code_lang = lang.to_string();
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                render_code_block(&code_buf, &code_lang, pad, width, code_bg, code_theme);
                in_code = false;
            }
            Event::Text(t) => {
                if in_code {
                    code_buf.push_str(&t);
                } else {
                    buf.push_str(&t);
                }
            }
            Event::SoftBreak => {
                if in_code {
                    code_buf.push('\n');
                } else {
                    buf.push(' ');
                }
            }
            Event::HardBreak => {
                if in_code {
                    code_buf.push('\n');
                } else {
                    buf.push('\n');
                }
            }
            _ => {}
        }
    }

    if !buf.trim().is_empty() {
        render_paragraph(buf.trim(), pad, width, bg, fg);
    }
}

struct RenderTheme {
    bg: Color,
    fg: Option<Color>,
    code_bg: Color,
    code_theme: &'static str,
    heading_fg: Option<Color>,
}

fn theme_from_config(cfg: Option<&Config>) -> RenderTheme {
    let name = cfg
        .and_then(|c| c.theme.as_deref())
        .unwrap_or("dark")
        .to_ascii_lowercase();
    if name == "light" {
        RenderTheme {
            bg: Color::White,
            fg: Some(Color::Black),
            code_bg: Color::White,
            code_theme: "base16-ocean.light",
            heading_fg: Some(Color::Blue),
        }
    } else {
        RenderTheme {
            bg: Color::Black,
            fg: None,
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }
}

fn render_heading(text: &str, level: HeadingLevel, pad: usize, width: usize, theme: &RenderTheme) {
    let inner_width = width.saturating_sub(pad * 2).max(10);
    let ch = match level {
        HeadingLevel::H1 => '=',
        HeadingLevel::H2 => '-',
        HeadingLevel::H3 => '~',
        _ => '.',
    };
    let rule = ch.to_string().repeat(inner_width);
    print_padded_line(&rule, pad, width, Some(theme.bg), theme.heading_fg);
    render_paragraph(text.trim(), pad, width, theme.bg, theme.heading_fg.or(theme.fg));
    print_padded_line(&rule, pad, width, Some(theme.bg), theme.heading_fg);
}

fn render_message(msg: &Message, theme: &RenderTheme) {
    let pad = 2;
    let width = term_width();
    match msg.role.as_str() {
        "user" => {
            print_padded_line("你>", pad, width, Some(theme.bg), theme.fg);
            render_markdown(
                &msg.content,
                pad,
                width,
                theme.bg,
                theme.fg,
                theme.code_bg,
                theme.code_theme,
                theme,
            );
        }
        "assistant" => {
            print_padded_line("AI>", pad, width, Some(theme.bg), theme.fg);
            render_markdown(
                &msg.content,
                pad,
                width,
                theme.bg,
                theme.fg,
                theme.code_bg,
                theme.code_theme,
                theme,
            );
        }
        _ => {}
    }
    let _ = std::io::stdout().flush();
}

const BIN_NAME: &str = "deepchat";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = args.base_url.trim_end_matches('/');
    let url = format!("{base_url}/chat/completions");
    let client = Client::new();

    let mut messages: Vec<Message> = Vec::new();

    let mut last_session_id: Option<String> = None;

    let cfg_path = match args.config.as_deref() {
        Some(p) => PathBuf::from(p),
        None => default_config_path()?,
    };
    let cfg_for_theme = load_config_optional(&cfg_path);
    let theme = theme_from_config(cfg_for_theme.as_ref());

    if let Some(id) = args.replay.as_deref() {
        let session = load_session(id)
            .map_err(|_| format!("无法读取回放会话：{id}"))?;
        messages = session.messages.clone();
        println!("回放模式已开启：{id}");
        for msg in &messages {
            render_message(msg, &theme);
        }
        if let Ok(id) = save_session(&messages) {
            println!("回放指令：{BIN_NAME} --resume {id}");
        }
        return Ok(());
    } else {
        if !args.system.trim().is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: args.system.clone(),
            });
        }
    }

    println!("DeepSeek CLI 已启动，输入 /help 查看命令。");

    loop {
        print!("你> ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }
        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('/') {
            match line {
                "/exit" | "/quit" => break,
                "/reset" | "/clear" => {
                    messages.clear();
                    if !args.system.trim().is_empty() {
                        messages.push(Message {
                            role: "system".to_string(),
                            content: args.system.clone(),
                        });
                    }
                    println!("已重置对话。");
                }
                "/save" => {
                    match save_session(&messages) {
                        Ok(id) => {
                            last_session_id = Some(id.clone());
                            println!("已保存会话：{id}");
                        }
                        Err(e) => eprintln!("保存失败：{e}"),
                    }
                }
                "/help" => {
                    println!("命令：/help /save /reset /clear /exit /quit");
                }
                _ => {
                    println!("未知命令：{line}");
                }
            }
            continue;
        }

        messages.push(Message {
            role: "user".to_string(),
            content: line.to_string(),
        });

        let cfg = load_config(&cfg_path)
            .map_err(|_| format!("无法读取配置文件：{}", cfg_path.display()))?;
        let api_key = cfg
            .api_key
            .ok_or("配置文件缺少 api_key")?;

        let req = ChatRequest {
            model: &args.model,
            messages: &messages,
            stream: false,
        };

        let resp = client
            .post(&url)
            .bearer_auth(&api_key)
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            eprintln!("请求失败：{status} {body}");
            continue;
        }

        let data: ChatResponse = resp.json().await?;
        let Some(choice) = data.choices.into_iter().next() else {
            eprintln!("响应中没有 choices。");
            continue;
        };

        if args.show_reasoning {
            if let Some(r) = choice.message.reasoning_content.as_deref() {
                if !r.trim().is_empty() {
                    println!("推理> {r}");
                }
            }
        }

        let content = choice.message.content.unwrap_or_default();
        let assistant_msg = Message {
            role: "assistant".to_string(),
            content,
        };
        render_message(&assistant_msg, &theme);
        messages.push(assistant_msg);
    }

    if last_session_id.is_none() {
        if let Ok(id) = save_session(&messages) {
            last_session_id = Some(id);
        }
    }
    if let Some(id) = last_session_id {
        println!("回放指令：{BIN_NAME} --resume {id}");
    }

    Ok(())
}
