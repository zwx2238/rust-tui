use crate::ui::code_exec_popup::draw_code_exec_popup;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::draw::redraw_with_overlay;
use crate::ui::file_patch_popup::draw_file_patch_popup;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::overlay_render_base::render_chat_view;
use crate::ui::render_context::RenderContext;
use ratatui::layout::Rect;
use std::error::Error;

struct OverlayRenderContext<'a> {
    terminal: &'a mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    tab_state: &'a mut crate::ui::runtime_helpers::TabState,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    theme: &'a crate::render::RenderTheme,
    startup_text: Option<&'a str>,
    input_height: u16,
    text: &'a ratatui::text::Text<'a>,
    total_lines: usize,
    header_note: Option<&'a str>,
}

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

fn render_file_patch_popup(
    ctx: &mut RenderContext<'_>,
    pending: crate::ui::state::PendingFilePatch,
) -> Result<(), Box<dyn Error>> {
    let Some(mut parts) = split_overlay_context(ctx) else {
        return Ok(());
    };
    let layout = file_patch_layout(&mut parts)?;
    let theme = parts.theme;
    clamp_patch_scroll(theme, parts.tab_state, &pending, layout);
    let scroll = parts.tab_state.app.file_patch_scroll;
    let hover = parts.tab_state.app.file_patch_hover;
    let selection = parts.tab_state.app.file_patch_selection;
    let redraw = build_redraw_params(&mut parts);
    redraw_with_overlay(redraw, |f| {
        draw_file_patch_popup(f, f.area(), &pending, scroll, hover, selection, theme);
    })?;
    Ok(())
}

fn split_overlay_context<'a>(ctx: &'a mut RenderContext<'_>) -> Option<OverlayRenderContext<'a>> {
    let parts = overlay_context_parts(ctx);
    let tab_state = parts.tabs.get_mut(parts.active_tab)?;
    Some(OverlayRenderContext {
        terminal: parts.terminal,
        tab_state,
        tab_labels: parts.tab_labels,
        active_tab_pos: parts.active_tab_pos,
        categories: parts.categories,
        active_category: parts.active_category,
        theme: parts.theme,
        startup_text: parts.startup_text,
        input_height: parts.input_height,
        text: parts.text,
        total_lines: parts.total_lines,
        header_note: parts.header_note,
    })
}

fn overlay_context_parts<'a>(ctx: &'a mut RenderContext<'_>) -> OverlayContextParts<'a> {
    let RenderContext {
        terminal, tabs, active_tab, tab_labels, active_tab_pos, categories, active_category, theme,
        startup_text, input_height, text, total_lines, header_note, ..
    } = ctx;
    OverlayContextParts {
        terminal, tabs,
        active_tab: *active_tab, tab_labels, active_tab_pos: *active_tab_pos,
        categories, active_category: *active_category,
        theme, startup_text: *startup_text, input_height: *input_height,
        text, total_lines: *total_lines, header_note: *header_note,
    }
}

struct OverlayContextParts<'a> {
    terminal: &'a mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    tabs: &'a mut Vec<crate::ui::runtime_helpers::TabState>,
    active_tab: usize,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    theme: &'a crate::render::RenderTheme,
    startup_text: Option<&'a str>,
    input_height: u16,
    text: &'a ratatui::text::Text<'a>,
    total_lines: usize,
    header_note: Option<&'a str>,
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
    draw_code_exec_popup(
        f,
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
        },
    );
}

fn build_redraw_params<'a>(
    parts: &'a mut OverlayRenderContext<'_>,
) -> crate::ui::draw::RedrawWithOverlayParams<'a> {
    crate::ui::draw::RedrawWithOverlayParams {
        terminal: parts.terminal,
        app: &mut parts.tab_state.app,
        theme: parts.theme,
        text: parts.text,
        total_lines: parts.total_lines,
        tab_labels: parts.tab_labels,
        active_tab_pos: parts.active_tab_pos,
        categories: parts.categories,
        active_category: parts.active_category,
        startup_text: parts.startup_text,
        input_height: parts.input_height,
        header_note: parts.header_note,
    }
}

fn code_exec_layout(
    parts: &mut OverlayRenderContext<'_>,
) -> Result<crate::ui::code_exec_popup_layout::CodeExecPopupLayout, Box<dyn Error>> {
    let size = parts.terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
    Ok(crate::ui::code_exec_popup_layout::code_exec_popup_layout(
        full,
        parts.tab_state.app.code_exec_reason_target.is_some(),
    ))
}

fn file_patch_layout(
    parts: &mut OverlayRenderContext<'_>,
) -> Result<crate::ui::file_patch_popup_layout::FilePatchPopupLayout, Box<dyn Error>> {
    let size = parts.terminal.size()?;
    let full = Rect::new(0, 0, size.width, size.height);
    Ok(crate::ui::file_patch_popup_layout::file_patch_popup_layout(full))
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
