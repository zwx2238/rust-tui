use crate::render::RenderTheme;
use crate::ui::draw::{redraw, redraw_with_overlay};
use crate::ui::jump::{build_jump_rows, max_preview_width, redraw_jump, JumpRow};
use crate::ui::model_popup::draw_model_popup;
use crate::ui::prompt_popup::{draw_prompt_popup, prompt_visible_rows};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::{ViewMode, ViewState};
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
    let jump_rows = if view.mode == ViewMode::Jump {
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
    match view.mode {
        ViewMode::Summary => {
            view.summary_selected = view.summary_selected.min(tabs.len().saturating_sub(1));
            redraw_summary(
                terminal,
                tabs,
                active_tab,
                theme,
                startup_text,
                view.summary_selected,
            )?;
        }
        ViewMode::Jump => {
            let max_scroll = jump_rows
                .len()
                .saturating_sub(visible_jump_rows(msg_area))
                .max(1)
                .saturating_sub(1);
            view.jump_scroll = view.jump_scroll.min(max_scroll);
            view.jump_selected = view.jump_selected.min(jump_rows.len().saturating_sub(1));
            let viewport_rows = visible_jump_rows(msg_area);
            if view.jump_selected >= view.jump_scroll + viewport_rows {
                view.jump_scroll = view
                    .jump_selected
                    .saturating_sub(viewport_rows.saturating_sub(1));
            }
            redraw_jump(
                terminal,
                theme,
                tabs.len(),
                active_tab,
                startup_text,
                &jump_rows,
                view.jump_selected,
                msg_area,
                tabs_area,
                view.jump_scroll,
            )?;
        }
        ViewMode::Chat => {
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
        ViewMode::Model => {
            let tabs_len = tabs.len();
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                if !models.is_empty() {
                    view.model_selected = view.model_selected.min(models.len() - 1);
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
                            view.model_selected,
                            0,
                            theme,
                        );
                    },
                )?;
            }
        }
        ViewMode::Prompt => {
            let tabs_len = tabs.len();
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                if !prompts.is_empty() {
                    view.prompt_selected = view.prompt_selected.min(prompts.len() - 1);
                }
                let viewport_rows = prompt_visible_rows(full_area, prompts.len());
                let max_scroll = prompts
                    .len()
                    .saturating_sub(viewport_rows)
                    .max(1)
                    .saturating_sub(1);
                view.prompt_scroll = view.prompt_scroll.min(max_scroll);
                if view.prompt_selected < view.prompt_scroll {
                    view.prompt_scroll = view.prompt_selected;
                }
                if view.prompt_selected >= view.prompt_scroll + viewport_rows {
                    view.prompt_scroll = view
                        .prompt_selected
                        .saturating_sub(viewport_rows.saturating_sub(1));
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
                        draw_prompt_popup(
                            f,
                            f.area(),
                            prompts,
                            view.prompt_selected,
                            view.prompt_scroll,
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
