use crate::types::{ChatRequest, ChatResponse, Message, Usage};

pub struct LlmResult {
    pub assistant: Option<String>,
    pub reasoning: Option<String>,
    pub error: Option<String>,
    pub usage: Option<Usage>,
}

pub fn request_llm(
    url: &str,
    api_key: &str,
    model: &str,
    show_reasoning: bool,
    messages: &[Message],
) -> LlmResult {
    let client = reqwest::blocking::Client::new();
    let req = ChatRequest {
        model,
        messages,
        stream: false,
    };
    let resp = client.post(url).bearer_auth(api_key).json(&req).send();
    match resp {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                return LlmResult {
                    assistant: None,
                    reasoning: None,
                    error: Some(format!("请求失败：{status} {body}")),
                    usage: None,
                };
            }
            let data: Result<ChatResponse, _> = resp.json();
            match data {
                Ok(data) => {
                    let Some(choice) = data.choices.into_iter().next() else {
                        return LlmResult {
                            assistant: None,
                            reasoning: None,
                            error: Some("响应中没有 choices。".to_string()),
                            usage: None,
                        };
                    };
                    let reasoning = if show_reasoning {
                        choice.message.reasoning_content.clone()
                    } else {
                        None
                    };
                    LlmResult {
                        assistant: Some(choice.message.content.unwrap_or_default()),
                        reasoning,
                        error: None,
                        usage: data.usage.clone(),
                    }
                }
                Err(e) => LlmResult {
                    assistant: None,
                    reasoning: None,
                    error: Some(format!("解析失败：{e}")),
                    usage: None,
                },
            }
        }
        Err(e) => LlmResult {
            assistant: None,
            reasoning: None,
            error: Some(format!("请求失败：{e}")),
            usage: None,
        },
    }
}
