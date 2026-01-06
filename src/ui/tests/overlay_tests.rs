#[cfg(test)]
mod tests {
    use crate::ui::overlay::{OverlayKind, OverlayState};

    #[test]
    fn overlay_state_transitions() {
        let mut state = OverlayState::default();
        assert!(state.is_chat());
        state.open(OverlayKind::Summary);
        assert!(state.is(OverlayKind::Summary));
        state.toggle(OverlayKind::Summary);
        assert!(state.is_chat());
        state.toggle(OverlayKind::Prompt);
        assert!(state.is(OverlayKind::Prompt));
        state.close();
        assert!(state.is_chat());
    }

    #[test]
    fn overlay_simple_layout() {
        let mut state = OverlayState::default();
        state.open(OverlayKind::Help);
        assert!(state.uses_simple_layout());
        state.open(OverlayKind::Model);
        assert!(!state.uses_simple_layout());
    }
}
