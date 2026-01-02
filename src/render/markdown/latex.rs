use std::process::Command;
use std::sync::OnceLock;

pub(crate) fn preprocess_math(text: &str) -> String {
    if !text.contains('$') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    let mut in_math_block = false;
    let mut math_buf = String::new();

    let mut first_line = true;
    for line in text.lines() {
        if !first_line {
            out.push('\n');
        }
        first_line = false;

        if in_math_block {
            if let Some(pos) = line.find("$$") {
                math_buf.push_str(&line[..pos]);
                append_math_block(&mut out, &math_buf);
                math_buf.clear();
                in_math_block = false;
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() {
                    out.push('\n');
                    out.push_str(&render_inline_math(rest));
                }
            } else {
                math_buf.push_str(line);
                math_buf.push('\n');
            }
            continue;
        }

        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        if in_fence {
            out.push_str(line);
            continue;
        }

        if let Some(pos) = line.find("$$") {
            let (before, after) = line.split_at(pos);
            if !before.is_empty() {
                out.push_str(before);
            }
            let after = &after[2..];
            if let Some(end_pos) = after.find("$$") {
                let expr = &after[..end_pos];
                append_math_block(&mut out, expr);
                let rest = &after[end_pos + 2..];
                if !rest.trim().is_empty() {
                    out.push('\n');
                    out.push_str(&render_inline_math(rest));
                }
            } else {
                in_math_block = true;
                math_buf.push_str(after);
            }
            continue;
        }

        out.push_str(&render_inline_math(line));
    }

    if in_math_block && !math_buf.trim().is_empty() {
        out.push('\n');
        append_math_block(&mut out, &math_buf);
    }
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn append_math_block(out: &mut String, expr: &str) {
    let rendered = render_texicode(expr).unwrap_or_else(|| expr.trim().to_string());
    if !out.ends_with('\n') && !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&rendered);
}

fn render_inline_math(line: &str) -> String {
    if !line.contains('$') {
        return line.to_string();
    }
    let mut out = String::new();
    let mut i = 0;
    let bytes = line.as_bytes();
    let mut in_code = false;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '`' {
            in_code = !in_code;
            out.push(ch);
            i += 1;
            continue;
        }
        if ch == '$' && !in_code && !is_escaped(bytes, i) {
            if i + 1 < bytes.len() && bytes[i + 1] as char == '$' {
                out.push_str("$$");
                i += 2;
                continue;
            }
            if let Some(end) = find_inline_end(bytes, i + 1) {
                let expr = &line[i + 1..end];
                if let Some(rendered) = render_texicode(expr) {
                    out.push_str(&rendered);
                } else {
                    out.push('$');
                    out.push_str(expr);
                    out.push('$');
                }
                i = end + 1;
                continue;
            }
        }
        out.push(ch);
        i += 1;
    }
    out
}

fn find_inline_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '$' && !is_escaped(bytes, i) {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn is_escaped(bytes: &[u8], idx: usize) -> bool {
    if idx == 0 {
        return false;
    }
    let mut backslashes = 0;
    let mut i = idx;
    while i > 0 {
        i -= 1;
        if bytes[i] as char == '\\' {
            backslashes += 1;
        } else {
            break;
        }
    }
    backslashes % 2 == 1
}

fn render_texicode(expr: &str) -> Option<String> {
    let expr = expr.trim();
    if expr.is_empty() || expr.len() > 2000 {
        return None;
    }
    if !txc_available() {
        return None;
    }
    let output = Command::new("txc").arg(expr).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn txc_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| Command::new("txc").arg("--help").output().is_ok())
}
