use crate::types::{
    ChatRequest, Message, StreamOptions, ToolCall, ToolDefinition, ToolFunctionCall,
    ToolFunctionDef, Usage,
};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub enum LlmEvent {
    Chunk(String),
    Reasoning(String),
    Error(String),
    Done { usage: Option<Usage> },
    ToolCalls { calls: Vec<ToolCall>, usage: Option<Usage> },
}

pub struct UiEvent {
    pub tab: usize,
    pub request_id: u64,
    pub event: LlmEvent,
}

#[derive(serde::Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(serde::Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(serde::Deserialize)]
struct StreamDelta {
    content: Option<String>,
    #[serde(rename = "reasoning_content")]
    reasoning_content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(serde::Deserialize)]
struct ToolCallDelta {
    index: usize,
    id: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    function: Option<ToolFunctionDelta>,
}

#[derive(serde::Deserialize)]
struct ToolFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

pub fn request_llm_stream(
    url: &str,
    api_key: &str,
    model: &str,
    show_reasoning: bool,
    messages: &[Message],
    cancel: Arc<AtomicBool>,
    tx: Sender<UiEvent>,
    tab: usize,
    request_id: u64,
) {
    let client = reqwest::blocking::Client::new();
    let req = ChatRequest {
        model,
        messages,
        stream: true,
        stream_options: Some(StreamOptions {
            include_usage: true,
        }),
        tools: Some(vec![web_search_tool_def()]),
        tool_choice: None,
    };
    let resp = client.post(url).bearer_auth(api_key).json(&req).send();
    match resp {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                let _ = tx.send(UiEvent {
                    tab,
                    request_id,
                    event: LlmEvent::Error(format!("请求失败：{status} {body}")),
                });
                return;
            }
            let mut reader = BufReader::new(resp);
            let mut line = String::new();
            let mut usage: Option<Usage> = None;
            let mut tool_calls: Vec<ToolCall> = Vec::new();
            loop {
                if cancel.load(Ordering::Relaxed) {
                    return;
                }
                line.clear();
                let read = match reader.read_line(&mut line) {
                    Ok(n) => n,
                    Err(e) => {
                        let _ = tx.send(UiEvent {
                            tab,
                            request_id,
                            event: LlmEvent::Error(format!("读取失败：{e}")),
                        });
                        return;
                    }
                };
                if read == 0 {
                    break;
                }
                let trimmed = line.trim_end();
                if trimmed.is_empty() {
                    continue;
                }
                let Some(data) = trimmed.strip_prefix("data: ") else {
                    continue;
                };
                if data == "[DONE]" {
                    if tool_calls.is_empty() {
                        let _ = tx.send(UiEvent {
                            tab,
                            request_id,
                            event: LlmEvent::Done { usage },
                        });
                    } else {
                        let _ = tx.send(UiEvent {
                            tab,
                            request_id,
                            event: LlmEvent::ToolCalls { calls: tool_calls, usage },
                        });
                    }
                    return;
                }
                let parsed: Result<StreamResponse, _> = serde_json::from_str(data);
                match parsed {
                    Ok(chunk) => {
                        if cancel.load(Ordering::Relaxed) {
                            return;
                        }
                        if let Some(u) = chunk.usage {
                            usage = Some(u);
                        } else {
                            for choice in &chunk.choices {
                                if let Some(u) = &choice.usage {
                                    usage = Some(u.clone());
                                }
                            }
                        }
                        for choice in chunk.choices {
                            if let Some(content) = choice.delta.content {
                                let _ = tx.send(UiEvent {
                                    tab,
                                    request_id,
                                    event: LlmEvent::Chunk(content),
                                });
                            }
                            if show_reasoning {
                                if let Some(r) = choice.delta.reasoning_content {
                                    let _ = tx.send(UiEvent {
                                        tab,
                                        request_id,
                                        event: LlmEvent::Reasoning(r),
                                    });
                                }
                            }
                            if let Some(delta_calls) = choice.delta.tool_calls {
                                merge_tool_calls(&mut tool_calls, delta_calls);
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(UiEvent {
                            tab,
                            request_id,
                            event: LlmEvent::Error(format!("解析失败：{e}")),
                        });
                        return;
                    }
                }
            }
            if tool_calls.is_empty() {
                let _ = tx.send(UiEvent {
                    tab,
                    request_id,
                    event: LlmEvent::Done { usage },
                });
            } else {
                let _ = tx.send(UiEvent {
                    tab,
                    request_id,
                    event: LlmEvent::ToolCalls { calls: tool_calls, usage },
                });
            }
        }
        Err(e) => {
            let _ = tx.send(UiEvent {
                tab,
                request_id,
                event: LlmEvent::Error(format!("请求失败：{e}")),
            });
        }
    }
}

fn merge_tool_calls(target: &mut Vec<ToolCall>, deltas: Vec<ToolCallDelta>) {
    for delta in deltas {
        if target.len() <= delta.index {
            target.resize_with(delta.index + 1, || ToolCall {
                id: String::new(),
                kind: "function".to_string(),
                function: ToolFunctionCall {
                    name: String::new(),
                    arguments: String::new(),
                },
            });
        }
        let entry = &mut target[delta.index];
        if let Some(id) = delta.id {
            if entry.id.is_empty() {
                entry.id = id;
            }
        }
        if let Some(kind) = delta.kind {
            entry.kind = kind;
        }
        if let Some(func) = delta.function {
            if let Some(name) = func.name {
                if entry.function.name.is_empty() {
                    entry.function.name = name;
                }
            }
            if let Some(args) = func.arguments {
                entry.function.arguments.push_str(&args);
            }
        }
    }
}

fn web_search_tool_def() -> ToolDefinition {
    let params = serde_json::json!({
        "type": "object",
        "properties": {
            "query": { "type": "string" },
            "top_k": { "type": "integer", "minimum": 1, "maximum": 10 }
        },
        "required": ["query"]
    });
    ToolDefinition {
        kind: "function".to_string(),
        function: ToolFunctionDef {
            name: "web_search".to_string(),
            description: "Search the web and return a short list of results.".to_string(),
            parameters: params,
        },
    }
}
