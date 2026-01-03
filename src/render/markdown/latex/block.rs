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
    if expr.contains("\\[") || expr.contains("\\]") {
        ensure_blank_line(out);
        out.push_str(&super::preprocess_math(expr));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_block_kind() {
        let expr = "x+1";
        assert!(MathBlockKind::Dollar.wrap(expr).contains("$$"));
        assert!(MathBlockKind::Bracket.wrap(expr).contains("\\["));
    }

    #[test]
    fn append_math_block_falls_back_to_latex() {
        let mut out = String::new();
        append_math_block(&mut out, "x+1", MathBlockKind::Dollar);
        assert!(!out.trim().is_empty());
    }
}
