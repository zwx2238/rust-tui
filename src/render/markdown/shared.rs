use crate::render::markdown::table::TableBuild;
use pulldown_cmark::{Options, Parser as MdParser};

#[derive(Clone, Copy)]
pub(crate) struct ListState {
    pub(crate) ordered: bool,
    pub(crate) index: u64,
}

pub(crate) struct ItemContext {
    pub(crate) buf: String,
    pub(crate) depth: usize,
    pub(crate) ordered: bool,
    pub(crate) index: u64,
}

pub(crate) fn append_text(
    buf: &mut String,
    item_stack: &mut [ItemContext],
    table: &mut TableBuild,
    text: &str,
) {
    if table.in_cell {
        table.push_text(text);
    } else if let Some(item) = item_stack.last_mut() {
        item.buf.push_str(text);
    } else {
        buf.push_str(text);
    }
}

pub(crate) fn list_prefix(ordered: bool, index: u64) -> String {
    if ordered {
        format!("{index}. ")
    } else {
        "• ".to_string()
    }
}

pub(crate) fn list_indent(depth: usize) -> String {
    "  ".repeat(depth.saturating_sub(1))
}

pub(crate) fn markdown_parser<'a>(text: &'a str) -> MdParser<'a> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    MdParser::new_ext(text, options)
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn list_prefix_and_indent() {
        assert_eq!(list_prefix(true, 2), "2. ");
        assert_eq!(list_prefix(false, 1), "• ");
        assert_eq!(list_indent(1), "");
        assert_eq!(list_indent(3), "    ");
    }

    #[test]
    fn append_text_targets_table_cell() {
        let mut buf = String::new();
        let mut items = Vec::new();
        let mut table = TableBuild::default();
        table.start(false);
        table.start_row();
        table.start_cell();
        append_text(&mut buf, &mut items, &mut table, "cell");
        table.end_cell();
        table.end_row();
        let out = table.finish_render(20, &theme());
        assert!(out[0].to_string().contains("cell"));
    }

    #[test]
    fn append_text_targets_item_buffer() {
        let mut buf = String::new();
        let mut items = vec![ItemContext {
            buf: String::new(),
            depth: 1,
            ordered: false,
            index: 1,
        }];
        let mut table = TableBuild::default();
        append_text(&mut buf, &mut items, &mut table, "text");
        assert_eq!(items[0].buf, "text");
        assert!(buf.is_empty());
    }
}
