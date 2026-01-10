use std::fs;
use std::path::{Path, PathBuf};

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
    let dir_path = PathBuf::from(dir);
    ensure_prompts_dir(&dir_path)?;
    let mut prompts = read_prompts_from_dir(&dir_path)?;
    prompts.sort_by(|a, b| a.key.cmp(&b.key));
    inject_default_prompt(&mut prompts, default_key, default_content);
    let default_key = select_default_key(&prompts, default_key);
    Ok(PromptRegistry {
        default_key,
        prompts,
    })
}

fn ensure_prompts_dir(dir_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dir_path.exists() {
        return Err(format!("提示词目录不存在：{}", dir_path.display()).into());
    }
    if !dir_path.is_dir() {
        return Err(format!("提示词路径不是目录：{}", dir_path.display()).into());
    }
    Ok(())
}

fn read_prompts_from_dir(dir_path: &Path) -> Result<Vec<SystemPrompt>, Box<dyn std::error::Error>> {
    let mut prompts = Vec::new();
    let entries = fs::read_dir(dir_path)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(prompt) = read_prompt_file(&path)
        {
            prompts.push(prompt);
        }
    }
    Ok(prompts)
}

fn inject_default_prompt(prompts: &mut Vec<SystemPrompt>, key: &str, content: &str) {
    let exists = prompts.iter().any(|p| p.key == key);
    if exists || content.trim().is_empty() {
        return;
    }
    prompts.insert(
        0,
        SystemPrompt {
            key: key.to_string(),
            content: content.to_string(),
        },
    );
}

fn select_default_key(prompts: &[SystemPrompt], key: &str) -> String {
    if prompts.iter().any(|p| p.key == key) {
        return key.to_string();
    }
    prompts
        .first()
        .map(|p| p.key.clone())
        .unwrap_or_else(|| key.to_string())
}

fn read_prompt_file(path: &PathBuf) -> Option<SystemPrompt> {
    let content = fs::read_to_string(path).ok()?;
    let key = path.file_stem()?.to_string_lossy().to_string();
    if key.is_empty() {
        return None;
    }
    Some(SystemPrompt { key, content })
}
