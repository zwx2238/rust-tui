use crate::render::markdown::preprocess_math;
use crate::render::markdown::render_state::RenderState;
use crate::render::markdown::shared::markdown_parser;
use crate::render::theme::RenderTheme;
use ratatui::text::Line;

pub fn render_markdown_lines(
    text: &str,
    width: usize,
    theme: &RenderTheme,
    streaming: bool,
    show_code_line_numbers: bool,
) -> Vec<Line<'static>> {
    let text = preprocess_math(text);
    let parser = markdown_parser(&text);
    let mut state = RenderState::new(width, theme, streaming, show_code_line_numbers);
    for event in parser {
        state.handle_event(event);
    }
    state.finish();
    state.into_lines()
}
