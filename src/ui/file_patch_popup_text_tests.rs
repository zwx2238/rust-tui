#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::file_patch_popup_text::{build_patch_text, patch_max_scroll};
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

    #[test]
    fn build_patch_text_and_scroll() {
        let (text, total) = build_patch_text("+ added", 40, 5, 0, &theme());
        assert!(total >= 1);
        assert!(!text.lines.is_empty());
        let _ = patch_max_scroll("+ added", 40, 3, &theme());
    }
}
