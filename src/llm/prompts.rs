use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct SystemPrompt {
    pub key: String,
    pub content: String,
}

#[derive(Clone)]
pub struct PromptRegistry {
    pub default_key: String,
    pub prompts: Vec<SystemPrompt>,
}

impl PromptRegistry {
    pub fn get(&self, key: &str) -> Option<&SystemPrompt> {
        self.prompts.iter().find(|p| p.key == key)
    }
}

pub fn load_prompts(
    dir: &str,
    default_key: &str,
    default_content: &str,
) -> Result<PromptRegistry, Box<dyn std::error::Error>> {
    let mut prompts = Vec::new();
    let dir_path = PathBuf::from(dir);
    if !dir_path.exists() {
        return Err(format!("提示词目录不存在：{}", dir_path.display()).into());
    }
    if !dir_path.is_dir() {
        return Err(format!("提示词路径不是目录：{}", dir_path.display()).into());
    }
    let entries = fs::read_dir(&dir_path)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(prompt) = read_prompt_file(&path) {
                prompts.push(prompt);
            }
        }
    }
    prompts.sort_by(|a, b| a.key.cmp(&b.key));
    if prompts.iter().all(|p| p.key != default_key) && !default_content.trim().is_empty() {
        prompts.insert(
            0,
            SystemPrompt {
                key: default_key.to_string(),
                content: default_content.to_string(),
            },
        );
    }
    let default_key = if prompts.iter().any(|p| p.key == default_key) {
        default_key.to_string()
    } else {
        prompts
            .first()
            .map(|p| p.key.clone())
            .unwrap_or_else(|| default_key.to_string())
    };
    Ok(PromptRegistry {
        default_key,
        prompts,
    })
}

fn read_prompt_file(path: &PathBuf) -> Option<SystemPrompt> {
    let content = fs::read_to_string(path).ok()?;
    let key = path.file_stem()?.to_string_lossy().to_string();
    if key.is_empty() {
        return None;
    }
    Some(SystemPrompt { key, content })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let dir = std::env::temp_dir().join(format!("deepchat_{name}_{ts}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn load_prompts_requires_directory() {
        let dir = temp_dir("prompts_missing");
        let _ = fs::remove_dir_all(&dir);
        let err = load_prompts(dir.to_string_lossy().as_ref(), "sys", "");
        let msg = match err {
            Ok(_) => String::new(),
            Err(e) => e.to_string(),
        };
        assert!(msg.contains("提示词目录不存在"));
    }

    #[test]
    fn load_prompts_sorts_and_injects_default() {
        let dir = temp_dir("prompts_ok");
        fs::write(dir.join("b.txt"), "B").unwrap();
        fs::write(dir.join("a.txt"), "A").unwrap();
        let registry = load_prompts(dir.to_string_lossy().as_ref(), "sys", "SYS").unwrap();
        assert_eq!(registry.default_key, "sys");
        assert_eq!(registry.prompts.len(), 3);
        assert_eq!(registry.prompts[0].key, "sys");
        assert_eq!(registry.prompts[1].key, "a");
        assert_eq!(registry.prompts[2].key, "b");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_prompts_falls_back_to_first_key() {
        let dir = temp_dir("prompts_fallback");
        fs::write(dir.join("b.txt"), "B").unwrap();
        fs::write(dir.join("a.txt"), "A").unwrap();
        let registry = load_prompts(dir.to_string_lossy().as_ref(), "sys", "").unwrap();
        assert_eq!(registry.default_key, "a");
        let _ = fs::remove_dir_all(&dir);
    }
}
