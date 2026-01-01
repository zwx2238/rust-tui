pub(crate) fn max_scroll(total_lines: usize, viewport_rows: usize) -> usize {
    total_lines.saturating_sub(viewport_rows)
}

pub(crate) fn max_scroll_u16(total_lines: usize, viewport_rows: u16) -> u16 {
    max_scroll(total_lines, viewport_rows as usize)
        .min(u16::MAX as usize) as u16
}

pub(crate) const SCROLL_STEP_I32: i32 = 3;
pub(crate) const SCROLL_STEP_U16: u16 = 3;
