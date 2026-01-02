use crate::ui::code_exec_popup::draw_code_exec_popup;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::file_patch_popup::draw_file_patch_popup;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::overlay_render_base::render_chat_view;
use crate::ui::render_context::RenderContext;
use ratatui::layout::Rect;
use std::error::Error;

pub(crate) fn render_code_exec_overlay(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    let size = ctx.terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
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
            let (stdout, stderr, live_snapshot) = tab_state
                .app
                .code_exec_live
                .as_ref()
                .and_then(|l| l.lock().ok())
                .map(|l| (l.stdout.clone(), l.stderr.clone(), Some(l.clone())))
                .unwrap_or_else(|| (String::new(), String::new(), None));
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
            let mut reason_input = std::mem::take(&mut tab_state.app.code_exec_reason_input);
            crate::ui::draw::redraw_with_overlay(
                ctx.terminal,
                &mut tab_state.app,
                ctx.theme,
                ctx.text,
                ctx.total_lines,
                ctx.tab_labels,
                ctx.active_tab_pos,
                ctx.categories,
                ctx.active_category,
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

pub(crate) fn render_file_patch_overlay(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    let size = ctx.terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
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
            crate::ui::draw::redraw_with_overlay(
                ctx.terminal,
                &mut tab_state.app,
                ctx.theme,
                ctx.text,
                ctx.total_lines,
                ctx.tab_labels,
                ctx.active_tab_pos,
                ctx.categories,
                ctx.active_category,
                ctx.startup_text,
                ctx.input_height,
                |f| {
                    draw_file_patch_popup(f, f.area(), &pending, scroll, hover, ctx.theme);
                },
                ctx.header_note,
            )?;
        } else {
            render_chat_view(ctx)?;
        }
    }
    Ok(())
}
