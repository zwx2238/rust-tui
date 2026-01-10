use super::{errors::is_known_txc_error, txc::render_texicode};

#[derive(Copy, Clone)]
pub(super) enum MathBlockKind {
    Dollar,
    Bracket,
}

impl MathBlockKind {
    pub(super) fn wrap(self, expr: &str) -> String {
        match self {
            MathBlockKind::Dollar => format!("$$\n{expr}\n$$"),
            MathBlockKind::Bracket => format!("\\[\n{expr}\n\\]"),
        }
    }
}

pub(super) fn append_math_block(out: &mut String, expr: &str, kind: MathBlockKind) {
    if should_bypass_texicode(expr) {
        append_preprocessed_math(out, expr);
        return;
    }
    match render_texicode(expr) {
        Ok(text) => append_rendered_math(out, &text),
        Err(err) => append_latex_fallback(out, expr, kind, &err),
    }
}

fn should_bypass_texicode(expr: &str) -> bool {
    expr.contains("\\[") || expr.contains("\\]")
}

fn append_preprocessed_math(out: &mut String, expr: &str) {
    ensure_blank_line(out);
    out.push_str(&super::preprocess_math(expr));
    ensure_blank_line(out);
}

fn append_latex_fallback(out: &mut String, expr: &str, kind: MathBlockKind, err: &str) {
    let raw = kind.wrap(expr.trim());
    ensure_blank_line(out);
    out.push_str("```latex\n");
    out.push_str(&raw);
    if !raw.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n");
    if !is_known_txc_error(err) {
        out.push_str("[txc] 渲染失败：");
        out.push_str(err);
    }
}

fn append_rendered_math(out: &mut String, rendered: &str) {
    ensure_blank_line(out);
    if rendered.lines().count() > 1 {
        out.push_str("```math\n");
        out.push_str(rendered);
        if !rendered.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```");
        ensure_blank_line(out);
    } else {
        out.push_str(rendered);
    }
}

pub(super) fn ensure_blank_line(out: &mut String) {
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
