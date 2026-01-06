use ratatui::layout::Size;

#[derive(Copy, Clone, Debug)]
pub(crate) struct BoxConstraints {
    pub(crate) min: Size,
    pub(crate) max: Size,
}

impl BoxConstraints {
    pub(crate) fn tight(max: Size) -> Self {
        Self { min: max, max }
    }

    pub(crate) fn loose(max: Size) -> Self {
        Self {
            min: Size {
                width: 0,
                height: 0,
            },
            max,
        }
    }

    pub(crate) fn constrain(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min.width, self.max.width),
            height: size.height.clamp(self.min.height, self.max.height),
        }
    }
}
