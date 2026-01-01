use crate::types::Message;
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
pub struct SessionTab {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub model_key: Option<String>,
    #[serde(default)]
    pub prompt_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub id: String,
    pub tabs: Vec<SessionTab>,
    #[serde(default)]
    pub active_tab: usize,
}

#[derive(Serialize, Deserialize)]
struct SessionV1 {
    id: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SessionFile {
    V2(SessionData),
    V1(SessionV1),
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

fn resolve_session_path(
    input: &str,
) -> Result<(PathBuf, bool), Box<dyn std::error::Error>> {
    let path = PathBuf::from(input);
    let is_path =
        path.is_absolute() || path.components().count() > 1 || path.extension().is_some();
    if is_path {
        return Ok((path, true));
    }
    let dir = sessions_dir()?;
    Ok((dir.join(format!("{input}.json")), false))
}

pub fn load_session(input: &str) -> Result<LoadedSession, Box<dyn std::error::Error>> {
    let (path, custom_path) = resolve_session_path(input)?;
    let text = fs::read_to_string(&path)?;
    let file: SessionFile = serde_json::from_str(&text)?;
    let mut data = match file {
        SessionFile::V2(data) => data,
        SessionFile::V1(v1) => SessionData {
            id: v1.id.clone(),
            tabs: vec![SessionTab {
                messages: v1.messages,
                model_key: None,
                prompt_key: None,
            }],
            active_tab: 0,
        },
    };
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
    tabs: &[SessionTab],
    active_tab: usize,
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
    let safe_active = if tabs.is_empty() {
        0
    } else {
        active_tab.min(tabs.len().saturating_sub(1))
    };
    let session = SessionData {
        id: id.clone(),
        tabs: tabs.to_vec(),
        active_tab: safe_active,
    };
    let text = serde_json::to_string_pretty(&session)?;
    fs::write(&path, text)?;
    Ok(SessionLocation {
        id,
        path,
        custom_path,
    })
}
