use crate::types::{ChatRequest, Message, StreamOptions, Usage};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;

pub enum LlmEvent {
    Chunk(String),
    Reasoning(String),
    Error(String),
    Done { usage: Option<Usage> },
}

pub struct UiEvent {
    pub tab: usize,
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
}

pub fn request_llm_stream(
    url: &str,
    api_key: &str,
    model: &str,
    show_reasoning: bool,
    messages: &[Message],
    tx: Sender<UiEvent>,
    tab: usize,
) {
    let client = reqwest::blocking::Client::new();
    let req = ChatRequest {
        model,
        messages,
        stream: true,
        stream_options: Some(StreamOptions {
            include_usage: true,
        }),
    };
    let resp = client.post(url).bearer_auth(api_key).json(&req).send();
    match resp {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                let _ = tx.send(UiEvent {
                    tab,
                    event: LlmEvent::Error(format!("请求失败：{status} {body}")),
                });
                return;
            }
            let mut reader = BufReader::new(resp);
            let mut line = String::new();
            let mut usage: Option<Usage> = None;
            loop {
                line.clear();
                let read = match reader.read_line(&mut line) {
                    Ok(n) => n,
                    Err(e) => {
                        let _ = tx.send(UiEvent {
                            tab,
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
                    let _ = tx.send(UiEvent {
                        tab,
                        event: LlmEvent::Done { usage },
                    });
                    return;
                }
                let parsed: Result<StreamResponse, _> = serde_json::from_str(data);
                match parsed {
                    Ok(chunk) => {
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
                                    event: LlmEvent::Chunk(content),
                                });
                            }
                            if show_reasoning {
                                if let Some(r) = choice.delta.reasoning_content {
                                    let _ = tx.send(UiEvent {
                                        tab,
                                        event: LlmEvent::Reasoning(r),
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(UiEvent {
                            tab,
                            event: LlmEvent::Error(format!("解析失败：{e}")),
                        });
                        return;
                    }
                }
            }
            let _ = tx.send(UiEvent {
                tab,
                event: LlmEvent::Done { usage },
            });
        }
        Err(e) => {
            let _ = tx.send(UiEvent {
                tab,
                event: LlmEvent::Error(format!("请求失败：{e}")),
            });
        }
    }
}
