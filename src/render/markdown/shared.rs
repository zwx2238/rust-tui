use crate::render::markdown::table::TableBuild;

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
