#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::file_patch_popup::draw_file_patch_popup;
    use crate::ui::state::{FilePatchHover, PendingFilePatch};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    fn pending() -> PendingFilePatch {
        PendingFilePatch {
            call_id: "p1".to_string(),
            path: Some("file.txt".to_string()),
            diff: "+ line".to_string(),
            preview: "+ line".to_string(),
        }
    }

    #[test]
    fn draw_file_patch_popup_smoke() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                draw_file_patch_popup(
                    f,
                    Rect::new(0, 0, 120, 40),
                    &pending(),
                    0,
                    Some(FilePatchHover::Apply),
                    &theme(),
                );
            })
            .unwrap();
    }
}
