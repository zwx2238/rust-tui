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
    let is_path = path.is_absolute() || path.components().count() > 1 || path.extension().is_some();
    if is_path {
        return Ok((path, true));
    }
    let dir = question_sets_dir()?;
    Ok((dir.join(format!("{spec}.txt")), false))
}

#[cfg(test)]
mod tests {
    use super::load_question_set;
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
}
