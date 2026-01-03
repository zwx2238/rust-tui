#[cfg(test)]
mod tests {
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_view::{ViewAction, ViewState, apply_view_action, handle_view_key, handle_view_mouse};
    use crate::ui::overlay::OverlayKind;
    use crate::ui::jump::JumpRow;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEventKind, MouseButton};

    fn setup_view_action_state() -> (Vec<TabState>, usize, Vec<String>, usize, Vec<JumpRow>) {
        let tabs = vec![
            TabState::new("a".into(), "cat1".into(), "", false, "m", "p"),
            TabState::new("b".into(), "cat2".into(), "", false, "m", "p"),
        ];
        let categories = vec!["cat1".to_string(), "cat2".to_string()];
        let jump_rows = vec![JumpRow {
            index: 1,
            role: crate::types::ROLE_USER.to_string(),
            preview: "hi".to_string(),
            scroll: 3,
        }];
        (tabs, 0, categories, 0, jump_rows)
    }

    #[test]
    fn handle_view_key_toggles_overlays() {
        let mut view = ViewState::new();
        let action = handle_view_key(
            &mut view,
            KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
            3,
            0,
            0,
        );
        assert!(matches!(action, ViewAction::None));
        assert!(view.overlay.is(OverlayKind::Summary));
        let _ = handle_view_key(
            &mut view,
            KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
            3,
            0,
            0,
        );
        assert!(view.overlay.is_chat());
    }

    #[test]
    fn handle_view_key_opens_jump_and_help() {
        let mut view = ViewState::new();
        let action = handle_view_key(
            &mut view,
            KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
            3,
            2,
            0,
        );
        assert!(matches!(action, ViewAction::None));
        assert!(view.overlay.is(OverlayKind::Jump));
        let action = handle_view_key(
            &mut view,
            KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE),
            3,
            0,
            0,
        );
        assert!(matches!(action, ViewAction::None));
        assert!(view.overlay.is(OverlayKind::Help));
    }

    #[test]
    fn handle_view_key_toggles_prompt_and_model() {
        let mut view = ViewState::new();
        let _ = handle_view_key(
            &mut view,
            KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE),
            1,
            0,
            0,
        );
        assert!(view.overlay.is(OverlayKind::Prompt));
        let _ = handle_view_key(
            &mut view,
            KeyEvent::new(KeyCode::F(4), KeyModifiers::NONE),
            1,
            0,
            0,
        );
        assert!(view.overlay.is(OverlayKind::Model));
    }

    #[test]
    fn handle_view_key_closes_help_when_open() {
        let mut view = ViewState::new();
        view.overlay.open(OverlayKind::Help);
        let _ = handle_view_key(
            &mut view,
            KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE),
            1,
            0,
            0,
        );
        assert!(view.overlay.is_chat());
    }

    #[test]
    fn handle_view_key_cycles_model() {
        let mut view = ViewState::new();
        let action = handle_view_key(
            &mut view,
            KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE),
            3,
            0,
            0,
        );
        assert!(matches!(action, ViewAction::CycleModel));
    }

    #[test]
    fn handle_view_mouse_switches() {
        let mut view = ViewState::new();
        view.overlay.open(OverlayKind::Summary);
        let action = handle_view_mouse(
            &mut view,
            Some(0),
            2,
            0,
            MouseEventKind::Down(MouseButton::Left),
        );
        assert!(matches!(action, ViewAction::SwitchTab(0)));
    }

    #[test]
    fn handle_view_mouse_selects_jump_and_prompt() {
        let mut view = ViewState::new();
        view.overlay.open(OverlayKind::Jump);
        let action = handle_view_mouse(
            &mut view,
            Some(1),
            0,
            3,
            MouseEventKind::Down(MouseButton::Left),
        );
        assert!(matches!(action, ViewAction::JumpTo(1)));
        view.overlay.open(OverlayKind::Prompt);
        let action = handle_view_mouse(
            &mut view,
            Some(2),
            0,
            0,
            MouseEventKind::Down(MouseButton::Left),
        );
        assert!(matches!(action, ViewAction::SelectPrompt(2)));
    }

    #[test]
    fn handle_view_mouse_selects_model_and_help() {
        let mut view = ViewState::new();
        view.overlay.open(OverlayKind::Model);
        let action = handle_view_mouse(
            &mut view,
            Some(1),
            0,
            0,
            MouseEventKind::Down(MouseButton::Left),
        );
        assert!(matches!(action, ViewAction::SelectModel(1)));
        view.overlay.open(OverlayKind::Help);
        let action = handle_view_mouse(
            &mut view,
            Some(0),
            0,
            0,
            MouseEventKind::Down(MouseButton::Left),
        );
        assert!(matches!(action, ViewAction::None));
        assert!(view.overlay.is_chat());
    }

    #[test]
    fn apply_view_action_switch_and_jump() {
        let (mut tabs, mut active_tab, mut categories, mut active_category, jump_rows) =
            setup_view_action_state();
        let switched = apply_view_action(
            ViewAction::SwitchTab(1),
            &jump_rows,
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
        );
        assert!(switched);
        assert_eq!(active_tab, 1);
        let jumped = apply_view_action(
            ViewAction::JumpTo(0),
            &jump_rows,
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
        );
        assert!(jumped);
        assert_eq!(tabs[active_tab].app.scroll, 3);
    }
}
