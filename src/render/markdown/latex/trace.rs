use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

pub(crate) fn write_tex_debug(raw: &str, sanitized: &str) {
    let Ok(dir) = env::var("DEEPCHAT_TEX_DEBUG_DIR") else {
        return;
    };
    let dir = dir.trim();
    if dir.is_empty() {
        return;
    }
    if fs::create_dir_all(dir).is_err() {
        return;
    }
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = format!("{}/tex_{}_{}.txt", dir, ts, seq);
    let mut out = String::new();
    out.push_str("--- raw ---\n");
    out.push_str(raw);
    out.push('\n');
    out.push_str("--- sanitized ---\n");
    out.push_str(sanitized);
    out.push('\n');
    let _ = fs::write(path, out);
}

pub(crate) fn write_math_trace(raw: &str, processed: Option<&str>, skipped: bool) {
    let Ok(dir) = env::var("DEEPCHAT_TEX_TRACE_DIR") else {
        return;
    };
    let dir = dir.trim();
    if dir.is_empty() {
        return;
    }
    if fs::create_dir_all(dir).is_err() {
        return;
    }
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = format!("{}/math_{}_{}.txt", dir, ts, seq);
    let mut out = String::new();
    out.push_str("skipped: ");
    out.push_str(if skipped { "true" } else { "false" });
    out.push('\n');
    out.push_str("--- raw ---\n");
    out.push_str(raw);
    out.push('\n');
    if let Some(text) = processed {
        out.push_str("--- processed ---\n");
        out.push_str(text);
        out.push('\n');
    }
    let _ = fs::write(path, out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{env_lock, restore_env, set_env};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let dir = std::env::temp_dir().join(format!("deepchat_{name}_{ts}"));
        fs::create_dir_all(&dir).unwrap();
        dir.to_string_lossy().to_string()
    }

    #[test]
    fn writes_tex_debug_and_trace_files() {
        let guard = env_lock().lock().unwrap();
        let dir = temp_dir("tex_trace");
        let prev_debug = set_env("DEEPCHAT_TEX_DEBUG_DIR", &dir);
        let prev_trace = set_env("DEEPCHAT_TEX_TRACE_DIR", &dir);

        write_tex_debug("raw", "sanitized");
        write_math_trace("raw", Some("processed"), false);

        restore_env("DEEPCHAT_TEX_DEBUG_DIR", prev_debug);
        restore_env("DEEPCHAT_TEX_TRACE_DIR", prev_trace);
        drop(guard);

        let entries = fs::read_dir(&dir).unwrap().count();
        assert!(entries >= 2);
        let _ = fs::remove_dir_all(&dir);
    }
}
