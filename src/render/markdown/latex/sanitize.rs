pub(crate) fn sanitize_tex(expr: &str) -> String {
    if !tex_filter_enabled() {
        return expr.to_string();
    }
    let lines = collect_candidate_lines(expr);
    filter_tex_lines(lines).join("\n")
}

fn collect_candidate_lines(expr: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = expr.lines().collect();
    if let Some(start) = lines.iter().position(|l| l.contains("\\begin{cases}"))
        && let Some(end) = lines.iter().rposition(|l| l.contains("\\end{cases}"))
        && end >= start
    {
        lines = lines[start..=end].to_vec();
    }
    lines
}

fn filter_tex_lines(lines: Vec<&str>) -> Vec<String> {
    let mut out = Vec::new();
    for raw in lines {
        if let Some(line) = sanitize_tex_line(raw) {
            out.push(line);
        }
    }
    out
}

fn sanitize_tex_line(raw: &str) -> Option<String> {
    let mut line = raw.replace('\r', "");
    let trimmed = line.trim();
    if trimmed == "]" || trimmed == "[" || trimmed == "\\]" || trimmed == "\\[" {
        return None;
    }
    line = strip_cjk_punct(&line);
    if contains_cjk_letters(&line) || !is_mathish_line(&line) {
        return None;
    }
    if line.ends_with(" \\") && !line.ends_with(" \\\\") {
        line.push('\\');
    }
    Some(line)
}

fn tex_filter_enabled() -> bool {
    match std::env::var("DEEPCHAT_TEX_FILTER") {
        Ok(value) => {
            let v = value.trim().to_ascii_lowercase();
            !(v.is_empty() || v == "0" || v == "false" || v == "off" || v == "no")
        }
        Err(_) => false,
    }
}

fn is_mathish_line(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }
    let has_digit = trimmed.chars().any(|ch| ch.is_ascii_digit());
    let has_backslash = trimmed.contains('\\');
    let has_operator = trimmed.chars().any(|ch| {
        matches!(
            ch,
            '+' | '-' | '=' | '*' | '/' | '^' | '_' | '{' | '}' | '(' | ')' | '[' | ']' | '|'
        )
    });
    has_digit || has_backslash || has_operator
}

fn strip_cjk_punct(input: &str) -> String {
    input
        .chars()
        .filter(|ch| {
            !matches!(
                ch,
                '、' | '，' | '。' | '；' | '：' | '（' | '）' | '【' | '】' | '「' | '」'
            )
        })
        .collect()
}

fn contains_cjk_letters(input: &str) -> bool {
    input.chars().any(|ch| {
        matches!(ch as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF | 0x2A700..=0x2B73F | 0x2B740..=0x2B81F | 0x2B820..=0x2CEAF)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{env_lock, restore_env, set_env};

    #[test]
    fn sanitize_no_filter_returns_original() {
        let input = "a + b = c";
        let guard = env_lock().lock().unwrap();
        let prev = set_env("DEEPCHAT_TEX_FILTER", "0");
        let out = sanitize_tex(input);
        restore_env("DEEPCHAT_TEX_FILTER", prev);
        drop(guard);
        assert_eq!(out, input);
    }

    #[test]
    fn sanitize_filters_non_math_lines() {
        let input = "这是中文行\nx^2 + 1\n\\begin{cases}\na+b=c\n\\end{cases}";
        let guard = env_lock().lock().unwrap();
        let prev = set_env("DEEPCHAT_TEX_FILTER", "1");
        let out = sanitize_tex(input);
        restore_env("DEEPCHAT_TEX_FILTER", prev);
        drop(guard);
        assert!(out.contains("a+b=c"));
        assert!(!out.contains("这是中文行"));
    }
}
