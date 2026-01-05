use crate::ui::state::{CodeExecHover, CodeExecReasonTarget, PendingCommand};
use crate::ui::widget_system::context::{EventCtx, UpdateOutput};
use crate::ui::widget_system::lifecycle::Widget;
use crate::ui::runtime_loop_steps::FrameLayout;

use super::mode::{CodeExecButtonsMode, resolve_code_exec_mode};
use super::render::configure_buttons;
use super::super::helpers::point_in_rect;
use super::super::widget::CodeExecWidget;

pub(in super::super) struct CodeExecButtonParams<'a> {
    pub(in super::super) active_tab: usize,
    pub(in super::super) layout: crate::ui::code_exec_popup_layout::CodeExecPopupLayout,
    pub(in super::super) theme: &'a crate::render::RenderTheme,
    pub(in super::super) frame_layout: &'a FrameLayout,
    pub(in super::super) update: &'a UpdateOutput,
}

pub(in super::super) fn handle_code_exec_buttons(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    m: crossterm::event::MouseEvent,
    params: &CodeExecButtonParams<'_>,
) -> bool {
    if !point_in_rect(m.column, m.row, params.layout.popup) {
        return false;
    }
    let (hover, reason_target, live) = read_button_state(ctx, params.active_tab);
    let mode = resolve_code_exec_mode(reason_target, live.as_ref());
    configure_buttons(widget, params.layout, mode, hover, params.theme);
    let event = crossterm::event::Event::Mouse(m);
    handle_button_clicks(widget, ctx, &event, mode, params)
}

fn handle_button_clicks(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    event: &crossterm::event::Event,
    mode: CodeExecButtonsMode,
    params: &CodeExecButtonParams<'_>,
) -> bool {
    if handle_approve_button(widget, ctx, event, mode, params) {
        return true;
    }
    if handle_deny_button(widget, ctx, event, mode, params) {
        return true;
    }
    if handle_stop_button(widget, ctx, event, params) {
        return true;
    }
    handle_exit_button(widget, ctx, event, params)
}

fn handle_approve_button(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    event: &crossterm::event::Event,
    mode: CodeExecButtonsMode,
    params: &CodeExecButtonParams<'_>,
) -> bool {
    if !button_clicked(
        &mut widget.approve_btn,
        ctx,
        params.frame_layout,
        params.update,
        params.layout.approve_btn,
        event,
    ) {
        return false;
    }
    if handle_approve_click(ctx, params.active_tab, mode) {
        ctx.view.overlay.close();
    }
    true
}

fn handle_deny_button(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    event: &crossterm::event::Event,
    mode: CodeExecButtonsMode,
    params: &CodeExecButtonParams<'_>,
) -> bool {
    if !button_clicked(
        &mut widget.deny_btn,
        ctx,
        params.frame_layout,
        params.update,
        params.layout.deny_btn,
        event,
    ) {
        return false;
    }
    handle_deny_click(ctx, params.active_tab, mode)
}

fn handle_stop_button(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    event: &crossterm::event::Event,
    params: &CodeExecButtonParams<'_>,
) -> bool {
    if !button_clicked(
        &mut widget.stop_btn,
        ctx,
        params.frame_layout,
        params.update,
        params.layout.stop_btn,
        event,
    ) {
        return false;
    }
    handle_stop_click(ctx, params.active_tab)
}

fn handle_exit_button(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    event: &crossterm::event::Event,
    params: &CodeExecButtonParams<'_>,
) -> bool {
    if !button_clicked(
        &mut widget.exit_btn,
        ctx,
        params.frame_layout,
        params.update,
        params.layout.exit_btn,
        event,
    ) {
        return false;
    }
    handle_exit_click(ctx, params.active_tab)
}

fn read_button_state(
    ctx: &EventCtx<'_>,
    active_tab: usize,
) -> (
    Option<CodeExecHover>,
    Option<CodeExecReasonTarget>,
    Option<crate::ui::state::CodeExecLive>,
) {
    ctx.tabs
        .get(active_tab)
        .map(|tab| {
            let live = tab
                .app
                .code_exec_live
                .as_ref()
                .and_then(|live| live.lock().ok())
                .map(|live| live.clone());
            (tab.app.code_exec_hover, tab.app.code_exec_reason_target, live)
        })
        .unwrap_or((None, None, None))
}

fn button_clicked(
    button: &mut crate::ui::widget_system::widgets::button::ButtonWidget,
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    rect: ratatui::layout::Rect,
    event: &crossterm::event::Event,
) -> bool {
    button
        .event(ctx, event, layout, update, &[], rect)
        .map(|r| r.handled)
        .unwrap_or(false)
}

fn handle_approve_click(
    ctx: &mut EventCtx<'_>,
    active_tab: usize,
    mode: CodeExecButtonsMode,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
        return handle_code_exec_approve(tab_state, mode);
    }
    false
}

fn handle_deny_click(
    ctx: &mut EventCtx<'_>,
    active_tab: usize,
    mode: CodeExecButtonsMode,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
        return handle_code_exec_deny(tab_state, mode);
    }
    false
}

fn handle_stop_click(ctx: &mut EventCtx<'_>, active_tab: usize) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
        tab_state.app.code_exec_reason_target = Some(CodeExecReasonTarget::Stop);
        tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
        tab_state.app.code_exec_hover = None;
        return true;
    }
    false
}

fn handle_exit_click(ctx: &mut EventCtx<'_>, active_tab: usize) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
        tab_state.app.pending_command = Some(PendingCommand::ExitCodeExec);
        return true;
    }
    false
}

fn handle_code_exec_approve(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    mode: CodeExecButtonsMode,
) -> bool {
    if let Some(target) = mode.reason_target {
        tab_state.app.pending_command = Some(match target {
            CodeExecReasonTarget::Deny => PendingCommand::DenyCodeExec,
            CodeExecReasonTarget::Stop => PendingCommand::StopCodeExec,
        });
        return matches!(target, CodeExecReasonTarget::Deny);
    }
    if mode.finished {
        tab_state.app.pending_command = Some(PendingCommand::ExitCodeExec);
        return false;
    }
    if mode.running {
        tab_state.app.code_exec_reason_target = Some(CodeExecReasonTarget::Stop);
        tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
        tab_state.app.code_exec_hover = None;
        return false;
    }
    tab_state.app.pending_command = Some(PendingCommand::ApproveCodeExec);
    tab_state.app.code_exec_hover = None;
    true
}

fn handle_code_exec_deny(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    mode: CodeExecButtonsMode,
) -> bool {
    if let Some(_target) = mode.reason_target {
        tab_state.app.code_exec_reason_target = None;
        tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
        tab_state.app.code_exec_hover = None;
        return true;
    }
    if mode.finished || mode.running {
        return false;
    }
    tab_state.app.code_exec_reason_target = Some(CodeExecReasonTarget::Deny);
    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
    tab_state.app.code_exec_hover = None;
    true
}
