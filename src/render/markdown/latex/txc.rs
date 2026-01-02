use super::sanitize::sanitize_tex;
use super::trace::write_tex_debug;
use std::process::Command;
use std::sync::OnceLock;

pub(crate) fn render_texicode(expr: &str) -> Result<String, String> {
    let expr = expr.trim();
    if expr.is_empty() || expr.len() > 2000 {
        return Err("表达式为空或过长".to_string());
    }
    if !txc_available() {
        return Err("未找到 txc".to_string());
    }
    let raw = expr.to_string();
    let sanitized = sanitize_tex(expr);
    write_tex_debug(&raw, &sanitized);
    if sanitized.trim().is_empty() {
        return Err("公式内容为空".to_string());
    }
    let output = Command::new("txc")
        .arg(&sanitized)
        .output()
        .map_err(|e| format!("无法执行 txc：{e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim_end().to_string();
        return Err(if err.is_empty() {
            "txc 执行失败".to_string()
        } else {
            err
        });
    }
    let text = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
    if text.is_empty() {
        return Err("txc 输出为空".to_string());
    }
    let lower = text.to_ascii_lowercase();
    if lower.contains("texicode: parsing error") {
        return Err(text);
    }
    Ok(text)
}

fn txc_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| Command::new("txc").arg("--help").output().is_ok())
}
