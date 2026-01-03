#[cfg(test)]
mod tests {
    use crate::ui::runtime_view::{ViewAction, ViewState};
    use crate::ui::runtime_view_handlers::{
        handle_help_key, handle_jump_key, handle_model_key, handle_prompt_key, handle_summary_key,
    };
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn summary_enter_switches_tab() {
        let mut view = ViewState::new();
        view.summary_order = vec![2];
        view.summary.selected = 0;
        let action = handle_summary_key(
            &mut view,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            3,
        );
        assert!(matches!(action, ViewAction::SwitchTab(2)));
    }

    #[test]
    fn summary_toggle_sort_resets_state() {
        let mut view = ViewState::new();
        view.summary.selected = 3;
        view.summary.scroll = 2;
        let action = handle_summary_key(
            &mut view,
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
            2,
        );
        assert!(matches!(action, ViewAction::None));
        assert_eq!(view.summary.selected, 0);
        assert_eq!(view.summary.scroll, 0);
    }

    #[test]
    fn summary_enter_out_of_range_returns_none() {
        let mut view = ViewState::new();
        view.summary.selected = 5;
        let action = handle_summary_key(
            &mut view,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            1,
        );
        assert!(matches!(action, ViewAction::None));
    }

    #[test]
    fn jump_key_fork_message() {
        let mut view = ViewState::new();
        view.jump.selected = 0;
        let action = handle_jump_key(
            &mut view,
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            1,
        );
        assert!(matches!(action, ViewAction::ForkMessage(0)));
    }

    #[test]
    fn jump_keys_move_selection() {
        let mut view = ViewState::new();
        let _ = handle_jump_key(
            &mut view,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            3,
        );
        let _ = handle_jump_key(
            &mut view,
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            10,
        );
        assert!(view.jump.selected > 0 || view.jump.scroll > 0);
    }

    #[test]
    fn jump_esc_closes_overlay() {
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::Jump);
        let action = handle_jump_key(
            &mut view,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            1,
        );
        assert!(matches!(action, ViewAction::None));
        assert!(view.overlay.is_chat());
    }

    #[test]
    fn model_key_selects_model() {
        let mut view = ViewState::new();
        view.model.selected = 1;
        let action = handle_model_key(
            &mut view,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        assert!(matches!(action, ViewAction::SelectModel(1)));
    }

    #[test]
    fn model_key_moves_selection() {
        let mut view = ViewState::new();
        let _ = handle_model_key(
            &mut view,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        assert!(view.model.selected > 0 || view.model.scroll > 0);
    }

    #[test]
    fn prompt_key_selects_prompt() {
        let mut view = ViewState::new();
        view.prompt.selected = 2;
        let action = handle_prompt_key(
            &mut view,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        assert!(matches!(action, ViewAction::SelectPrompt(2)));
    }

    #[test]
    fn prompt_key_moves_selection() {
        let mut view = ViewState::new();
        let _ = handle_prompt_key(
            &mut view,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        assert!(view.prompt.selected > 0 || view.prompt.scroll > 0);
    }

    #[test]
    fn help_key_page_down_moves_selection() {
        let mut view = ViewState::new();
        let action = handle_help_key(
            &mut view,
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
        );
        assert!(matches!(action, ViewAction::None));
        assert!(view.help.selected > 0 || view.help.scroll > 0);
    }

    #[test]
    fn help_key_esc_closes_overlay() {
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::Help);
        let action = handle_help_key(
            &mut view,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        );
        assert!(matches!(action, ViewAction::None));
        assert!(view.overlay.is_chat());
    }

    #[test]
    fn summary_key_down_moves_selection() {
        let mut view = ViewState::new();
        let _ = handle_summary_key(
            &mut view,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            2,
        );
        assert_eq!(view.summary.selected, 1);
    }

    #[test]
    fn help_key_up_runs() {
        let mut view = ViewState::new();
        let _ = handle_help_key(
            &mut view,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
        );
        assert_eq!(view.help.selected, 0);
    }

    #[test]
    fn summary_esc_closes_overlay() {
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::Summary);
        let action = handle_summary_key(
            &mut view,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            1,
        );
        assert!(matches!(action, ViewAction::None));
        assert!(view.overlay.is_chat());
    }

    #[test]
    fn jump_enter_returns_jump() {
        let mut view = ViewState::new();
        view.jump.selected = 0;
        let action = handle_jump_key(
            &mut view,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            1,
        );
        assert!(matches!(action, ViewAction::JumpTo(0)));
    }
}
