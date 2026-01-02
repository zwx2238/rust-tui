pub(super) fn is_known_txc_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("cmd_bgin")
        || lower.contains("expected ['cls_dlim']")
        || lower.contains("expected ['cls_dlim', 'cmd_lbrk']")
        || lower.contains("expected ['cls_dlim', 'cmd_lbrk', 'cls_line']")
}
