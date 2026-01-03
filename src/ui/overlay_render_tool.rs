use crate::ui::code_exec_popup::draw_code_exec_popup;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::draw::redraw_with_overlay;
use crate::ui::file_patch_popup::draw_file_patch_popup;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::overlay_render_base::render_chat_view;
use crate::ui::render_context::RenderContext;
use ratatui::layout::Rect;
use std::error::Error;

pub(crate) fn render_code_exec_overlay(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    let pending = ctx
        .tabs
        .get(ctx.active_tab)
        .and_then(|tab_state| tab_state.app.pending_code_exec.clone());
    if let Some(pending) = pending {
        render_code_exec_popup(ctx, pending)
    } else {
        render_chat_view(ctx)
    }
}

pub(crate) fn render_file_patch_overlay(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    let pending = ctx
        .tabs
        .get(ctx.active_tab)
        .and_then(|tab_state| tab_state.app.pending_file_patch.clone());
    if let Some(pending) = pending {
        render_file_patch_popup(ctx, pending)
    } else {
        render_chat_view(ctx)
    }
}

fn render_code_exec_popup(
    ctx: &mut RenderContext<'_>,
    pending: crate::ui::state::PendingCodeExec,
) -> Result<(), Box<dyn Error>> {
    let RenderContext {
        terminal,
        tabs,
        active_tab,
        tab_labels,
        active_tab_pos,
        categories,
        active_category,
        theme,
        startup_text,
        input_height,
        text,
        total_lines,
        header_note,
        ..
    } = ctx;
    let Some(tab_state) = tabs.get_mut(*active_tab) else { return Ok(()); };
    let size = terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
    let layout = crate::ui::code_exec_popup_layout::code_exec_popup_layout(
        full,
        tab_state.app.code_exec_reason_target.is_some(),
    );
    let (stdout, stderr, live_snapshot) = snapshot_live(tab_state);
    clamp_code_scroll(theme, tab_state, &pending, layout);
    clamp_output_scrolls(theme, tab_state, &stdout, &stderr, layout);
    let ui_state = read_code_exec_ui(tab_state);
    let mut reason_input = take_reason_input(tab_state);
    redraw_with_overlay(
        terminal,
        &mut tab_state.app,
        theme,
        text,
        *total_lines,
        tab_labels,
        *active_tab_pos,
        categories,
        *active_category,
        *startup_text,
        *input_height,
        |f| {
            draw_code_exec_popup(
                f,
                f.area(),
                &pending,
                ui_state.0,
                ui_state.1,
                ui_state.2,
                ui_state.3,
                ui_state.4,
                &mut reason_input,
                live_snapshot.as_ref(),
                theme,
            );
        },
        *header_note,
    )?;
    tab_state.app.code_exec_reason_input = reason_input;
    Ok(())
}

fn take_reason_input(tab_state: &mut crate::ui::runtime_helpers::TabState) -> tui_textarea::TextArea<'static> {
    std::mem::take(&mut tab_state.app.code_exec_reason_input)
}

fn render_file_patch_popup(
    ctx: &mut RenderContext<'_>,
    pending: crate::ui::state::PendingFilePatch,
) -> Result<(), Box<dyn Error>> {
    let RenderContext {
        terminal,
        tabs,
        active_tab,
        tab_labels,
        active_tab_pos,
        categories,
        active_category,
        theme,
        startup_text,
        input_height,
        text,
        total_lines,
        header_note,
        ..
    } = ctx;
    let Some(tab_state) = tabs.get_mut(*active_tab) else { return Ok(()); };
    let size = terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
    let layout = crate::ui::file_patch_popup_layout::file_patch_popup_layout(full);
    clamp_patch_scroll(theme, tab_state, &pending, layout);
    let scroll = tab_state.app.file_patch_scroll;
    let hover = tab_state.app.file_patch_hover;
    crate::ui::draw::redraw_with_overlay(
        terminal,
        &mut tab_state.app,
        theme,
        text,
        *total_lines,
        tab_labels,
        *active_tab_pos,
        categories,
        *active_category,
        *startup_text,
        *input_height,
        |f| draw_file_patch_popup(f, f.area(), &pending, scroll, hover, theme),
        *header_note,
    )?;
    Ok(())
}

fn clamp_code_scroll(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    layout: crate::ui::code_exec_popup_layout::CodeExecPopupLayout,
) {
    let max_scroll = code_max_scroll(
        &pending.code,
        layout.code_text_area.width,
        layout.code_text_area.height,
        theme,
    );
    if tab_state.app.code_exec_scroll > max_scroll {
        tab_state.app.code_exec_scroll = max_scroll;
    }
}

fn snapshot_live(
    tab_state: &crate::ui::runtime_helpers::TabState,
) -> (String, String, Option<crate::ui::state::CodeExecLive>) {
    tab_state
        .app
        .code_exec_live
        .as_ref()
        .and_then(|l| l.lock().ok())
        .map(|l| (l.stdout.clone(), l.stderr.clone(), Some(l.clone())))
        .unwrap_or_else(|| (String::new(), String::new(), None))
}

fn clamp_output_scrolls(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    stdout: &str,
    stderr: &str,
    layout: crate::ui::code_exec_popup_layout::CodeExecPopupLayout,
) {
    let max_stdout = stdout_max_scroll(
        stdout,
        layout.stdout_text_area.width,
        layout.stdout_text_area.height,
        theme,
    );
    let max_stderr = stderr_max_scroll(
        stderr,
        layout.stderr_text_area.width,
        layout.stderr_text_area.height,
        theme,
    );
    if tab_state.app.code_exec_stdout_scroll > max_stdout {
        tab_state.app.code_exec_stdout_scroll = max_stdout;
    }
    if tab_state.app.code_exec_stderr_scroll > max_stderr {
        tab_state.app.code_exec_stderr_scroll = max_stderr;
    }
}

fn read_code_exec_ui(
    tab_state: &crate::ui::runtime_helpers::TabState,
) -> (
    usize,
    usize,
    usize,
    Option<crate::ui::state::CodeExecHover>,
    Option<crate::ui::state::CodeExecReasonTarget>,
) {
    (
        tab_state.app.code_exec_scroll,
        tab_state.app.code_exec_stdout_scroll,
        tab_state.app.code_exec_stderr_scroll,
        tab_state.app.code_exec_hover,
        tab_state.app.code_exec_reason_target,
    )
}

fn clamp_patch_scroll(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    layout: crate::ui::file_patch_popup_layout::FilePatchPopupLayout,
) {
    let max_scroll = patch_max_scroll(
        &pending.preview,
        layout.preview_area.width,
        layout.preview_area.height,
        theme,
    );
    if tab_state.app.file_patch_scroll > max_scroll {
        tab_state.app.file_patch_scroll = max_scroll;
    }
}
