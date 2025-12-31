use crate::types::Message;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct Session {
    id: String,
    messages: Vec<Message>,
}

fn sessions_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("deepseek")
        .join("sessions"))
}

pub fn save_session(messages: &[Message]) -> Result<String, Box<dyn std::error::Error>> {
    let dir = sessions_dir()?;
    fs::create_dir_all(&dir)?;
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间异常")?
        .as_secs()
        .to_string();
    let path = dir.join(format!("{id}.json"));
    let session = Session {
        id: id.clone(),
        messages: messages.to_vec(),
    };
    let text = serde_json::to_string_pretty(&session)?;
    fs::write(path, text)?;
    Ok(id)
}

pub fn load_session(id: &str) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(id);
    if path.extension().is_none() {
        let dir = sessions_dir()?;
        path = dir.join(format!("{id}.json"));
    }
    let text = fs::read_to_string(&path)?;
    let session: Session = serde_json::from_str(&text)?;
    Ok(session.messages)
}
