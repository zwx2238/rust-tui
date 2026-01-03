#[cfg(test)]
mod tests {
    use crate::ui::command_suggestions::{
        apply_command_suggestion, clear_command_suggestions, command_suggestions_active,
        command_suggestions_area, command_suggestions_row_at, handle_command_suggestion_click,
        refresh_command_suggestions,
    };
    use crate::ui::state::{App, Focus};
    use ratatui::layout::Rect;

    fn app_with_input(line: &str) -> App {
        let mut app = App::new("system", "model", "prompt");
        app.input = tui_textarea::TextArea::from(vec![line.to_string()]);
        app.focus = Focus::Input;
        app
    }

    #[test]
    fn refresh_and_clear_suggestions() {
        let mut app = app_with_input("/he");
        refresh_command_suggestions(&mut app);
        assert!(command_suggestions_active(&app));
        clear_command_suggestions(&mut app);
        assert!(!command_suggestions_active(&app));
    }

    #[test]
    fn apply_command_suggestion_updates_input() {
        let mut app = app_with_input("/he");
        refresh_command_suggestions(&mut app);
        assert!(apply_command_suggestion(&mut app));
        let line = app.input.lines().get(0).cloned().unwrap_or_default();
        assert!(line.starts_with("/help"));
    }

    #[test]
    fn suggestion_area_and_row_at() {
        let msg_area = Rect::new(0, 0, 60, 20);
        let input_area = Rect::new(0, 20, 60, 3);
        let area = command_suggestions_area(msg_area, input_area, 3);
        assert!(area.height >= 3);
        let row = command_suggestions_row_at(msg_area, input_area, 3, 0, area.x + 1, area.y + 2);
        assert_eq!(row, Some(0));
    }

    #[test]
    fn click_applies_suggestion() {
        let mut app = app_with_input("/he");
        refresh_command_suggestions(&mut app);
        let msg_area = Rect::new(0, 0, 60, 20);
        let input_area = Rect::new(0, 20, 60, 3);
        let area = command_suggestions_area(msg_area, input_area, app.command_suggestions.len());
        let clicked = handle_command_suggestion_click(
            &mut app,
            msg_area,
            input_area,
            area.x + 1,
            area.y + 2,
        );
        assert!(clicked);
        let line = app.input.lines().get(0).cloned().unwrap_or_default();
        assert!(line.starts_with("/help"));
    }
}
