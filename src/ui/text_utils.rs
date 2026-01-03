use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn collapse_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if text.width() <= max_width {
        return text.to_string();
    }
    let ellipsis = "...";
    let mut out = String::new();
    let mut width = 0usize;
    let limit = max_width.saturating_sub(ellipsis.width());
    for ch in text.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(1);
        if width.saturating_add(w) > limit {
            break;
        }
        out.push(ch);
        width = width.saturating_add(w);
    }
    out.push_str(ellipsis);
    out
}

#[cfg(test)]
mod tests {
    use super::{collapse_text, truncate_to_width};

    #[test]
    fn collapse_text_merges_whitespace() {
        let input = "a  b\nc\t d";
        assert_eq!(collapse_text(input), "a b c d");
    }

    #[test]
    fn truncate_to_width_keeps_short() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
    }

    #[test]
    fn truncate_to_width_adds_ellipsis() {
        let out = truncate_to_width("hello world", 6);
        assert_eq!(out, "he...");
    }
}
