#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::code_exec_popup::draw_code_exec_popup;
    use crate::ui::state::{CodeExecHover, CodeExecLive, CodeExecReasonTarget, PendingCodeExec};
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use ratatui::Terminal;
    use std::time::Instant;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    fn pending() -> PendingCodeExec {
        PendingCodeExec {
            call_id: "c1".to_string(),
            language: "python".to_string(),
            code: "print('hi')".to_string(),
            exec_code: None,
            requested_at: Instant::now(),
            stop_reason: None,
        }
    }

    #[test]
    fn draw_popup_with_reason() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut input = tui_textarea::TextArea::default();
        terminal
            .draw(|f| {
                draw_code_exec_popup(
                    f,
                    Rect::new(0, 0, 120, 40),
                    &pending(),
                    0,
                    0,
                    0,
                    Some(CodeExecHover::Approve),
                    Some(CodeExecReasonTarget::Deny),
                    &mut input,
                    None,
                    &theme(),
                );
            })
            .unwrap();
    }

    #[test]
    fn draw_popup_running_and_finished() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut input = tui_textarea::TextArea::default();
        let live = CodeExecLive {
            started_at: Instant::now(),
            finished_at: Some(Instant::now()),
            stdout: "ok".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            done: true,
        };
        terminal
            .draw(|f| {
                draw_code_exec_popup(
                    f,
                    Rect::new(0, 0, 120, 40),
                    &pending(),
                    0,
                    0,
                    0,
                    Some(CodeExecHover::Exit),
                    None,
                    &mut input,
                    Some(&live),
                    &theme(),
                );
            })
            .unwrap();
    }
}
