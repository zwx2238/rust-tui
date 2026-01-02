use crate::render::RenderTheme;
use crate::ui::runtime_helpers::TabState;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::text::Text;
use std::io::Stdout;

pub struct RenderContext<'a> {
    pub terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: usize,
    pub theme: &'a RenderTheme,
    pub startup_text: Option<&'a str>,
    pub full_area: Rect,
    pub input_height: u16,
    pub msg_area: Rect,
    pub tabs_area: Rect,
    pub header_area: Rect,
    pub footer_area: Rect,
    pub msg_width: usize,
    pub text: &'a Text<'a>,
    pub total_lines: usize,
    pub header_note: Option<&'a str>,
    pub models: &'a [crate::model_registry::ModelProfile],
    pub prompts: &'a [crate::system_prompts::SystemPrompt],
}
