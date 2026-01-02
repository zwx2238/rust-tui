use crate::render::RenderTheme;
use crate::ui::draw::{redraw, redraw_with_overlay};
use crate::ui::jump::{JumpRow, build_jump_rows, max_preview_width, redraw_jump};
use crate::ui::code_exec_popup::draw_code_exec_popup;
use crate::ui::code_exec_popup_text::{code_max_scroll, stdout_max_scroll, stderr_max_scroll};
use crate::ui::model_popup::draw_model_popup;
use crate::ui::prompt_popup::draw_prompt_popup;
use crate::ui::shortcut_help::draw_shortcut_help;
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
    header_note: Option<&str>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    let rows = redraw_summary(
        terminal,
        tabs,
        active_tab,
        theme,
        startup_text,
        header_note,
        view.summary.selected,
        view.summary.scroll,
        view.summary_sort,
    )?;
    view.summary_order = rows.iter().map(|r| r.tab_index).collect();
    Ok(())
}

pub(crate) fn render_jump_overlay(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    theme: &RenderTheme,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    startup_text: Option<&str>,
    header_note: Option<&str>,
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
        header_note,
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
    header_note: Option<&str>,
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
            header_note,
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
    header_note: Option<&str>,
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
            header_note,
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
    header_note: Option<&str>,
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
            header_note,
        )?;
    }
    Ok(())
}

pub(crate) fn render_help_overlay(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    startup_text: Option<&str>,
    input_height: u16,
    header_note: Option<&str>,
    view: &mut ViewState,
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
                draw_shortcut_help(
                    f,
                    f.area(),
                    view.help.selected,
                    view.help.scroll,
                    theme,
                );
            },
            header_note,
        )?;
    }
    Ok(())
}

pub(crate) fn render_code_exec_overlay(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    startup_text: Option<&str>,
    input_height: u16,
    header_note: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let size = terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
    let tabs_len = tabs.len();
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let pending = tab_state.app.pending_code_exec.clone();
        if let Some(pending) = pending {
            let layout = crate::ui::code_exec_popup_layout::code_exec_popup_layout(full);
            let max_scroll = code_max_scroll(
                &pending.code,
                layout.code_text_area.width,
                layout.code_text_area.height,
                theme,
            );
            if tab_state.app.code_exec_scroll > max_scroll {
                tab_state.app.code_exec_scroll = max_scroll;
            }
            let live_snapshot = tab_state
                .app
                .code_exec_live
                .as_ref()
                .and_then(|l| l.lock().ok().map(|l| {
                    crate::ui::state::CodeExecLive {
                        started_at: l.started_at,
                        finished_at: l.finished_at,
                        stdout: l.stdout.clone(),
                        stderr: l.stderr.clone(),
                        exit_code: l.exit_code,
                        done: l.done,
                    }
                }));
            let (stdout, stderr) = live_snapshot
                .as_ref()
                .map(|l| (l.stdout.clone(), l.stderr.clone()))
                .unwrap_or_else(|| (String::new(), String::new()));
            let max_stdout_scroll = stdout_max_scroll(
                &stdout,
                layout.stdout_text_area.width,
                layout.stdout_text_area.height,
                theme,
            );
            let max_stderr_scroll = stderr_max_scroll(
                &stderr,
                layout.stderr_text_area.width,
                layout.stderr_text_area.height,
                theme,
            );
            if tab_state.app.code_exec_stdout_scroll > max_stdout_scroll {
                tab_state.app.code_exec_stdout_scroll = max_stdout_scroll;
            }
            if tab_state.app.code_exec_stderr_scroll > max_stderr_scroll {
                tab_state.app.code_exec_stderr_scroll = max_stderr_scroll;
            }
            let scroll = tab_state.app.code_exec_scroll;
            let stdout_scroll = tab_state.app.code_exec_stdout_scroll;
            let stderr_scroll = tab_state.app.code_exec_stderr_scroll;
            let hover = tab_state.app.code_exec_hover;
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
                    draw_code_exec_popup(
                        f,
                        f.area(),
                        &pending,
                        scroll,
                        stdout_scroll,
                        stderr_scroll,
                        hover,
                        live_snapshot.as_ref(),
                        theme,
                    );
                },
                header_note,
            )?;
        } else {
            render_chat_view(
                terminal,
                tabs,
                active_tab,
                theme,
                text,
                total_lines,
                startup_text,
                input_height,
                header_note,
            )?;
        }
    }
    Ok(())
}
