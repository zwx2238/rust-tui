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
    let Some(dir) = trace_dir() else {
        return;
    };
    let path = trace_file_path(&dir, "math");
    let out = build_math_trace_output(raw, processed, skipped);
    let _ = fs::write(path, out);
}

fn trace_dir() -> Option<String> {
    let dir = env::var("DEEPCHAT_TEX_TRACE_DIR").ok()?;
    let dir = dir.trim();
    if dir.is_empty() || fs::create_dir_all(dir).is_err() {
        return None;
    }
    Some(dir.to_string())
}

fn trace_file_path(dir: &str, prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{}/{}_{}_{}.txt", dir, prefix, ts, seq)
}

fn build_math_trace_output(raw: &str, processed: Option<&str>, skipped: bool) -> String {
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
    out
}
