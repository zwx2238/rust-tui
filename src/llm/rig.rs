use crate::llm::templates::RigTemplates;
use crate::types::Message as UiMessage;
use rig::completion::{CompletionModel, Message, ModelChoice, ToolDefinition};
use rig::providers::openai;

#[derive(Debug)]
pub enum RigOutcome {
    Message(String),
    ToolCall { name: String, args: serde_json::Value },
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
    match response.choice {
        ModelChoice::Message(msg) => Ok(RigOutcome::Message(msg)),
        ModelChoice::ToolCall(name, args) => Ok(RigOutcome::ToolCall { name, args }),
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

fn extract_system(messages: &[UiMessage]) -> String {
    messages
        .iter()
        .find(|m| m.role == crate::types::ROLE_SYSTEM)
        .map(|m| m.content.clone())
        .unwrap_or_default()
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

fn augment_system(base: &str) -> String {
    let mut out = String::new();
    if !base.trim().is_empty() {
        out.push_str(base.trim());
        out.push('\n');
    }
    out.push_str("注意：数学公式仅支持 $...$ 或 $$...$$，不支持 ```latex``` 代码块。");
    out
}

fn build_history_and_prompt(
    messages: &[UiMessage],
    templates: &RigTemplates,
) -> Result<(Vec<Message>, String), String> {
    let mut last_user_idx = None;
    for (idx, msg) in messages.iter().enumerate() {
        if msg.role == crate::types::ROLE_USER {
            last_user_idx = Some(idx);
        }
    }
    let mut history = Vec::new();
    if let Some(idx) = last_user_idx {
        for msg in &messages[..idx] {
            if msg.role == crate::types::ROLE_SYSTEM {
                continue;
            }
            if msg.role == crate::types::ROLE_TOOL {
                let wrapped = templates.render_tool_result("tool", &serde_json::Value::Null, &msg.content)?;
                history.push(Message {
                    role: "user".to_string(),
                    content: wrapped,
                });
            } else {
                history.push(Message {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                });
            }
        }
        Ok((history, messages[idx].content.clone()))
    } else {
        for msg in messages {
            if msg.role == crate::types::ROLE_SYSTEM {
                continue;
            }
            if msg.role == crate::types::ROLE_TOOL {
                let wrapped = templates.render_tool_result("tool", &serde_json::Value::Null, &msg.content)?;
                history.push(Message {
                    role: "user".to_string(),
                    content: wrapped,
                });
            } else {
                history.push(Message {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                });
            }
        }
        Ok((history, templates.render_followup()?))
    }
}
