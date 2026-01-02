mod cache;
mod layout;
mod markdown;
mod theme;
mod util;

pub use cache::{
    RenderCacheEntry, build_cache_entry, insert_empty_cache_entry, messages_to_plain_lines,
    messages_to_viewport_text_cached, messages_to_viewport_text_cached_with_layout,
    set_cache_entry,
};
pub use layout::{MessageLayout, label_line_layout, label_line_with_button};
pub use markdown::render_markdown_lines;

pub fn count_message_lines(msg: &crate::types::Message, width: usize, streaming: bool) -> usize {
    cache::count_message_lines(msg, width, streaming)
}

pub fn label_for_role(role: &str, suffix: Option<&str>) -> Option<String> {
    util::label_for_role(role, suffix)
}
pub use theme::{RenderTheme, theme_from_config};
