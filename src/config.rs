//! 配置管理模块
//!
//! 提供应用程序配置的加载、解析和管理功能。

use serde::{Deserialize, Serialize};
use crate::types::{ROLE_ASSISTANT, ROLE_REASONING, ROLE_SYSTEM, ROLE_TOOL, ROLE_USER};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub theme: String,
    pub models: Vec<ModelItem>,
    pub default_model: String,
    #[serde(default = "default_role")]
    pub default_role: String,
    pub prompts_dir: String,
    pub tavily_api_key: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ModelItem {
    pub key: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: Option<u64>,
}

pub fn default_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("deepseek")
        .join("config.json"))
}

pub fn load_config(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let cfg: Config = serde_json::from_str(&text)?;
    validate_config(&cfg)?;
    Ok(cfg)
}

pub fn save_config(path: &PathBuf, cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    validate_config(cfg)?;
    let parent = path.parent().ok_or("配置路径无效：缺少父目录")?;
    fs::create_dir_all(parent)?;
    let text = serde_json::to_string_pretty(cfg)?;
    fs::write(path, text)?;
    Ok(())
}

fn validate_config(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    validate_required_fields(cfg)?;
    validate_models(cfg)?;
    validate_default_role(cfg)?;
    Ok(())
}

fn validate_required_fields(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if cfg.theme.trim().is_empty() {
        return Err("配置文件错误：theme 不能为空".into());
    }
    if cfg.prompts_dir.trim().is_empty() {
        return Err("配置文件错误：prompts_dir 不能为空".into());
    }
    if cfg.models.is_empty() {
        return Err("配置文件错误：models 不能为空".into());
    }
    if cfg.default_model.trim().is_empty() {
        return Err("配置文件错误：default_model 不能为空".into());
    }
    Ok(())
}

fn validate_models(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if cfg.models.iter().any(|m| {
        m.key.trim().is_empty()
            || m.base_url.trim().is_empty()
            || m.api_key.trim().is_empty()
            || m.model.trim().is_empty()
    }) {
        return Err("配置文件错误：models 中每个条目必须包含 key/base_url/api_key/model".into());
    }
    if cfg
        .models
        .iter()
        .any(|m| matches!(m.max_tokens, Some(0)))
    {
        return Err("配置文件错误：max_tokens 不能为 0".into());
    }
    if cfg.models.iter().all(|m| m.key != cfg.default_model) {
        return Err("配置文件错误：default_model 必须在 models 中存在".into());
    }
    Ok(())
}

fn validate_default_role(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    normalize_role(&cfg.default_role)
        .map(|_| ())
        .map_err(|e| format!("配置文件错误：default_role {e}").into())
}

fn default_role() -> String {
    ROLE_USER.to_string()
}

pub fn normalize_role(raw: &str) -> Result<String, String> {
    let role = raw.trim().to_ascii_lowercase();
    if role.is_empty() {
        return Err("不能为空".to_string());
    }
    match role.as_str() {
        ROLE_USER | ROLE_ASSISTANT | ROLE_REASONING | ROLE_SYSTEM | ROLE_TOOL => Ok(role),
        _ => Err("必须为 user/assistant/reasoning/system/tool".to_string()),
    }
}
