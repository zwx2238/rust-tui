use crate::llm::prompt_manager::{augment_system, build_history_and_prompt, extract_system};
use crate::llm::templates::RigTemplates;
use crate::types::{Message as UiMessage, Usage};
use rig::completion::{CompletionModel, Message, ModelChoice, ToolDefinition};
use rig::providers::openai;

#[derive(Debug)]
pub enum RigOutcome {
    Message {
        content: String,
        usage: Option<Usage>,
    },
    ToolCall {
        name: String,
        args: serde_json::Value,
        usage: Option<Usage>,
    },
}

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
    let preamble = if tools.is_empty() {
        base_system
    } else {
        templates.render_preamble(&base_system, &tools)?
    };
    let (history, prompt) = build_history_and_prompt(messages, &templates)?;
    let tool_defs = tools
        .iter()
        .map(|t| ToolDefinition {
            name: t.name.clone(),
            description: t.description.clone(),
            parameters: t.parameters.clone(),
        })
        .collect();
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

pub async fn rig_complete(
    base_url: &str,
    api_key: &str,
    model: &str,
    ctx: RigRequestContext,
) -> Result<RigOutcome, String> {
    let url = normalize_base_url(base_url);
    let client = openai::Client::from_url(api_key, &url);
    let completion_model = client.completion_model(model);
    let request = completion_model
        .completion_request(&ctx.prompt)
        .preamble(ctx.preamble)
        .messages(ctx.history)
        .tools(ctx.tools)
        .build();
    let response = completion_model
        .completion(request)
        .await
        .map_err(|e| format!("请求失败：{e}"))?;
    let usage = extract_usage(&response.raw_response);
    match response.choice {
        ModelChoice::Message(msg) => Ok(RigOutcome::Message {
            content: msg,
            usage,
        }),
        ModelChoice::ToolCall(name, args) => Ok(RigOutcome::ToolCall { name, args, usage }),
    }
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

fn extract_usage(response: &openai::CompletionResponse) -> Option<Usage> {
    response.usage.as_ref().map(|u| {
        let prompt = u.prompt_tokens as u64;
        let total = u.total_tokens as u64;
        let completion = total.saturating_sub(prompt);
        Usage {
            prompt_tokens: Some(prompt),
            completion_tokens: Some(completion),
            total_tokens: Some(total),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::templates::ToolSchema;

    #[test]
    fn normalize_base_url_appends_v1() {
        assert_eq!(normalize_base_url("https://api.example.com"), "https://api.example.com/v1");
        assert_eq!(normalize_base_url("https://api.example.com/"), "https://api.example.com/v1");
        assert_eq!(normalize_base_url("https://api.example.com/v1"), "https://api.example.com/v1");
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
