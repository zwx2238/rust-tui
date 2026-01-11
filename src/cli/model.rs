use crate::config::{Config, ModelItem, default_config_path, load_config, save_config};
use std::io::{self, Write};
use std::path::PathBuf;

pub(crate) fn run_add(cfg_override: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let path = resolve_config_path(cfg_override)?;
    let mut cfg = load_or_create_config(&path)?;
    let item = prompt_model_item()?;
    upsert_model(&mut cfg, item)?;
    save_config(&path, &cfg)?;
    println!("已写入配置：{}", path.display());
    Ok(())
}

fn resolve_config_path(cfg_override: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    match cfg_override {
        Some(p) => Ok(PathBuf::from(p)),
        None => default_config_path(),
    }
}

fn load_or_create_config(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    if path.exists() {
        return load_config(path);
    }
    let parent = path.parent().ok_or("配置路径无效：缺少父目录")?;
    std::fs::create_dir_all(parent)?;
    let prompts_dir = parent.join("prompts");
    std::fs::create_dir_all(&prompts_dir)?;
    Ok(default_config(prompts_dir.to_string_lossy().as_ref()))
}

fn default_config(prompts_dir: &str) -> Config {
    Config {
        theme: "default".to_string(),
        models: Vec::new(),
        default_model: String::new(),
        default_prompt: "default".to_string(),
        hooks: Vec::new(),
        prompts_dir: prompts_dir.to_string(),
        tavily_api_key: String::new(),
    }
}

fn prompt_model_item() -> Result<ModelItem, Box<dyn std::error::Error>> {
    let key = prompt_non_empty("模型 key（如 m1）: ")?;
    let base_url = prompt_non_empty("API Base URL（如 https://api.deepseek.com）: ")?;
    let model = prompt_non_empty("模型名称（如 deepseek-chat）: ")?;
    let max_tokens = prompt_optional_u64("max_tokens（可选，回车跳过；Anthropic 必填）: ")?;
    let api_key = prompt_non_empty("API Key: ")?;
    Ok(ModelItem {
        key,
        base_url,
        api_key,
        model,
        max_tokens,
    })
}

fn prompt_optional_u64(prompt: &str) -> Result<Option<u64>, Box<dyn std::error::Error>> {
    loop {
        let s = prompt_line(prompt)?;
        let s = s.trim();
        if s.is_empty() {
            return Ok(None);
        }
        if let Ok(v) = s.parse::<u64>() {
            return Ok(Some(v));
        }
    }
}

fn upsert_model(cfg: &mut Config, item: ModelItem) -> Result<(), Box<dyn std::error::Error>> {
    let key = item.key.clone();
    if let Some(existing) = cfg.models.iter_mut().find(|m| m.key == item.key) {
        if !confirm("该 key 已存在，是否覆盖？[y/N] ")? {
            return Err("已取消".into());
        }
        *existing = item;
    } else {
        cfg.models.push(item);
    }
    let should_set_default =
        cfg.default_model.trim().is_empty() || confirm("设为默认模型？[y/N] ")?;
    if should_set_default {
        cfg.default_model = key;
    }
    Ok(())
}

fn prompt_non_empty(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let s = prompt_line(prompt)?;
        if !s.trim().is_empty() {
            return Ok(s);
        }
    }
}

fn confirm(prompt: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let s = prompt_line(prompt)?;
    let s = s.trim().to_ascii_lowercase();
    Ok(matches!(s.as_str(), "y" | "yes"))
}

fn prompt_line(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut out = io::stdout();
    out.write_all(prompt.as_bytes())?;
    out.flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim_end().to_string())
}
