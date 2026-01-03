use crate::ui::code_exec_popup::draw_code_exec_popup;
use crate::ui::code_exec_popup_layout::code_exec_popup_layout;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::OverlayTableController;

pub(crate) struct CodeExecWidget {
    _private: (),
}

impl CodeExecWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for CodeExecWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        match event {
            crossterm::event::Event::Mouse(m) => {
                let mut binding = bind_event(ctx, layout, update);
                let handled = crate::ui::runtime_dispatch::mouse_overlay::handle_code_exec_overlay_mouse(
                    *m,
                    &mut binding.dispatch,
                    binding.layout,
                    binding.view,
                );
                if handled {
                    return Ok(EventResult::handled());
                }
                Ok(EventResult::ignored())
            }
            crossterm::event::Event::Key(_) => {
                let mut binding = bind_event(ctx, layout, update);
                let mut controller = OverlayTableController {
                    dispatch: binding.dispatch,
                    layout: binding.layout,
                    view: binding.view,
                    jump_rows,
                };
                controller.handle_event(event)
            }
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(result) = frame
            .state
            .with_active_tab_mut(|tab_state| -> Result<(), Box<dyn Error>> {
            let Some(pending) = tab_state.app.pending_code_exec.clone() else {
                return Ok(());
            };
            let popup = code_exec_popup_layout(
                frame.frame.area(),
                tab_state.app.code_exec_reason_target.is_some(),
            );
            let live_snapshot =
                prepare_code_exec_overlay(frame.state.theme, tab_state, &pending, popup);
            let ui_state = read_code_exec_ui(tab_state);
            let mut reason_input = std::mem::take(&mut tab_state.app.code_exec_reason_input);
            draw_code_exec_popup(
                frame.frame,
                crate::ui::code_exec_popup::CodeExecPopupParams {
                    area: frame.frame.area(),
                    pending: &pending,
                    scroll: ui_state.0,
                    stdout_scroll: ui_state.1,
                    stderr_scroll: ui_state.2,
                    hover: ui_state.3,
                    reason_target: ui_state.4,
                    reason_input: &mut reason_input,
                    live: live_snapshot.as_ref(),
                    theme: frame.state.theme,
                },
            );
            tab_state.app.code_exec_reason_input = reason_input;
            Ok(())
        }) {
            result?;
        }
        Ok(())
    }
}

fn prepare_code_exec_overlay(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    layout: crate::ui::code_exec_popup_layout::CodeExecPopupLayout,
) -> Option<crate::ui::state::CodeExecLive> {
    let (stdout, stderr, live_snapshot) = snapshot_live(tab_state);
    clamp_code_scroll(theme, tab_state, pending, layout);
    clamp_output_scrolls(theme, tab_state, &stdout, &stderr, layout);
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
