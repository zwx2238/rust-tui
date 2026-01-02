use super::{errors::is_known_txc_error, txc::render_texicode};

pub(super) fn render_inline_math(line: &str) -> String {
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
                    append_inline_render(&mut out, &rendered);
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
                    append_inline_render(&mut out, &rendered);
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

fn append_inline_render(out: &mut String, rendered: &str) {
    if rendered.contains('\n') {
        if !out.ends_with('\n') && !out.is_empty() {
            out.push('\n');
        }
        out.push('\n');
        out.push_str("```math\n");
        out.push_str(rendered);
        if !rendered.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n\n");
    } else {
        out.push_str(rendered);
    }
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
