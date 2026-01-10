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
    if !should_process_math(text) {
        write_math_trace(text, None, true);
        return text.to_string();
    }
    let mut state = MathPreprocessor::new(text.len());
    for line in text.lines() {
        state.push_line(line);
    }
    state.finish(text.ends_with('\n'));
    write_math_trace(text, Some(&state.out), false);
    state.out
}

fn should_process_math(text: &str) -> bool {
    text.contains('$') || text.contains("\\(") || text.contains("\\[")
}

struct MathPreprocessor {
    out: String,
    in_fence: bool,
    in_math_block: Option<MathBlockKind>,
    math_buf: String,
    first_line: bool,
}

impl MathPreprocessor {
    fn new(capacity: usize) -> Self {
        Self {
            out: String::with_capacity(capacity),
            in_fence: false,
            in_math_block: None,
            math_buf: String::new(),
            first_line: true,
        }
    }

    fn push_line(&mut self, line: &str) {
        if !self.first_line {
            self.out.push('\n');
        }
        self.first_line = false;
        if self.handle_math_block_line(line) {
            return;
        }
        if self.handle_fence_line(line) {
            return;
        }
        if self.handle_dollar_start(line) {
            return;
        }
        if self.handle_bracket_start(line) {
            return;
        }
        self.out.push_str(&render_inline_math(line));
    }

    fn finish(&mut self, ended_with_newline: bool) {
        if let Some(kind) = self.in_math_block
            && !self.math_buf.trim().is_empty()
        {
            self.out.push('\n');
            append_math_block(&mut self.out, &self.math_buf, kind);
        }
        if ended_with_newline {
            self.out.push('\n');
        }
    }

    fn handle_math_block_line(&mut self, line: &str) -> bool {
        let Some(kind) = self.in_math_block else {
            return false;
        };
        match kind {
            MathBlockKind::Dollar => self.handle_block_line(line, "$$", kind),
            MathBlockKind::Bracket => self.handle_block_line(line, "\\]", kind),
        }
    }

    fn handle_block_line(&mut self, line: &str, end_marker: &str, kind: MathBlockKind) -> bool {
        if let Some(pos) = line.find(end_marker) {
            self.math_buf.push_str(&line[..pos]);
            append_math_block(&mut self.out, &self.math_buf, kind);
            self.math_buf.clear();
            self.in_math_block = None;
            let rest = &line[pos + end_marker.len()..];
            if !rest.trim().is_empty() {
                self.out.push('\n');
                self.out.push_str(&render_inline_math(rest));
            }
        } else {
            self.math_buf.push_str(line);
            self.math_buf.push('\n');
        }
        true
    }

    fn handle_fence_line(&mut self, line: &str) -> bool {
        if line.trim_start().starts_with("```") {
            self.in_fence = !self.in_fence;
            self.out.push_str(line);
            return true;
        }
        if self.in_fence {
            self.out.push_str(line);
            return true;
        }
        false
    }

    fn handle_dollar_start(&mut self, line: &str) -> bool {
        let Some(pos) = line.find("$$") else {
            return false;
        };
        let (before, after) = line.split_at(pos);
        self.write_before_math(before);
        let after = &after[2..];
        if let Some(end_pos) = after.find("$$") {
            let expr = &after[..end_pos];
            append_math_block(&mut self.out, expr, MathBlockKind::Dollar);
            self.write_after_math(&after[end_pos + 2..]);
        } else {
            self.in_math_block = Some(MathBlockKind::Dollar);
            self.math_buf.push_str(after);
        }
        true
    }

    fn handle_bracket_start(&mut self, line: &str) -> bool {
        let Some(pos) = line.find("\\[") else {
            return false;
        };
        let (before, after) = line.split_at(pos);
        self.write_before_math(before);
        let after = &after[2..];
        if let Some(end_pos) = after.find("\\]") {
            let expr = &after[..end_pos];
            append_math_block(&mut self.out, expr, MathBlockKind::Bracket);
            self.write_after_math(&after[end_pos + 2..]);
        } else {
            self.in_math_block = Some(MathBlockKind::Bracket);
            self.math_buf.push_str(after);
        }
        true
    }

    fn write_before_math(&mut self, before: &str) {
        if before.is_empty() {
            return;
        }
        self.out.push_str(before);
        if !before.trim().is_empty() {
            ensure_blank_line(&mut self.out);
        }
    }

    fn write_after_math(&mut self, rest: &str) {
        if !rest.trim().is_empty() {
            ensure_blank_line(&mut self.out);
            self.out.push_str(&render_inline_math(rest));
        }
    }
}
