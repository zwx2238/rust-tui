#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum OverlayKind {
    Summary,
    Jump,
    Model,
    Prompt,
}

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct OverlayState {
    pub(crate) active: Option<OverlayKind>,
}

impl OverlayState {
    pub(crate) fn is_chat(&self) -> bool {
        self.active.is_none()
    }

    pub(crate) fn is(&self, kind: OverlayKind) -> bool {
        self.active == Some(kind)
    }

    pub(crate) fn uses_simple_layout(&self) -> bool {
        matches!(self.active, Some(OverlayKind::Summary | OverlayKind::Jump))
    }

    pub(crate) fn toggle(&mut self, kind: OverlayKind) {
        self.active = if self.active == Some(kind) {
            None
        } else {
            Some(kind)
        };
    }

    pub(crate) fn open(&mut self, kind: OverlayKind) {
        self.active = Some(kind);
    }

    pub(crate) fn close(&mut self) {
        self.active = None;
    }
}
