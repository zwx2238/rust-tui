mod cache;
mod markdown;
mod theme;
mod util;

pub use cache::{
    build_cache_entry, insert_empty_cache_entry, messages_to_plain_lines,
    messages_to_viewport_text_cached, set_cache_entry, RenderCacheEntry,
};

pub fn count_message_lines(
    msg: &crate::types::Message,
    width: usize,
    streaming: bool,
) -> usize {
    cache::count_message_lines(msg, width, streaming)
}

pub fn label_for_role(role: &str, suffix: Option<&str>) -> Option<String> {
    util::label_for_role(role, suffix)
}
pub use theme::{theme_from_config, RenderTheme};
