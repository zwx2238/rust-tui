use super::sanitize::sanitize_tex;
use super::trace::write_tex_debug;
use std::process::Command;
use std::sync::OnceLock;

pub(crate) fn render_texicode(expr: &str) -> Result<String, String> {
    let expr = validate_expression(expr)?;
    ensure_txc_available()?;
    let raw = expr.to_string();
    let sanitized = sanitize_tex(expr);
    write_tex_debug(&raw, &sanitized);
    ensure_sanitized_non_empty(&sanitized)?;
    let output = run_txc(&sanitized)?;
    let text = parse_txc_output(&output)?;
    ensure_no_parsing_error(&text)?;
    Ok(text)
}

fn validate_expression(expr: &str) -> Result<&str, String> {
    let expr = expr.trim();
    if expr.is_empty() || expr.len() > 2000 {
        Err("表达式为空或过长".to_string())
    } else {
        Ok(expr)
    }
}

fn ensure_txc_available() -> Result<(), String> {
    if txc_available() {
        Ok(())
    } else {
        Err("未找到 txc".to_string())
    }
}

fn ensure_sanitized_non_empty(sanitized: &str) -> Result<(), String> {
    if sanitized.trim().is_empty() {
        Err("公式内容为空".to_string())
    } else {
        Ok(())
    }
}

fn run_txc(sanitized: &str) -> Result<std::process::Output, String> {
    Command::new("txc")
        .arg(sanitized)
        .output()
        .map_err(|e| format!("无法执行 txc：{e}"))
}

fn parse_txc_output(output: &std::process::Output) -> Result<String, String> {
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string();
        return Err(if err.is_empty() {
            "txc 执行失败".to_string()
        } else {
            err
        });
    }
    let text = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    if text.is_empty() {
        Err("txc 输出为空".to_string())
    } else {
        Ok(text)
    }
}

fn ensure_no_parsing_error(text: &str) -> Result<(), String> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("texicode: parsing error") {
        Err(text.to_string())
    } else {
        Ok(())
    }
}

fn txc_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| Command::new("txc").arg("--help").output().is_ok())
}
