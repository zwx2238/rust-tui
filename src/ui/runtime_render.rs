use crate::render::RenderTheme;
use crate::ui::jump::JumpRow;
use crate::ui::overlay::OverlayKind;
use crate::ui::overlay_render::{
    build_jump_overlay_rows, render_chat_view, render_jump_overlay, render_model_overlay,
    render_prompt_overlay, render_summary_overlay,
};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::ViewState;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::error::Error;
use std::io::Stdout;

pub(crate) fn render_view(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    theme: &RenderTheme,
    startup_text: Option<&str>,
    full_area: Rect,
    input_height: u16,
    msg_area: Rect,
    tabs_area: Rect,
    msg_width: usize,
    text: &Text<'_>,
    total_lines: usize,
    view: &mut ViewState,
    models: &[crate::model_registry::ModelProfile],
    prompts: &[crate::system_prompts::SystemPrompt],
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let jump_rows = build_jump_overlay_rows(view, tabs, active_tab, msg_width, msg_area);
    match view.overlay.active {
        Some(OverlayKind::Summary) => {
            render_summary_overlay(terminal, tabs, active_tab, theme, startup_text, view)?;
        }
        Some(OverlayKind::Jump) => {
            render_jump_overlay(
                terminal,
                theme,
                tabs.len(),
                active_tab,
                startup_text,
                view,
                msg_area,
                tabs_area,
                &jump_rows,
            )?;
        }
        None => {
            render_chat_view(
                terminal,
                tabs,
                active_tab,
                theme,
                text,
                total_lines,
                startup_text,
                input_height,
            )?;
        }
        Some(OverlayKind::Model) => {
            render_model_overlay(
                terminal,
                tabs,
                active_tab,
                theme,
                text,
                total_lines,
                startup_text,
                input_height,
                full_area,
                view,
                models,
            )?;
        }
        Some(OverlayKind::Prompt) => {
            render_prompt_overlay(
                terminal,
                tabs,
                active_tab,
                theme,
                text,
                total_lines,
                startup_text,
                input_height,
                full_area,
                view,
                prompts,
            )?;
        }
    }
    Ok(jump_rows)
}
