use std::error::Error;
use std::fmt;

use clap::Parser;
use rig::completion::CompletionModel;
use rig::completion::{AssistantContent, Message};
use rig::prelude::CompletionClient;
use rig::providers::anthropic;

#[derive(Debug, Clone, Parser)]
#[command(name = "anthropic-rig", about = "用 rig 调用 Anthropic Messages API（支持自定义 base_url）")]
struct Args {
    /// API Key（不传则读取环境变量 ANTHROPIC_API_KEY）
    #[arg(long)]
    api_key: Option<String>,

    /// 自定义 API 地址（不传则读取环境变量 ANTHROPIC_BASE_URL）
    #[arg(long)]
    base_url: Option<String>,

    /// 模型名（可以是代理侧自定义模型名）
    #[arg(long, default_value = "claude-sonnet-4-5-20250929")]
    model: String,

    /// max_tokens（Anthropic 必填）
    #[arg(long, default_value_t = 1024)]
    max_tokens: u64,

    /// 用户消息内容
    #[arg(long, default_value = "Hello")]
    prompt: String,
}

#[derive(Debug)]
struct AppError(String);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for AppError {}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let args = Args::parse();
    let client = build_client(&args)?;
    let model = client.completion_model(&args.model);

    let response = model
        .completion_request(Message::user(args.prompt))
        .max_tokens(args.max_tokens)
        .send()
        .await
        .map_err(to_err)?;

    // 尽量贴近 Python `print(message.content)`：输出原始 content blocks 的 Debug 视图
    println!("{:#?}", response.raw_response.content);

    // 同时把可见文本拼出来（方便直接复制）
    let text = extract_text(response.choice);
    if !text.is_empty() {
        println!("\n{text}");
    }

    Ok(())
}

fn build_client(args: &Args) -> Result<anthropic::Client, AppError> {
    let api_key = args
        .api_key
        .clone()
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .ok_or_else(|| AppError("缺少 API Key：请传 --api-key 或设置 ANTHROPIC_API_KEY".into()))?;

    let mut builder = anthropic::Client::builder().api_key(api_key);
    let base_url = args
        .base_url
        .clone()
        .or_else(|| std::env::var("ANTHROPIC_BASE_URL").ok());
    if let Some(url) = base_url {
        builder = builder.base_url(url);
    }

    builder.build().map_err(to_err)
}

fn extract_text(choice: rig::OneOrMany<AssistantContent>) -> String {
    choice
        .into_iter()
        .filter_map(|c| match c {
            AssistantContent::Text(t) => Some(t.text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn to_err<E: fmt::Display>(e: E) -> AppError {
    AppError(e.to_string())
}
