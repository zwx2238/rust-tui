#[path = "code_exec_popup_render.rs"]
mod code_exec_popup_render;

use crate::render::RenderTheme;
use crate::ui::code_exec_popup_layout::code_exec_popup_layout;
use crate::ui::state::{CodeExecHover, CodeExecReasonTarget, PendingCodeExec};
use ratatui::layout::Rect;
use tui_textarea::TextArea;

use code_exec_popup_render::{
    build_title, draw_reason_input, popup_mask, render_action_buttons, render_code_panel,
    render_mask, render_popup_base, render_stderr_panel, render_stdout_panel,
};

pub(crate) struct CodeExecPopupParams<'a, 'b> {
    pub area: Rect,
    pub pending: &'a PendingCodeExec,
    pub scroll: usize,
    pub stdout_scroll: usize,
    pub stderr_scroll: usize,
    pub hover: Option<CodeExecHover>,
    pub reason_target: Option<CodeExecReasonTarget>,
    pub reason_input: &'a mut TextArea<'b>,
    pub live: Option<&'a crate::ui::state::CodeExecLive>,
    pub theme: &'a RenderTheme,
}

pub(crate) fn draw_code_exec_popup<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: CodeExecPopupParams<'a, 'b>,
) {
    let layout = code_exec_popup_layout(params.area, params.reason_target.is_some());
    let mask = popup_mask(params.area, layout.popup);
    render_mask(f, params.theme, mask);
    render_popup_base(f, params.theme, layout.popup, &build_title(params.live));
    render_code_panel(f, params.theme, params.pending, layout, params.scroll);
    render_stdout_panel(f, params.theme, layout, params.live, params.stdout_scroll);
    render_stderr_panel(f, params.theme, layout, params.live, params.stderr_scroll);
    if let Some(target) = params.reason_target {
        draw_reason_input(
            f,
            layout.reason_input_area,
            params.reason_input,
            target,
            params.theme,
        );
    }
    render_action_buttons(
        f,
        layout,
        params.hover,
        params.reason_target,
        params.live,
        params.theme,
    );
}
