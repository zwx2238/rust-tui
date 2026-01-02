mod sanitize;
mod trace;
mod txc;

use self::trace::write_math_trace;
use self::txc::render_texicode;

#[derive(Copy, Clone)]
enum MathBlockKind {
    Dollar,
    Bracket,
}

impl MathBlockKind {
    fn wrap(self, expr: &str) -> String {
        match self {
            MathBlockKind::Dollar => format!("$$\n{expr}\n$$"),
            MathBlockKind::Bracket => format!("\\[\n{expr}\n\\]"),
        }
    }
}

pub(crate) fn preprocess_math(text: &str) -> String {
    let should_process = text.contains('$') || text.contains("\\(") || text.contains("\\[");
    if !should_process {
        write_math_trace(text, None, true);
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    let mut in_math_block: Option<MathBlockKind> = None;
    let mut math_buf = String::new();

    let mut first_line = true;
    for line in text.lines() {
        if !first_line {
            out.push('\n');
        }
        first_line = false;

        if let Some(kind) = in_math_block {
            match kind {
                MathBlockKind::Dollar => {
                    if let Some(pos) = line.find("$$") {
                        math_buf.push_str(&line[..pos]);
                        append_math_block(&mut out, &math_buf, kind);
                        math_buf.clear();
                        in_math_block = None;
                        let rest = &line[pos + 2..];
                        if !rest.trim().is_empty() {
                            out.push('\n');
                            out.push_str(&render_inline_math(rest));
                        }
                    } else {
                        math_buf.push_str(line);
                        math_buf.push('\n');
                    }
                }
                MathBlockKind::Bracket => {
                    if let Some(pos) = line.find("\\]") {
                        math_buf.push_str(&line[..pos]);
                        append_math_block(&mut out, &math_buf, kind);
                        math_buf.clear();
                        in_math_block = None;
                        let rest = &line[pos + 2..];
                        if !rest.trim().is_empty() {
                            out.push('\n');
                            out.push_str(&render_inline_math(rest));
                        }
                    } else {
                        math_buf.push_str(line);
                        math_buf.push('\n');
                    }
                }
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
                if !before.trim().is_empty() {
                    ensure_blank_line(&mut out);
                }
            }
            let after = &after[2..];
            if let Some(end_pos) = after.find("$$") {
                let expr = &after[..end_pos];
                append_math_block(&mut out, expr, MathBlockKind::Dollar);
                let rest = &after[end_pos + 2..];
                if !rest.trim().is_empty() {
                    ensure_blank_line(&mut out);
                    out.push_str(&render_inline_math(rest));
                }
            } else {
                in_math_block = Some(MathBlockKind::Dollar);
                math_buf.push_str(after);
            }
            continue;
        }
        if let Some(pos) = line.find("\\[") {
            let (before, after) = line.split_at(pos);
            if !before.is_empty() {
                out.push_str(before);
                if !before.trim().is_empty() {
                    ensure_blank_line(&mut out);
                }
            }
            let after = &after[2..];
            if let Some(end_pos) = after.find("\\]") {
                let expr = &after[..end_pos];
                append_math_block(&mut out, expr, MathBlockKind::Bracket);
                let rest = &after[end_pos + 2..];
                if !rest.trim().is_empty() {
                    ensure_blank_line(&mut out);
                    out.push_str(&render_inline_math(rest));
                }
            } else {
                in_math_block = Some(MathBlockKind::Bracket);
                math_buf.push_str(after);
            }
            continue;
        }

        out.push_str(&render_inline_math(line));
    }

    if let Some(kind) = in_math_block {
        if !math_buf.trim().is_empty() {
            out.push('\n');
            append_math_block(&mut out, &math_buf, kind);
        }
    }
    if text.ends_with('\n') {
        out.push('\n');
    }
    write_math_trace(text, Some(&out), false);
    out
}

fn append_math_block(out: &mut String, expr: &str, kind: MathBlockKind) {
    if expr.contains("\\[") || expr.contains("\\]") {
        ensure_blank_line(out);
        out.push_str(&preprocess_math(expr));
        ensure_blank_line(out);
        return;
    }
    let rendered = match render_texicode(expr) {
        Ok(text) => text,
        Err(err) => {
            let raw = kind.wrap(expr.trim());
            ensure_blank_line(out);
            out.push_str("```latex\n");
            out.push_str(&raw);
            if !raw.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n");
            if !is_known_txc_error(&err) {
                out.push_str("[txc] 渲染失败：");
                out.push_str(&err);
            }
            return;
        }
    };
    ensure_blank_line(out);
    if rendered.lines().count() > 1 {
        out.push_str("```math\n");
        out.push_str(&rendered);
        if !rendered.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```");
        ensure_blank_line(out);
    } else {
        out.push_str(&rendered);
    }
}

fn ensure_blank_line(out: &mut String) {
    if out.is_empty() {
        return;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
}

fn render_inline_math(line: &str) -> String {
    if !line.contains('$') && !line.contains("\\(") {
        return line.to_string();
    }
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i = 0usize;
    let mut last = 0usize;
    let mut in_code = false;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '`' {
            if last < i {
                out.push_str(&line[last..i]);
            }
            out.push('`');
            in_code = !in_code;
            i += 1;
            last = i;
            continue;
        }
        if ch == '\\' && !in_code && i + 1 < bytes.len() && bytes[i + 1] as char == '(' {
            if last < i {
                out.push_str(&line[last..i]);
            }
            if let Some(end) = find_inline_paren_end(bytes, i + 2) {
                let expr = &line[i + 2..end];
                if let Ok(rendered) = render_texicode(expr) {
                    if rendered.contains('\n') {
                        if !out.ends_with('\n') && !out.is_empty() {
                            out.push('\n');
                        }
                        out.push('\n');
                        out.push_str("```math\n");
                        out.push_str(&rendered);
                        if !rendered.ends_with('\n') {
                            out.push('\n');
                        }
                        out.push_str("```\n\n");
                    } else {
                        out.push_str(&rendered);
                    }
                } else if let Err(err) = render_texicode(expr) {
                    out.push('`');
                    out.push_str("\\(");
                    out.push_str(expr);
                    out.push_str("\\)");
                    out.push('`');
                    if !is_known_txc_error(&err) {
                        out.push('\n');
                        out.push_str("[txc] 渲染失败：");
                        out.push_str(&err);
                    }
                }
                i = end + 2;
                last = i;
                continue;
            }
        }
        if ch == '$' && !in_code && !is_escaped(bytes, i) {
            if i + 1 < bytes.len() && bytes[i + 1] as char == '$' {
                if last < i {
                    out.push_str(&line[last..i]);
                }
                out.push_str("$$");
                i += 2;
                last = i;
                continue;
            }
            if let Some(end) = find_inline_end(bytes, i + 1) {
                if last < i {
                    out.push_str(&line[last..i]);
                }
                let expr = &line[i + 1..end];
                if let Ok(rendered) = render_texicode(expr) {
                    if rendered.contains('\n') {
                        if !out.ends_with('\n') && !out.is_empty() {
                            out.push('\n');
                        }
                        out.push('\n');
                        out.push_str("```math\n");
                        out.push_str(&rendered);
                        if !rendered.ends_with('\n') {
                            out.push('\n');
                        }
                        out.push_str("```\n\n");
                    } else {
                        out.push_str(&rendered);
                    }
                } else if let Err(err) = render_texicode(expr) {
                    out.push('`');
                    out.push('$');
                    out.push_str(expr);
                    out.push('$');
                    out.push('`');
                    if !is_known_txc_error(&err) {
                        out.push('\n');
                        out.push_str("[txc] 渲染失败：");
                        out.push_str(&err);
                    }
                }
                i = end + 1;
                last = i;
                continue;
            }
        }
        i += 1;
    }
    if last < line.len() {
        out.push_str(&line[last..]);
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

fn find_inline_paren_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    while i + 1 < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '\\' && bytes[i + 1] as char == ')' && !is_escaped(bytes, i) {
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

fn is_known_txc_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("cmd_bgin")
        || lower.contains("expected ['cls_dlim']")
        || lower.contains("expected ['cls_dlim', 'cmd_lbrk']")
        || lower.contains("expected ['cls_dlim', 'cmd_lbrk', 'cls_line']")
}
