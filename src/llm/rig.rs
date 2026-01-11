use crate::llm::prompt_manager::{augment_system, build_history_and_prompt, extract_system};
use crate::llm::http_client::JsonStreamingClient;
use crate::llm::templates::RigTemplates;
use crate::types::Message as UiMessage;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use rig::completion::{CompletionModel, CompletionRequestBuilder, Message, ToolDefinition};
use rig::prelude::CompletionClient;
use rig::providers::{anthropic, deepseek, openai};

pub struct RigRequestContext {
    pub preamble: String,
    pub history: Vec<Message>,
    pub prompt: String,
    pub tools: Vec<ToolDefinition>,
}

pub enum CompletionModelChoice {
    OpenAi(openai::completion::CompletionModel),
    DeepSeek(deepseek::CompletionModel<JsonStreamingClient>),
    Anthropic(anthropic::completion::CompletionModel),
}

pub fn prepare_rig_context(
    messages: &[UiMessage],
    prompts_dir: &str,
    enabled_tools: &[&str],
    default_role: &str,
) -> Result<(RigRequestContext, RigTemplates), String> {
    let templates = RigTemplates::load(prompts_dir)?;
    let tools = filter_tools(templates.tool_defs()?, enabled_tools);
    let base_system = augment_system(&extract_system(messages));
    let preamble = build_preamble(&templates, &base_system, &tools)?;
    let (history, prompt) = build_history_and_prompt(messages, &templates, default_role)?;
    let tool_defs = build_tool_defs(&tools);
    Ok((
        RigRequestContext {
            preamble,
            history,
            prompt,
            tools: tool_defs,
        },
        templates,
    ))
}

fn build_preamble(
    templates: &RigTemplates,
    base_system: &str,
    tools: &[crate::llm::templates::ToolSchema],
) -> Result<String, String> {
    if tools.is_empty() {
        return Ok(base_system.to_string());
    }
    templates.render_preamble(base_system, tools)
}

fn build_tool_defs(tools: &[crate::llm::templates::ToolSchema]) -> Vec<ToolDefinition> {
    tools
        .iter()
        .map(|t| ToolDefinition {
            name: t.name.clone(),
            description: t.description.clone(),
            parameters: t.parameters.clone(),
        })
        .collect()
}

pub fn build_completion_request<M: CompletionModel>(
    model: &M,
    ctx: &RigRequestContext,
) -> CompletionRequestBuilder<M> {
    model
        .completion_request(Message::user(ctx.prompt.clone()))
        .preamble(ctx.preamble.clone())
        .messages(ctx.history.clone())
        .tools(ctx.tools.clone())
}

pub fn completion_model_for(
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<CompletionModelChoice, String> {
    if is_anthropic_provider(base_url, model) {
        return anthropic_completion_model(base_url, api_key, model)
            .map(CompletionModelChoice::Anthropic);
    }
    if is_deepseek_provider(base_url, model) {
        return deepseek_completion_model(base_url, api_key, model)
            .map(CompletionModelChoice::DeepSeek);
    }
    openai_completion_model(base_url, api_key, model).map(CompletionModelChoice::OpenAi)
}

fn openai_completion_model(
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<openai::completion::CompletionModel, String> {
    let url = normalize_openai_base_url(base_url);
    let http_client = build_http_client()?;
    let client = openai::CompletionsClient::<reqwest::Client>::builder()
        .api_key(api_key)
        .base_url(&url)
        .http_client(http_client)
        .build()
        .map_err(|e| format!("初始化 OpenAI 客户端失败：{e}"))?;
    Ok(client.completion_model(model))
}

fn deepseek_completion_model(
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<deepseek::CompletionModel<JsonStreamingClient>, String> {
    let url = normalize_deepseek_base_url(base_url);
    let client = deepseek::Client::<JsonStreamingClient>::builder()
        .api_key(api_key)
        .base_url(&url)
        .http_client(JsonStreamingClient::default())
        .build()
        .map_err(|e| format!("初始化 DeepSeek 客户端失败：{e}"))?;
    Ok(client.completion_model(model))
}

fn anthropic_completion_model(
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<anthropic::completion::CompletionModel, String> {
    let url = normalize_anthropic_base_url(base_url);
    let http_client = build_http_client()?;
    let client = anthropic::Client::<reqwest::Client>::builder()
        .api_key(api_key)
        .base_url(&url)
        .http_client(http_client)
        .build()
        .map_err(|e| format!("初始化 Anthropic 客户端失败：{e}"))?;
    Ok(client.completion_model(model))
}

fn build_http_client() -> Result<reqwest::Client, String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("初始化 HTTP 客户端失败：{e}"))
}

fn normalize_openai_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

fn normalize_deepseek_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    trimmed.strip_suffix("/v1").unwrap_or(trimmed).to_string()
}

fn normalize_anthropic_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    trimmed.strip_suffix("/v1").unwrap_or(trimmed).to_string()
}

fn is_deepseek_provider(base_url: &str, model: &str) -> bool {
    let base = base_url.to_ascii_lowercase();
    if base.contains("deepseek") {
        return true;
    }
    model.to_ascii_lowercase().starts_with("deepseek-")
}

fn is_anthropic_provider(base_url: &str, model: &str) -> bool {
    let base = base_url.to_ascii_lowercase();
    if base.contains("anthropic") || base.contains("claude") {
        return true;
    }
    model.to_ascii_lowercase().starts_with("claude-")
}

fn filter_tools(
    tools: Vec<crate::llm::templates::ToolSchema>,
    enabled: &[&str],
) -> Vec<crate::llm::templates::ToolSchema> {
    if enabled.is_empty() {
        return Vec::new();
    }
    tools
        .into_iter()
        .filter(|tool| enabled.iter().any(|name| *name == tool.name))
        .collect()
}
