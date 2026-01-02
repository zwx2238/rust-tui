use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct SessionLocation {
    pub id: String,
    pub path: PathBuf,
    pub custom_path: bool,
}

impl SessionLocation {
    pub fn display_hint(&self) -> String {
        if self.custom_path {
            self.path.display().to_string()
        } else {
            self.id.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub id: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub active_category: String,
    #[serde(default)]
    pub open_conversations: Vec<String>,
    #[serde(default)]
    pub active_conversation: Option<String>,
}

pub struct LoadedSession {
    pub location: SessionLocation,
    pub data: SessionData,
}

fn sessions_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("deepseek")
        .join("sessions"))
}

fn resolve_session_path(input: &str) -> Result<(PathBuf, bool), Box<dyn std::error::Error>> {
    let path = PathBuf::from(input);
    let is_path = path.is_absolute() || path.components().count() > 1 || path.extension().is_some();
    if is_path {
        return Ok((path, true));
    }
    let dir = sessions_dir()?;
    Ok((dir.join(format!("{input}.json")), false))
}

pub fn load_session(input: &str) -> Result<LoadedSession, Box<dyn std::error::Error>> {
    let (path, custom_path) = resolve_session_path(input)?;
    let text = fs::read_to_string(&path)?;
    let raw: serde_json::Value = serde_json::from_str(&text)?;
    if raw.get("tabs").is_some() {
        return Err("会话格式已升级，不再兼容旧版 tabs 字段，请新建会话".into());
    }
    let mut data: SessionData = serde_json::from_value(raw)?;
    let id = if !data.id.trim().is_empty() {
        data.id.clone()
    } else if !custom_path {
        input.to_string()
    } else {
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "session".to_string())
    };
    if data.id.trim().is_empty() {
        data.id = id.clone();
    }
    if data.categories.is_empty() {
        data.categories = vec!["默认".to_string()];
    }
    if data.active_category.trim().is_empty() {
        data.active_category = data
            .categories
            .first()
            .cloned()
            .unwrap_or_else(|| "默认".to_string());
    }
    if !data.categories.contains(&data.active_category) {
        data.categories.push(data.active_category.clone());
    }
    Ok(LoadedSession {
        location: SessionLocation {
            id,
            path,
            custom_path,
        },
        data,
    })
}

pub fn save_session(
    categories: &[String],
    open_conversations: &[String],
    active_conversation: Option<&str>,
    active_category: Option<&str>,
    location: Option<&SessionLocation>,
) -> Result<SessionLocation, Box<dyn std::error::Error>> {
    let (id, path, custom_path) = if let Some(loc) = location {
        (loc.id.clone(), loc.path.clone(), loc.custom_path)
    } else {
        let dir = sessions_dir()?;
        fs::create_dir_all(&dir)?;
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "系统时间异常")?
            .as_secs()
            .to_string();
        let path = dir.join(format!("{id}.json"));
        (id, path, false)
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut categories = categories.to_vec();
    if categories.is_empty() {
        categories.push("默认".to_string());
    }
    let active_category = active_category
        .filter(|c| !c.trim().is_empty())
        .map(|c| c.to_string())
        .unwrap_or_else(|| categories[0].clone());
    if !categories.contains(&active_category) {
        categories.push(active_category.clone());
    }
    let session = SessionData {
        id: id.clone(),
        categories,
        active_category,
        open_conversations: open_conversations.to_vec(),
        active_conversation: active_conversation.map(|s| s.to_string()),
    };
    let text = serde_json::to_string_pretty(&session)?;
    fs::write(&path, text)?;
    Ok(SessionLocation {
        id,
        path,
        custom_path,
    })
}
