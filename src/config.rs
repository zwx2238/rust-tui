use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
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
    Ok(cfg)
}

pub fn load_config_optional(path: &PathBuf) -> Option<Config> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
}
