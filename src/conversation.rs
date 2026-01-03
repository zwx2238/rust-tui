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

#[cfg(test)]
mod tests {
    use super::{ConversationData, conversation_path, load_conversation, save_conversation};
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn set_home(temp: &std::path::Path) -> Option<String> {
        let prev = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", temp.to_string_lossy().to_string()) };
        prev
    }

    fn restore_home(prev: Option<String>) {
        if let Some(val) = prev {
            unsafe { std::env::set_var("HOME", val) };
        }
    }

    #[test]
    fn save_and_load_conversation() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-conv-test");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);
        let data = ConversationData {
            id: "abc".to_string(),
            category: "默认".to_string(),
            messages: Vec::new(),
            model_key: None,
            prompt_key: None,
            code_exec_container_id: None,
        };
        let path = save_conversation(&data).unwrap();
        let loaded = load_conversation("abc").unwrap();
        assert_eq!(loaded.id, "abc");
        assert_eq!(path, conversation_path("abc").unwrap());
        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }
}
