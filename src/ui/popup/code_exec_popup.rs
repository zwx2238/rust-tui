#[path = "code_exec_popup_render.rs"]
mod code_exec_popup_render;

use crate::render::RenderTheme;
use crate::ui::code_exec_popup_layout::code_exec_popup_layout;
use crate::ui::state::{CodeExecReasonTarget, PendingCodeExec};
use ratatui::layout::Rect;
use tui_textarea::TextArea;

use code_exec_popup_render::{
    build_title, draw_reason_input, popup_mask, render_code_panel, render_mask, render_popup_base,
    render_stderr_panel, render_stdout_panel,
};

pub(crate) struct CodeExecPopupParams<'a, 'b> {
    pub area: Rect,
    pub pending: &'a PendingCodeExec,
    pub scroll: usize,
    pub stdout_scroll: usize,
    pub stderr_scroll: usize,
    pub reason_target: Option<CodeExecReasonTarget>,
    pub reason_input: &'a mut TextArea<'b>,
    pub live: Option<&'a crate::ui::state::CodeExecLive>,
    pub code_selection: Option<crate::ui::selection::Selection>,
    pub stdout_selection: Option<crate::ui::selection::Selection>,
    pub stderr_selection: Option<crate::ui::selection::Selection>,
    pub theme: &'a RenderTheme,
}

pub(crate) fn draw_code_exec_popup_base<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: &mut CodeExecPopupParams<'a, 'b>,
) {
    let layout = code_exec_popup_layout(params.area, params.reason_target.is_some());
    render_popup_base_layer(f, params, layout);
    render_panels(f, params, layout);
    render_reason_if_needed(f, params, layout);
}

fn render_popup_base_layer<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: &CodeExecPopupParams<'a, 'b>,
    layout: crate::ui::code_exec_popup_layout::CodeExecPopupLayout,
) {
    let mask = popup_mask(params.area, layout.popup);
    render_mask(f, params.theme, mask);
    render_popup_base(f, params.theme, layout.popup, &build_title(params.live));
}

fn render_panels<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: &CodeExecPopupParams<'a, 'b>,
    layout: crate::ui::code_exec_popup_layout::CodeExecPopupLayout,
) {
    render_code_panel(
        f,
        params.theme,
        params.pending,
        layout,
        params.scroll,
        params.code_selection,
    );
    render_stdout_panel(
        f,
        params.theme,
        layout,
        params.live,
        params.stdout_scroll,
        params.stdout_selection,
    );
    render_stderr_panel(
        f,
        params.theme,
        layout,
        params.live,
        params.stderr_scroll,
        params.stderr_selection,
    );
}

fn render_reason_if_needed<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: &mut CodeExecPopupParams<'a, 'b>,
    layout: crate::ui::code_exec_popup_layout::CodeExecPopupLayout,
) {
    if let Some(target) = params.reason_target {
        draw_reason_input(
            f,
            layout.reason_input_area,
            params.reason_input,
            target,
            params.theme,
        );
    }
}
