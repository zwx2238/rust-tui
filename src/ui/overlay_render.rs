use crate::render::RenderTheme;
use crate::ui::draw::{redraw, redraw_with_overlay};
use crate::ui::jump::{JumpRow, build_jump_rows, max_preview_width, redraw_jump};
use crate::ui::model_popup::draw_model_popup;
use crate::ui::prompt_popup::draw_prompt_popup;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::ViewState;
use crate::ui::summary::redraw_summary;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::error::Error;
use std::io::Stdout;

pub(crate) fn build_jump_overlay_rows(
    view: &ViewState,
    tabs: &[TabState],
    active_tab: usize,
    msg_width: usize,
    msg_area: Rect,
) -> Vec<JumpRow> {
    if !view.overlay.is(crate::ui::overlay::OverlayKind::Jump) {
        return Vec::new();
    }
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
}

pub(crate) fn render_summary_overlay(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    theme: &RenderTheme,
    startup_text: Option<&str>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    redraw_summary(
        terminal,
        tabs,
        active_tab,
        theme,
        startup_text,
        view.summary.selected,
        view.summary.scroll,
    )?;
    Ok(())
}

pub(crate) fn render_jump_overlay(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    theme: &RenderTheme,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    startup_text: Option<&str>,
    view: &mut ViewState,
    msg_area: Rect,
    tabs_area: Rect,
    header_area: Rect,
    footer_area: Rect,
    jump_rows: &[JumpRow],
) -> Result<(), Box<dyn Error>> {
    redraw_jump(
        terminal,
        theme,
        tabs,
        active_tab,
        startup_text,
        jump_rows,
        view.jump.selected,
        msg_area,
        header_area,
        tabs_area,
        footer_area,
        view.jump.scroll,
    )?;
    Ok(())
}

pub(crate) fn render_chat_view(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    startup_text: Option<&str>,
    input_height: u16,
) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

pub(crate) fn render_model_overlay(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    startup_text: Option<&str>,
    input_height: u16,
    view: &mut ViewState,
    models: &[crate::model_registry::ModelProfile],
) -> Result<(), Box<dyn Error>> {
    let tabs_len = tabs.len();
    if let Some(tab_state) = tabs.get_mut(active_tab) {
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
    Ok(())
}

pub(crate) fn render_prompt_overlay(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    startup_text: Option<&str>,
    input_height: u16,
    view: &mut ViewState,
    prompts: &[crate::system_prompts::SystemPrompt],
) -> Result<(), Box<dyn Error>> {
    let tabs_len = tabs.len();
    if let Some(tab_state) = tabs.get_mut(active_tab) {
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
    Ok(())
}
