#[cfg(test)]
mod tests {
    use crate::ui::scroll::{max_scroll, max_scroll_u16, SCROLL_STEP_I32, SCROLL_STEP_U16};

    #[test]
    fn computes_max_scroll() {
        assert_eq!(max_scroll(10, 3), 7);
        assert_eq!(max_scroll(3, 10), 0);
    }

    #[test]
    fn computes_max_scroll_u16() {
        assert_eq!(max_scroll_u16(10, 3), 7);
        assert_eq!(max_scroll_u16(2, 10), 0);
        assert!(max_scroll_u16(100_000, 1) <= u16::MAX);
    }

    #[test]
    fn scroll_steps_are_positive() {
        assert!(SCROLL_STEP_I32 > 0);
        assert!(SCROLL_STEP_U16 > 0);
    }
}
