#[cfg(test)]
mod tests {
    use crate::render::markdown::table::TableBuild;
    use crate::render::theme::RenderTheme;
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
    fn renders_streaming_table() {
        let mut table = TableBuild::default();
        table.start(true);
        table.start_head();
        table.start_row();
        table.start_cell();
        table.push_text("A");
        table.end_cell();
        table.start_cell();
        table.push_text("B");
        table.end_cell();
        table.end_row();
        table.end_head();
        table.start_row();
        table.start_cell();
        table.push_text("1");
        table.end_cell();
        table.start_cell();
        table.push_text("2");
        table.end_cell();
        table.end_row();
        let out = table.finish_render(40, &theme());
        assert_eq!(out.len(), 3);
        assert!(out[0].to_string().contains("| A |"));
    }

    #[test]
    fn renders_aligned_table() {
        let mut table = TableBuild::default();
        table.start(false);
        table.start_head();
        table.start_row();
        table.start_cell();
        table.push_text("Header");
        table.end_cell();
        table.end_row();
        table.end_head();
        table.start_row();
        table.start_cell();
        table.push_text("Cell");
        table.end_cell();
        table.end_row();
        let out = table.finish_render(20, &theme());
        assert_eq!(out.len(), 3);
        assert!(out[1].to_string().contains("-"));
    }

    #[test]
    fn counts_table_lines() {
        let mut table = TableBuild::default();
        table.start(false);
        table.start_row();
        table.start_cell();
        table.push_text("A");
        table.end_cell();
        table.end_row();
        let count = table.finish_count(10);
        assert_eq!(count, 1);
    }
}
