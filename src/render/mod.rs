//! 渲染模块
//!
//! 负责将消息和内容渲染为终端可显示的格式。
//!
//! ## 子模块
//!
//! - `cache` - 渲染缓存管理
//! - `layout` - 布局管理
//! - `markdown` - Markdown 渲染
//! - `theme` - 主题管理
//! - `util` - 工具函数

mod cache;
mod layout;
mod markdown;
mod theme;
mod util;

pub use cache::{
    RenderCacheEntry, SingleMessageRenderParams, build_cache_entry, insert_empty_cache_entry,
    message_to_plain_lines, message_to_viewport_text_cached,
    message_to_viewport_text_cached_with_layout, set_cache_entry,
};
pub use layout::MessageLayout;
pub use markdown::render_markdown_lines;

pub fn label_for_role(role: &str, suffix: Option<&str>) -> Option<String> {
    util::label_for_role(role, suffix)
}
pub use theme::{RenderTheme, theme_from_config};
