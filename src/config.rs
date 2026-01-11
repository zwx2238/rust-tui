//! 配置管理模块
//!
//! 提供应用程序配置的加载、解析和管理功能。

use serde::{Deserialize, Serialize};
use crate::hooks::HookSpec;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub theme: String,
    pub models: Vec<ModelItem>,
    pub default_model: String,
    #[serde(default = "default_prompt_key")]
    pub default_prompt: String,
    #[serde(default)]
    pub hooks: Vec<HookSpec>,
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
    if let Some(root) = find_project_root() {
        return Ok(root.join("config").join("config.json"));
    }
    home_config_path()
}

pub fn load_config(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    load_env_file(path.as_path())?;
    let text = fs::read_to_string(path)?;
    let mut cfg: Config = serde_json::from_str(&text)?;
    normalize_config_paths(path.as_path(), &mut cfg);
    apply_env_overrides(&mut cfg);
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
    Ok(())
}

fn default_prompt_key() -> String {
    "default".to_string()
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
    if cfg.default_prompt.trim().is_empty() {
        return Err("配置文件错误：default_prompt 不能为空".into());
    }
    Ok(())
}

fn validate_models(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if cfg.models.iter().any(|m| {
        m.key.trim().is_empty()
            || m.base_url.trim().is_empty()
            || m.model.trim().is_empty()
    }) {
        return Err("配置文件错误：models 中每个条目必须包含 key/base_url/model".into());
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

fn find_project_root() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        if dir.join("Cargo.toml").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn home_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("deepseek")
        .join("config.json"))
}

fn normalize_config_paths(path: &Path, cfg: &mut Config) {
    let Some(parent) = path.parent() else {
        return;
    };
    if Path::new(&cfg.prompts_dir).is_relative() {
        cfg.prompts_dir = parent
            .join(&cfg.prompts_dir)
            .to_string_lossy()
            .to_string();
    }
}

fn apply_env_overrides(cfg: &mut Config) {
    apply_tavily_env(cfg);
    apply_model_env(cfg);
}

fn apply_tavily_env(cfg: &mut Config) {
    if let Some(name) = env_placeholder_name(&cfg.tavily_api_key) {
        cfg.tavily_api_key = env::var(name).unwrap_or_default();
        return;
    }
    if cfg.tavily_api_key.trim().is_empty()
        && let Ok(val) = env::var("DEEPCHAT_TAVILY_API_KEY")
    {
        cfg.tavily_api_key = val;
    }
}

fn apply_model_env(cfg: &mut Config) {
    for model in &mut cfg.models {
        if let Some(name) = env_placeholder_name(&model.api_key) {
            model.api_key = env::var(name).unwrap_or_default();
            continue;
        }
        if model.api_key.trim().is_empty() {
            let key = api_key_env_key(&model.key);
            if let Ok(val) = env::var(key) {
                model.api_key = val;
            }
        }
    }
}

fn env_placeholder_name(raw: &str) -> Option<&str> {
    raw.trim().strip_prefix("$ENV:")
}

fn api_key_env_key(key: &str) -> String {
    let mut out = String::from("DEEPCHAT_API_KEY_");
    for ch in key.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push('_');
        }
    }
    out
}

fn load_env_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let env_path = parent.join(".env");
    if !env_path.exists() {
        return Ok(());
    }
    let text = fs::read_to_string(env_path)?;
    for line in text.lines() {
        if let Some((key, value)) = parse_env_line(line) {
            set_env_if_missing(&key, &value);
        }
    }
    Ok(())
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let line = line.strip_prefix("export ").unwrap_or(line).trim();
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key.to_string(), trim_quotes(value.trim()).to_string()))
}

fn trim_quotes(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"' {
        return &value[1..value.len() - 1];
    }
    if bytes.len() >= 2 && bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'' {
        return &value[1..value.len() - 1];
    }
    value
}

fn set_env_if_missing(key: &str, value: &str) {
    if env::var(key).is_err() {
        unsafe {
            env::set_var(key, value);
        }
    }
}
