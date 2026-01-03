use crate::llm::prompt_manager::{augment_system, build_history_and_prompt, extract_system};
use crate::llm::templates::RigTemplates;
use crate::types::Message as UiMessage;
use reqwest12::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use rig::completion::{CompletionModel, CompletionRequestBuilder, Message, ToolDefinition};
use rig::prelude::CompletionClient;
use rig::providers::openai;

pub struct RigRequestContext {
    pub preamble: String,
    pub history: Vec<Message>,
    pub prompt: String,
    pub tools: Vec<ToolDefinition>,
}

pub fn prepare_rig_context(
    messages: &[UiMessage],
    prompts_dir: &str,
    enabled_tools: &[&str],
) -> Result<(RigRequestContext, RigTemplates), String> {
    let templates = RigTemplates::load(prompts_dir)?;
    let tools = filter_tools(templates.tool_defs()?, enabled_tools);
    let base_system = augment_system(&extract_system(messages));
    let preamble = build_preamble(&templates, &base_system, &tools)?;
    let (history, prompt) = build_history_and_prompt(messages, &templates)?;
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

pub fn openai_completion_model(
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<openai::completion::CompletionModel, String> {
    let url = normalize_base_url(base_url);
    let http_client = build_http_client()?;
    let client = openai::CompletionsClient::<reqwest12::Client>::builder()
        .api_key(api_key)
        .base_url(&url)
        .http_client(http_client)
        .build()
        .map_err(|e| format!("初始化 OpenAI 客户端失败：{e}"))?;
    Ok(client.completion_model(model))
}

fn build_http_client() -> Result<reqwest12::Client, String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    reqwest12::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("初始化 HTTP 客户端失败：{e}"))
}

fn normalize_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::templates::ToolSchema;

    #[test]
    fn normalize_base_url_appends_v1() {
        assert_eq!(
            normalize_base_url("https://api.example.com"),
            "https://api.example.com/v1"
        );
        assert_eq!(
            normalize_base_url("https://api.example.com/"),
            "https://api.example.com/v1"
        );
        assert_eq!(
            normalize_base_url("https://api.example.com/v1"),
            "https://api.example.com/v1"
        );
    }

    #[test]
    fn filter_tools_by_enabled_names() {
        let tools = vec![
            ToolSchema {
                name: "a".to_string(),
                description: "A".to_string(),
                parameters: serde_json::json!({}),
            },
            ToolSchema {
                name: "b".to_string(),
                description: "B".to_string(),
                parameters: serde_json::json!({}),
            },
        ];
        let filtered = filter_tools(tools, &["b"]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "b");
    }
}
