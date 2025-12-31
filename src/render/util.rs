use crate::render::theme::RenderTheme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(crate) fn ranges_overlap(start: usize, end: usize, a: usize, b: usize) -> bool {
    a < end && b > start
}

pub(crate) fn suffix_for_index<'a>(suffixes: &'a [(usize, String)], idx: usize) -> Option<&'a str> {
    suffixes
        .iter()
        .find(|(i, _)| *i == idx)
        .map(|(_, s)| s.as_str())
}

pub(crate) fn label_for_role(role: &str, suffix: Option<&str>) -> Option<String> {
    match role {
        "user" => Some("👤".to_string()),
        "assistant" => {
            let mut label = "🤖".to_string();
            if let Some(s) = suffix {
                if !s.is_empty() {
                    label.push(' ');
                    label.push_str(s);
                }
            }
            Some(label)
        }
        _ => None,
    }
}

pub(crate) fn label_line(text: &str, theme: &RenderTheme) -> Line<'static> {
    let style = Style::default()
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White))
        .add_modifier(Modifier::BOLD);
    Line::from(Span::styled(text.to_string(), style))
}

pub(crate) fn hash_message(role: &str, content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    role.hash(&mut hasher);
    content.hash(&mut hasher);
    hasher.finish()
}
