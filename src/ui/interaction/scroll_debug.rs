use std::sync::OnceLock;

pub struct ScrollDebug {
    pub total_lines: usize,
    pub scroll: u16,
    pub content_height: u16,
    pub max_scroll: u16,
    pub viewport_len: usize,
    pub scroll_area_height: u16,
}

static ENABLED: OnceLock<bool> = OnceLock::new();

pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var("DEBUG_SCROLL").is_ok())
}

pub fn format(info: &ScrollDebug) -> String {
    format!(
        "scroll={} max={} total={} view={} vp={} sbh={}",
        info.scroll,
        info.max_scroll,
        info.total_lines,
        info.content_height,
        info.viewport_len,
        info.scroll_area_height
    )
}
