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

pub fn list_question_sets() -> Result<Vec<String>, String> {
    let dir = question_sets_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("读取问题集目录失败：{} ({e})", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取问题集目录失败：{} ({e})", dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("txt") {
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            out.push(stem.to_string());
        }
    }
    out.sort();
    Ok(out)
}

pub fn question_sets_dir() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|_| "无法确定 HOME".to_string())?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("deepseek")
        .join("question_sets"))
}

fn resolve_question_set_path(spec: &str) -> Result<(PathBuf, bool), String> {
    let path = PathBuf::from(spec);
    let is_path = path.is_absolute() || path.components().count() > 1 || path.extension().is_some();
    if is_path {
        return Ok((path, true));
    }
    let dir = question_sets_dir()?;
    Ok((dir.join(format!("{spec}.txt")), false))
}

#[cfg(test)]
mod tests {
    use super::{list_question_sets, load_question_set, question_sets_dir};
    use crate::test_support::{env_lock, restore_env, set_env};
    use std::fs;

    fn set_home(temp: &std::path::Path) -> Option<String> {
        set_env("HOME", &temp.to_string_lossy())
    }

    fn restore_home(prev: Option<String>) {
        restore_env("HOME", prev);
    }

    #[test]
    fn load_named_question_set() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-qs-test");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);
        let path = temp
            .join(".config")
            .join("deepseek")
            .join("question_sets")
            .join("latex.txt");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, "# comment\nQ1\n\nQ2\n").unwrap();
        let qs = load_question_set("latex").unwrap();
        assert_eq!(qs, vec!["Q1".to_string(), "Q2".to_string()]);
        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn empty_question_set_errors() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-qs-empty");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);
        let path = temp
            .join(".config")
            .join("deepseek")
            .join("question_sets")
            .join("empty.txt");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, "# only comment\n").unwrap();
        let err = load_question_set("empty").unwrap_err();
        assert!(err.contains("问题集为空"));
        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn list_question_sets_sorted() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-qs-list");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);
        let dir = question_sets_dir().unwrap();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("b.txt"), "Q1\n").unwrap();
        fs::write(dir.join("a.txt"), "Q1\n").unwrap();
        fs::write(dir.join("c.md"), "Q1\n").unwrap();
        let list = list_question_sets().unwrap();
        assert_eq!(list, vec!["a".to_string(), "b".to_string()]);
        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }
}
