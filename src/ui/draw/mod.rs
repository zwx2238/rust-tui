pub mod layout;
pub(crate) mod style;

pub use layout::{
    inner_area, inner_height, inner_width, input_inner_area, scrollbar_area,
};

// 旧的「在 draw 里直接驱动整屏重绘」API 已废弃。
