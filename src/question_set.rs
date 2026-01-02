use std::env;
use std::fs;
use std::path::PathBuf;

pub fn load_question_set(spec: &str) -> Result<Vec<String>, String> {
    let (path, _) = resolve_question_set_path(spec)?;
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("读取问题集失败：{} ({e})", path.display()))?;
    let mut questions = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        questions.push(trimmed.to_string());
    }
    if questions.is_empty() {
        return Err(format!("问题集为空：{}", path.display()));
    }
    Ok(questions)
}

fn question_sets_dir() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME".to_string())?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("deepseek")
        .join("question_sets"))
}

fn resolve_question_set_path(spec: &str) -> Result<(PathBuf, bool), String> {
    let path = PathBuf::from(spec);
    let is_path =
        path.is_absolute() || path.components().count() > 1 || path.extension().is_some();
    if is_path {
        return Ok((path, true));
    }
    let dir = question_sets_dir()?;
    Ok((dir.join(format!("{spec}.txt")), false))
}
