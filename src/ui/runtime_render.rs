use crate::render::RenderTheme;
use crate::ui::draw::{redraw, redraw_with_overlay};
use crate::ui::jump::{
    build_jump_rows, jump_visible_rows, max_preview_width, redraw_jump, JumpRow,
};
use crate::ui::model_popup::{draw_model_popup, model_visible_rows};
use crate::ui::prompt_popup::{draw_prompt_popup, prompt_visible_rows};
use crate::ui::runtime_helpers::TabState;
use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_view::ViewState;
use crate::ui::summary::redraw_summary;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::{backend::CrosstermBackend, Terminal};
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
    let jump_rows = if view.overlay.is(OverlayKind::Jump) {
        tabs.get(active_tab)
            .map(|tab| {
                build_jump_rows(
                    &tab.app.messages,
                    msg_width,
                    max_preview_width(msg_area),
                    tab.app.pending_assistant,
                )
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    match view.overlay.active {
        Some(OverlayKind::Summary) => {
            view.summary.clamp(tabs.len());
            redraw_summary(
                terminal,
                tabs,
                active_tab,
                theme,
                startup_text,
                view.summary.selected,
            )?;
        }
        Some(OverlayKind::Jump) => {
            let viewport_rows = jump_visible_rows(msg_area);
            view.jump.clamp_with_viewport(jump_rows.len(), viewport_rows);
            redraw_jump(
                terminal,
                theme,
                tabs.len(),
                active_tab,
                startup_text,
                &jump_rows,
                view.jump.selected,
                msg_area,
                tabs_area,
                view.jump.scroll,
            )?;
        }
        None => {
            let tabs_len = tabs.len();
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                redraw(
                    terminal,
                    &mut tab_state.app,
                    theme,
                    text,
                    total_lines,
                    tabs_len,
                    active_tab,
                    startup_text,
                    input_height,
                )?;
            }
        }
        Some(OverlayKind::Model) => {
            let tabs_len = tabs.len();
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                let viewport_rows = model_visible_rows(full_area, models.len());
                view.model.clamp_with_viewport(models.len(), viewport_rows);
                redraw_with_overlay(
                    terminal,
                    &mut tab_state.app,
                    theme,
                    text,
                    total_lines,
                    tabs_len,
                    active_tab,
                    startup_text,
                    input_height,
                    |f| {
                        draw_model_popup(
                            f,
                            f.area(),
                            models,
                            view.model.selected,
                            view.model.scroll,
                            theme,
                        );
                    },
                )?;
            }
        }
        Some(OverlayKind::Prompt) => {
            let tabs_len = tabs.len();
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                let viewport_rows = prompt_visible_rows(full_area, prompts.len());
                view.prompt.clamp_with_viewport(prompts.len(), viewport_rows);
                redraw_with_overlay(
                    terminal,
                    &mut tab_state.app,
                    theme,
                    text,
                    total_lines,
                    tabs_len,
                    active_tab,
                    startup_text,
                    input_height,
                    |f| {
                        draw_prompt_popup(
                            f,
                            f.area(),
                            prompts,
                            view.prompt.selected,
                            view.prompt.scroll,
                            theme,
                        );
                    },
                )?;
            }
        }
    }
    Ok(jump_rows)
}
