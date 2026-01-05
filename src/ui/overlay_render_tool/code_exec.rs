use crate::ui::code_exec_popup::draw_code_exec_popup;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::draw::redraw_with_overlay;
use crate::ui::render_context::RenderContext;
use std::error::Error;

use super::{OverlayRenderContext, build_redraw_params, split_overlay_context};

pub(super) fn render_code_exec_popup(
    ctx: &mut RenderContext<'_>,
    pending: crate::ui::state::PendingCodeExec,
) -> Result<(), Box<dyn Error>> {
    let Some(mut parts) = split_overlay_context(ctx) else {
        return Ok(());
    };
    let layout = code_exec_layout(&mut parts)?;
    let live_snapshot = prepare_code_exec_overlay(&mut parts, &pending, layout);
    let ui_state = read_code_exec_ui(parts.tab_state);
    let mut reason_input = take_reason_input(parts.tab_state);
    let theme = parts.theme;
    let redraw_result = {
        let redraw = build_redraw_params(&mut parts);
        redraw_with_overlay(redraw, |f| {
            draw_code_exec_popup_frame(
                f,
                &pending,
                ui_state,
                &mut reason_input,
                live_snapshot.as_ref(),
                theme,
            );
        })
    };
    parts.tab_state.app.code_exec_reason_input = reason_input;
    redraw_result?;
    Ok(())
}

fn take_reason_input(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
) -> tui_textarea::TextArea<'static> {
    std::mem::take(&mut tab_state.app.code_exec_reason_input)
}

fn draw_code_exec_popup_frame(
    f: &mut ratatui::Frame<'_>,
    pending: &crate::ui::state::PendingCodeExec,
    ui_state: (
        usize,
        usize,
        usize,
        Option<crate::ui::state::CodeExecHover>,
        Option<crate::ui::state::CodeExecReasonTarget>,
    ),
    reason_input: &mut tui_textarea::TextArea<'static>,
    live_snapshot: Option<&crate::ui::state::CodeExecLive>,
    theme: &crate::render::RenderTheme,
) {
    let params = build_code_exec_popup_params(
        f,
        pending,
        ui_state,
        reason_input,
        live_snapshot,
        theme,
    );
    draw_code_exec_popup(f, params);
}

fn build_code_exec_popup_params<'a>(
    f: &mut ratatui::Frame<'_>,
    pending: &'a crate::ui::state::PendingCodeExec,
    ui_state: (
        usize,
        usize,
        usize,
        Option<crate::ui::state::CodeExecHover>,
        Option<crate::ui::state::CodeExecReasonTarget>,
    ),
    reason_input: &'a mut tui_textarea::TextArea<'static>,
    live_snapshot: Option<&'a crate::ui::state::CodeExecLive>,
    theme: &'a crate::render::RenderTheme,
) -> crate::ui::code_exec_popup::CodeExecPopupParams<'a, 'static> {
    crate::ui::code_exec_popup::CodeExecPopupParams {
        area: f.area(),
        pending,
        scroll: ui_state.0,
        stdout_scroll: ui_state.1,
        stderr_scroll: ui_state.2,
        hover: ui_state.3,
        reason_target: ui_state.4,
        reason_input,
        live: live_snapshot,
        code_selection: None,
        stdout_selection: None,
        stderr_selection: None,
        theme,
    }
}

fn code_exec_layout(
    parts: &mut OverlayRenderContext<'_>,
) -> Result<crate::ui::code_exec_popup_layout::CodeExecPopupLayout, Box<dyn Error>> {
    let full = parts.terminal.get_frame().area();
    Ok(crate::ui::code_exec_popup_layout::code_exec_popup_layout(
        full,
        parts.tab_state.app.code_exec_reason_target.is_some(),
    ))
}

fn prepare_code_exec_overlay(
    parts: &mut OverlayRenderContext<'_>,
    pending: &crate::ui::state::PendingCodeExec,
    layout: crate::ui::code_exec_popup_layout::CodeExecPopupLayout,
) -> Option<crate::ui::state::CodeExecLive> {
    let (stdout, stderr, live_snapshot) = snapshot_live(parts.tab_state);
    clamp_code_scroll(parts.theme, parts.tab_state, pending, layout);
    clamp_output_scrolls(parts.theme, parts.tab_state, &stdout, &stderr, layout);
    live_snapshot
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
