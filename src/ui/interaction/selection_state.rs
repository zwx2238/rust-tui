use crate::ui::scroll::max_scroll;

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct SelectionState {
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
}

impl SelectionState {
    pub(crate) fn select(&mut self, idx: usize) {
        self.selected = idx;
    }

    pub(crate) fn clamp_with_viewport(&mut self, len: usize, viewport_rows: usize) {
        if len == 0 {
            self.selected = 0;
            self.scroll = 0;
            return;
        }
        if self.selected >= len {
            self.selected = len - 1;
        }
        if viewport_rows == 0 {
            self.scroll = 0;
            return;
        }
        let max_scroll = max_scroll(len, viewport_rows);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
        self.ensure_visible(viewport_rows);
    }

    pub(crate) fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
    }

    pub(crate) fn move_down(&mut self) {
        self.selected = self.selected.saturating_add(1);
    }

    pub(crate) fn page_up(&mut self, step: usize) {
        self.scroll = self.scroll.saturating_sub(step);
        if self.selected < self.scroll {
            self.selected = self.scroll;
        }
    }

    pub(crate) fn page_down(&mut self, step: usize) {
        self.scroll = self.scroll.saturating_add(step);
        if self.selected < self.scroll {
            self.selected = self.scroll;
        }
    }

    pub(crate) fn ensure_visible(&mut self, viewport_rows: usize) {
        if viewport_rows == 0 {
            return;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + viewport_rows {
            self.scroll = self
                .selected
                .saturating_sub(viewport_rows.saturating_sub(1));
        }
    }

    pub(crate) fn scroll_offset_by(&mut self, delta: i32, max_scroll: usize) {
        self.scroll = offset_scroll(self.scroll, delta);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
    }

    pub(crate) fn scroll_by(&mut self, delta: i32, max_scroll: usize, viewport_rows: usize) {
        self.scroll_offset_by(delta, max_scroll);
        self.ensure_visible(viewport_rows);
    }
}

fn offset_scroll(scroll: usize, delta: i32) -> usize {
    if delta.is_negative() {
        let step = delta.unsigned_abs() as usize;
        scroll.saturating_sub(step)
    } else {
        let step = delta as usize;
        scroll.saturating_add(step)
    }
}
