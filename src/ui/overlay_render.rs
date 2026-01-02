use crate::ui::draw::{redraw, redraw_with_overlay};
use crate::ui::jump::{JumpRow, build_jump_rows, max_preview_width, redraw_jump};
use crate::ui::code_exec_popup::draw_code_exec_popup;
use crate::ui::code_exec_popup_text::{code_max_scroll, stdout_max_scroll, stderr_max_scroll};
use crate::ui::file_patch_popup::draw_file_patch_popup;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::model_popup::draw_model_popup;
use crate::ui::prompt_popup::draw_prompt_popup;
use crate::ui::shortcut_help::draw_shortcut_help;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_view::ViewState;
use crate::ui::summary::redraw_summary;
use ratatui::layout::Rect;
use std::error::Error;

pub(crate) fn build_jump_overlay_rows(
    view: &ViewState,
    ctx: &RenderContext<'_>,
) -> Vec<JumpRow> {
    if !view.overlay.is(crate::ui::overlay::OverlayKind::Jump) {
        return Vec::new();
    }
    ctx.tabs
        .get(ctx.active_tab)
        .map(|tab| {
            build_jump_rows(
                &tab.app.messages,
                ctx.msg_width,
                max_preview_width(ctx.msg_area),
                tab.app.pending_assistant,
            )
        })
        .unwrap_or_default()
}

pub(crate) fn render_summary_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    let rows = redraw_summary(
        ctx.terminal,
        ctx.tabs,
        ctx.active_tab,
        ctx.theme,
        ctx.startup_text,
        ctx.header_note,
        view.summary.selected,
        view.summary.scroll,
        view.summary_sort,
    )?;
    view.summary_order = rows.iter().map(|r| r.tab_index).collect();
    Ok(())
}

pub(crate) fn render_jump_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
    jump_rows: &[JumpRow],
) -> Result<(), Box<dyn Error>> {
    redraw_jump(
        ctx.terminal,
        ctx.theme,
        ctx.tabs,
        ctx.active_tab,
        ctx.startup_text,
        ctx.header_note,
        jump_rows,
        view.jump.selected,
        ctx.msg_area,
        ctx.header_area,
        ctx.tabs_area,
        ctx.footer_area,
        view.jump.scroll,
    )?;
    Ok(())
}

pub(crate) fn render_chat_view(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    let tabs_len = ctx.tabs.len();
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw(
            ctx.terminal,
            &mut tab_state.app,
            ctx.theme,
            ctx.text,
            ctx.total_lines,
            tabs_len,
            ctx.active_tab,
            ctx.startup_text,
            ctx.input_height,
            ctx.header_note,
        )?;
    }
    Ok(())
}

pub(crate) fn render_model_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    let tabs_len = ctx.tabs.len();
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw_with_overlay(
            ctx.terminal,
            &mut tab_state.app,
            ctx.theme,
            ctx.text,
            ctx.total_lines,
            tabs_len,
            ctx.active_tab,
            ctx.startup_text,
            ctx.input_height,
            |f| {
                draw_model_popup(
                    f,
                    f.area(),
                    ctx.models,
                    view.model.selected,
                    view.model.scroll,
                    ctx.theme,
                );
            },
            ctx.header_note,
        )?;
    }
    Ok(())
}

pub(crate) fn render_prompt_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    let tabs_len = ctx.tabs.len();
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw_with_overlay(
            ctx.terminal,
            &mut tab_state.app,
            ctx.theme,
            ctx.text,
            ctx.total_lines,
            tabs_len,
            ctx.active_tab,
            ctx.startup_text,
            ctx.input_height,
            |f| {
                draw_prompt_popup(
                    f,
                    f.area(),
                    ctx.prompts,
                    view.prompt.selected,
                    view.prompt.scroll,
                    ctx.theme,
                );
            },
            ctx.header_note,
        )?;
    }
    Ok(())
}

pub(crate) fn render_help_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    let tabs_len = ctx.tabs.len();
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw_with_overlay(
            ctx.terminal,
            &mut tab_state.app,
            ctx.theme,
            ctx.text,
            ctx.total_lines,
            tabs_len,
            ctx.active_tab,
            ctx.startup_text,
            ctx.input_height,
            |f| {
                draw_shortcut_help(
                    f,
                    f.area(),
                    view.help.selected,
                    view.help.scroll,
                    ctx.theme,
                );
            },
            ctx.header_note,
        )?;
    }
    Ok(())
}

pub(crate) fn render_code_exec_overlay(
    ctx: &mut RenderContext<'_>,
) -> Result<(), Box<dyn Error>> {
    let size = ctx.terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
    let tabs_len = ctx.tabs.len();
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        let pending = tab_state.app.pending_code_exec.clone();
        if let Some(pending) = pending {
            let layout = crate::ui::code_exec_popup_layout::code_exec_popup_layout(
                full,
                tab_state.app.code_exec_reason_target.is_some(),
            );
            let max_scroll = code_max_scroll(
                &pending.code,
                layout.code_text_area.width,
                layout.code_text_area.height,
                ctx.theme,
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
                ctx.theme,
            );
            let max_stderr_scroll = stderr_max_scroll(
                &stderr,
                layout.stderr_text_area.width,
                layout.stderr_text_area.height,
                ctx.theme,
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
            let reason_target = tab_state.app.code_exec_reason_target;
            let mut reason_input =
                std::mem::take(&mut tab_state.app.code_exec_reason_input);
            redraw_with_overlay(
                ctx.terminal,
                &mut tab_state.app,
                ctx.theme,
                ctx.text,
                ctx.total_lines,
                tabs_len,
                ctx.active_tab,
                ctx.startup_text,
                ctx.input_height,
                |f| {
                    draw_code_exec_popup(
                        f,
                        f.area(),
                        &pending,
                        scroll,
                        stdout_scroll,
                        stderr_scroll,
                        hover,
                        reason_target,
                        &mut reason_input,
                        live_snapshot.as_ref(),
                        ctx.theme,
                    );
                },
                ctx.header_note,
            )?;
            tab_state.app.code_exec_reason_input = reason_input;
        } else {
            render_chat_view(ctx)?;
        }
    }
    Ok(())
}

pub(crate) fn render_file_patch_overlay(
    ctx: &mut RenderContext<'_>,
) -> Result<(), Box<dyn Error>> {
    let size = ctx.terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
    let tabs_len = ctx.tabs.len();
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        let pending = tab_state.app.pending_file_patch.clone();
        if let Some(pending) = pending {
            let layout = crate::ui::file_patch_popup_layout::file_patch_popup_layout(full);
            let max_scroll = patch_max_scroll(
                &pending.preview,
                layout.preview_area.width,
                layout.preview_area.height,
                ctx.theme,
            );
            if tab_state.app.file_patch_scroll > max_scroll {
                tab_state.app.file_patch_scroll = max_scroll;
            }
            let scroll = tab_state.app.file_patch_scroll;
            let hover = tab_state.app.file_patch_hover;
            redraw_with_overlay(
                ctx.terminal,
                &mut tab_state.app,
                ctx.theme,
                ctx.text,
                ctx.total_lines,
                tabs_len,
                ctx.active_tab,
                ctx.startup_text,
                ctx.input_height,
                |f| {
                    draw_file_patch_popup(
                        f,
                        f.area(),
                        &pending,
                        scroll,
                        hover,
                        ctx.theme,
                    );
                },
                ctx.header_note,
            )?;
        } else {
            render_chat_view(ctx)?;
        }
    }
    Ok(())
}
