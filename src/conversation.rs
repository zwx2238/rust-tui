//! 对话管理模块
//!
//! 处理对话会话的创建、保存、加载和管理功能。

use crate::types::Message;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
pub struct ConversationData {
    pub id: String,
    #[serde(default)]
    pub category: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub model_key: Option<String>,
    #[serde(default)]
    pub prompt_key: Option<String>,
    #[serde(default)]
    pub code_exec_container_id: Option<String>,
}

pub fn conversations_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("deepseek")
        .join("conversations"))
}

pub fn new_conversation_id() -> Result<String, Box<dyn std::error::Error>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间异常")?;
    Ok(format!("{}{}", now.as_secs(), now.subsec_micros()))
}

pub fn conversation_path(id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = conversations_dir()?;
    Ok(dir.join(format!("{id}.json")))
}

pub fn load_conversation(id: &str) -> Result<ConversationData, Box<dyn std::error::Error>> {
    let path = conversation_path(id)?;
    let text = fs::read_to_string(&path)?;
    let mut data: ConversationData = serde_json::from_str(&text)?;
    if data.id.trim().is_empty() {
        data.id = id.to_string();
    }
    Ok(data)
}

pub fn save_conversation(data: &ConversationData) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = conversation_path(&data.id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(data)?;
    fs::write(&path, text)?;
    Ok(path)
}
