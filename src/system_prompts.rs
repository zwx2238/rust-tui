use crate::prompt_pack::ensure_prompt_pack;
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
    ensure_prompt_pack(&dir_path)?;
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
