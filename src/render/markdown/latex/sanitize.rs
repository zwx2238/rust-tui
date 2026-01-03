pub(crate) fn sanitize_tex(expr: &str) -> String {
    if !tex_filter_enabled() {
        return expr.to_string();
    }
    let mut out = Vec::new();
    let mut lines: Vec<&str> = expr.lines().collect();
    if let Some(start) = lines.iter().position(|l| l.contains("\\begin{cases}")) {
        if let Some(end) = lines.iter().rposition(|l| l.contains("\\end{cases}")) {
            if end >= start {
                lines = lines[start..=end].to_vec();
            }
        }
    }
    for raw in lines {
        let mut line = raw.replace('\r', "");
        let trimmed = line.trim();
        if trimmed == "]" || trimmed == "[" || trimmed == "\\]" || trimmed == "\\[" {
            continue;
        }
        line = strip_cjk_punct(&line);
        if contains_cjk_letters(&line) {
            continue;
        }
        if !is_mathish_line(&line) {
            continue;
        }
        if line.ends_with(" \\") && !line.ends_with(" \\\\") {
            line.push('\\');
        }
        out.push(line);
    }
    out.join("\n")
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
