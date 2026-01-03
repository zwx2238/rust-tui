mod code;
mod count;
mod latex;
mod list;
mod render;
mod render_state;
mod shared;
mod table;
mod text;

pub(crate) use count::count_markdown_lines;
pub(crate) use latex::preprocess_math;
pub use render::render_markdown_lines;

#[cfg(test)]
mod table_tests;

pub(crate) fn close_unbalanced_code_fence(input: &str) -> String {
    let mut fence_count = 0usize;
    for line in input.lines() {
        if line.trim_start().starts_with("```") {
            fence_count += 1;
        }
    }
    if fence_count % 2 == 1 {
        let mut out = String::with_capacity(input.len() + 4);
        out.push_str(input);
        if !input.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```");
        out
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{close_unbalanced_code_fence, count_markdown_lines, preprocess_math};

    #[test]
    fn closes_unbalanced_fence() {
        let input = "```\ncode";
        let out = close_unbalanced_code_fence(input);
        assert!(out.ends_with("```"));
    }

    #[test]
    fn count_markdown_lines_basic() {
        let text = "# Title\n\n- a\n- b\n\n`code`";
        let count = count_markdown_lines(text, 40);
        assert!(count > 0);
    }

    #[test]
    fn preprocess_math_keeps_plain() {
        let input = "no math here";
        let out = preprocess_math(input);
        assert_eq!(out, input);
    }
}
