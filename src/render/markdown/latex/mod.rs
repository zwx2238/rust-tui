mod block;
mod errors;
mod inline;
mod sanitize;
mod trace;
mod txc;

use self::block::{MathBlockKind, append_math_block, ensure_blank_line};
use self::inline::render_inline_math;
use self::trace::write_math_trace;

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
