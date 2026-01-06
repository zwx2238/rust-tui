mod categories;
mod header_footer;
pub mod layout;
mod messages;
pub(crate) mod style;
mod tabs;

pub(crate) use categories::draw_categories;
pub(crate) use header_footer::{draw_footer, draw_header};
pub use layout::{
    inner_area, inner_height, inner_width, input_inner_area, layout_chunks, scrollbar_area,
};
pub(crate) use messages::{MessagesDrawParams, draw_messages};

pub(crate) use tabs::draw_tabs;

// 旧的「在 draw 里直接驱动整屏重绘」API 已废弃。
