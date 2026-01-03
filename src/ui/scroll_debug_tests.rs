#[cfg(test)]
mod tests {
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::scroll_debug::{ScrollDebug, enabled, format};

    #[test]
    fn debug_enabled_checks_env() {
        let _guard = env_lock().lock().unwrap();
        let prev = set_env("DEBUG_SCROLL", "1");
        let _ = enabled();
        restore_env("DEBUG_SCROLL", prev);
    }

    #[test]
    fn formats_scroll_debug() {
        let info = ScrollDebug {
            total_lines: 10,
            scroll: 2,
            content_height: 5,
            max_scroll: 7,
            viewport_len: 4,
            scroll_area_height: 3,
        };
        let text = format(&info);
        assert!(text.contains("scroll=2"));
        assert!(text.contains("max=7"));
    }
}
