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

pub(crate) fn draw_code_exec_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingCodeExec,
    scroll: usize,
    stdout_scroll: usize,
    stderr_scroll: usize,
    hover: Option<CodeExecHover>,
    reason_target: Option<CodeExecReasonTarget>,
    reason_input: &mut TextArea<'_>,
    live: Option<&crate::ui::state::CodeExecLive>,
    theme: &RenderTheme,
) {
    let layout = code_exec_popup_layout(area, reason_target.is_some());
    let mask = popup_mask(area, layout.popup);
    render_mask(f, theme, mask);
    render_popup_base(f, theme, layout.popup, &build_title(live));
    render_code_panel(f, theme, pending, layout, scroll);
    render_stdout_panel(f, theme, layout, live, stdout_scroll);
    render_stderr_panel(f, theme, layout, live, stderr_scroll);
    if let Some(target) = reason_target {
        draw_reason_input(f, layout.reason_input_area, reason_input, target, theme);
    }
    render_action_buttons(f, layout, hover, reason_target, live, theme);
}
