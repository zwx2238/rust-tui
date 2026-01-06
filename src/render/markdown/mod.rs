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
mod tests;

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
