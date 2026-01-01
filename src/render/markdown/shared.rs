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
    item_stack: &mut Vec<ItemContext>,
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
