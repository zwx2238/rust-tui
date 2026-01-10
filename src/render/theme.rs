use crate::config::Config;
use ratatui::style::Color;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct RenderTheme {
    pub bg: Color,
    pub fg: Option<Color>,
    pub code_bg: Color,
    pub code_theme: &'static str,
    pub heading_fg: Option<Color>,
}

pub fn theme_from_config(cfg: &Config) -> Result<RenderTheme, Box<dyn std::error::Error>> {
    let name = cfg.theme.trim().to_ascii_lowercase();
    match name.as_str() {
        "light" => Ok(RenderTheme {
            bg: Color::White,
            fg: Some(Color::Black),
            code_bg: Color::White,
            code_theme: "base16-ocean.light",
            heading_fg: Some(Color::Blue),
        }),
        "dark" => Ok(RenderTheme {
            bg: Color::Black,
            fg: None,
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }),
        _ => Err(format!("配置文件错误：theme 不支持 {}", cfg.theme).into()),
    }
}

pub(crate) fn theme_cache_key(theme: &RenderTheme) -> u64 {
    let mut hasher = DefaultHasher::new();
    theme.bg.hash(&mut hasher);
    theme.fg.hash(&mut hasher);
    theme.code_bg.hash(&mut hasher);
    theme.code_theme.hash(&mut hasher);
    theme.heading_fg.hash(&mut hasher);
    hasher.finish()
}
