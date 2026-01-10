use super::{errors::is_known_txc_error, txc::render_texicode};

pub(super) fn render_inline_math(line: &str) -> String {
    if !line.contains('$') && !line.contains("\\(") {
        return line.to_string();
    }
    let mut state = InlineMathState::new(line);
    state.process();
    state.finish()
}

struct InlineMathState<'a> {
    line: &'a str,
    bytes: &'a [u8],
    out: String,
    i: usize,
    last: usize,
    in_code: bool,
}

impl<'a> InlineMathState<'a> {
    fn new(line: &'a str) -> Self {
        Self {
            line,
            bytes: line.as_bytes(),
            out: String::with_capacity(line.len()),
            i: 0,
            last: 0,
            in_code: false,
        }
    }

    fn process(&mut self) {
        while self.i < self.bytes.len() {
            let ch = self.bytes[self.i] as char;
            if self.handle_backtick(ch) {
                continue;
            }
            if self.handle_paren_math(ch) {
                continue;
            }
            if self.handle_dollar_math(ch) {
                continue;
            }
            self.i += 1;
        }
    }

    fn finish(mut self) -> String {
        if self.last < self.line.len() {
            self.out.push_str(&self.line[self.last..]);
        }
        self.out
    }

    fn handle_backtick(&mut self, ch: char) -> bool {
        if ch != '`' {
            return false;
        }
        if self.last < self.i {
            self.out.push_str(&self.line[self.last..self.i]);
        }
        self.out.push('`');
        self.in_code = !self.in_code;
        self.i += 1;
        self.last = self.i;
        true
    }

    fn handle_paren_math(&mut self, ch: char) -> bool {
        if ch != '\\' || self.in_code || self.i + 1 >= self.bytes.len() {
            return false;
        }
        if self.bytes[self.i + 1] as char != '(' {
            return false;
        }
        if self.last < self.i {
            self.out.push_str(&self.line[self.last..self.i]);
        }
        if let Some(end) = find_inline_paren_end(self.bytes, self.i + 2) {
            self.render_inline_expr(self.i + 2, end, "\\(", "\\)");
            self.i = end + 2;
            self.last = self.i;
            return true;
        }
        false
    }

    fn handle_dollar_math(&mut self, ch: char) -> bool {
        if ch != '$' || self.in_code || is_escaped(self.bytes, self.i) {
            return false;
        }
        if self.i + 1 < self.bytes.len() && self.bytes[self.i + 1] as char == '$' {
            if self.last < self.i {
                self.out.push_str(&self.line[self.last..self.i]);
            }
            self.out.push_str("$$");
            self.i += 2;
            self.last = self.i;
            return true;
        }
        if let Some(end) = find_inline_end(self.bytes, self.i + 1) {
            if self.last < self.i {
                self.out.push_str(&self.line[self.last..self.i]);
            }
            self.render_inline_expr(self.i + 1, end, "$", "$");
            self.i = end + 1;
            self.last = self.i;
            return true;
        }
        false
    }

    fn render_inline_expr(&mut self, start: usize, end: usize, left: &str, right: &str) {
        let expr = &self.line[start..end];
        match render_texicode(expr) {
            Ok(rendered) => append_inline_render(&mut self.out, &rendered),
            Err(err) => self.render_inline_error(expr, left, right, &err),
        }
    }

    fn render_inline_error(&mut self, expr: &str, left: &str, right: &str, err: &str) {
        self.out.push('`');
        self.out.push_str(left);
        self.out.push_str(expr);
        self.out.push_str(right);
        self.out.push('`');
        if !is_known_txc_error(err) {
            self.out.push('\n');
            self.out.push_str("[txc] 渲染失败：");
            self.out.push_str(err);
        }
    }
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
