use crate::render::RenderTheme;
use crate::ui::draw::{redraw, redraw_with_overlay};
use crate::ui::jump::{build_jump_rows, max_preview_width, redraw_jump, JumpRow};
use crate::ui::model_popup::draw_model_popup;
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
            let max_scroll = jump_rows
                .len()
                .saturating_sub(visible_jump_rows(msg_area))
                .max(1)
                .saturating_sub(1);
            view.jump.scroll = view.jump.scroll.min(max_scroll);
            view.jump.clamp(jump_rows.len());
            let viewport_rows = visible_jump_rows(msg_area);
            view.jump.ensure_visible(viewport_rows);
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
                if !models.is_empty() {
                    view.model.clamp(models.len());
                }
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
                            0,
                            theme,
                        );
                    },
                )?;
            }
        }
        Some(OverlayKind::Prompt) => {
            let tabs_len = tabs.len();
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                view.prompt.clamp(prompts.len());
                let viewport_rows = prompt_visible_rows(full_area, prompts.len());
                let max_scroll = prompts
                    .len()
                    .saturating_sub(viewport_rows)
                    .max(1)
                    .saturating_sub(1);
                view.prompt.scroll = view.prompt.scroll.min(max_scroll);
                view.prompt.ensure_visible(viewport_rows);
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

fn visible_jump_rows(area: Rect) -> usize {
    area.height.saturating_sub(2).saturating_sub(1) as usize
}
