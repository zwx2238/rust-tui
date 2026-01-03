use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub theme: String,
    pub models: Vec<ModelItem>,
    pub default_model: String,
    pub prompts_dir: String,
    pub tavily_api_key: String,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ModelItem {
    pub key: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
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

fn validate_config(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if cfg.theme.trim().is_empty() {
        return Err("配置文件错误：theme 不能为空".into());
    }
    if cfg.prompts_dir.trim().is_empty() {
        return Err("配置文件错误：prompts_dir 不能为空".into());
    }
    if cfg.tavily_api_key.trim().is_empty() {
        return Err("配置文件错误：tavily_api_key 不能为空".into());
    }
    if cfg.models.is_empty() {
        return Err("配置文件错误：models 不能为空".into());
    }
    if cfg.default_model.trim().is_empty() {
        return Err("配置文件错误：default_model 不能为空".into());
    }
    if cfg.models.iter().any(|m| {
        m.key.trim().is_empty()
            || m.base_url.trim().is_empty()
            || m.api_key.trim().is_empty()
            || m.model.trim().is_empty()
    }) {
        return Err("配置文件错误：models 中每个条目必须包含 key/base_url/api_key/model".into());
    }
    if cfg.models.iter().all(|m| m.key != cfg.default_model) {
        return Err("配置文件错误：default_model 必须在 models 中存在".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::load_config;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("deepchat-{name}-{id}.json"))
    }

    #[test]
    fn load_config_ok() {
        let path = temp_path("ok");
        let text = r#"{
            "theme": "dark",
            "models": [{"key":"m1","base_url":"https://api.test/","api_key":"k","model":"m"}],
            "default_model": "m1",
            "prompts_dir": "/tmp/prompts",
            "tavily_api_key": "key"
        }"#;
        fs::write(&path, text).unwrap();
        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.default_model, "m1");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_config_missing_default_model() {
        let path = temp_path("bad");
        let text = r#"{
            "theme": "dark",
            "models": [{"key":"m1","base_url":"https://api.test/","api_key":"k","model":"m"}],
            "default_model": "missing",
            "prompts_dir": "/tmp/prompts",
            "tavily_api_key": "key"
        }"#;
        fs::write(&path, text).unwrap();
        let err = match load_config(&path) {
            Ok(_) => "expected error".to_string(),
            Err(e) => e.to_string(),
        };
        assert!(err.contains("default_model"));
        let _ = fs::remove_file(path);
    }
}
