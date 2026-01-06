#[cfg(test)]
mod tests {
    use crate::ui::scroll::{SCROLL_STEP_I32, SCROLL_STEP_U16, max_scroll, max_scroll_u16};

    #[test]
    fn computes_max_scroll() {
        assert_eq!(max_scroll(10, 3), 7);
        assert_eq!(max_scroll(3, 10), 0);
    }

    #[test]
    fn computes_max_scroll_u16() {
        assert_eq!(max_scroll_u16(10, 3), 7);
        assert_eq!(max_scroll_u16(2, 10), 0);
        // Test that large values are clamped to u16::MAX
        let result = max_scroll_u16(100_000, 1);
        assert_eq!(result, u16::MAX);
    }

    #[test]
    fn scroll_steps_are_positive() {
        const {
            assert!(SCROLL_STEP_I32 > 0);
            assert!(SCROLL_STEP_U16 > 0);
        }
    }
}
