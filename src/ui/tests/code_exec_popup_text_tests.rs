#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::code_exec_popup_text::{
        build_code_text, build_stderr_text, build_stdout_text, code_max_scroll, stderr_max_scroll,
        stdout_max_scroll,
    };
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
    fn builds_code_text_and_scroll() {
        let (text, total) = build_code_text("print(1)", 40, 5, 0, &theme());
        assert!(total >= 1);
        assert!(!text.lines.is_empty());
        let _ = code_max_scroll("print(1)", 40, 3, &theme());
    }

    #[test]
    fn builds_stdout_and_stderr_text() {
        let (stdout, total_out) = build_stdout_text(Some("ok"), 40, 4, 0, &theme());
        assert!(total_out >= 1);
        assert!(!stdout.lines.is_empty());
        let (stderr, total_err) = build_stderr_text(Some("err"), 40, 4, 0, &theme());
        assert!(total_err >= 1);
        assert!(!stderr.lines.is_empty());
        let _ = stdout_max_scroll("ok", 40, 3, &theme());
        let _ = stderr_max_scroll("err", 40, 3, &theme());
    }
}
