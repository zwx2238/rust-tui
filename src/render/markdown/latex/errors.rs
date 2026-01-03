pub(super) fn is_known_txc_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("cmd_bgin")
        || lower.contains("expected ['cls_dlim']")
        || lower.contains("expected ['cls_dlim', 'cmd_lbrk']")
        || lower.contains("expected ['cls_dlim', 'cmd_lbrk', 'cls_line']")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_errors() {
        assert!(is_known_txc_error("Expected ['cls_dlim']"));
        assert!(is_known_txc_error("cmd_bgin"));
        assert!(!is_known_txc_error("other error"));
    }
}
