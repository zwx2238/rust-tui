use crate::ui::code_exec_popup_layout::{CodeExecPopupLayout, code_exec_popup_layout};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::state::{CodeExecHover, CodeExecReasonTarget};
use crate::ui::widget_system::context::{UpdateOutput, WidgetFrame};
use crate::ui::widget_system::lifecycle::Widget;
use ratatui::style::{Modifier, Style};

use super::super::widget::CodeExecWidget;
use super::mode::{CodeExecButtonsMode, resolve_code_exec_mode};

pub(in super::super) struct CodeExecButtonsRenderParams<'a> {
    pub(in super::super) area: ratatui::layout::Rect,
    pub(in super::super) hover: Option<CodeExecHover>,
    pub(in super::super) reason_target: Option<CodeExecReasonTarget>,
    pub(in super::super) live: Option<&'a crate::ui::state::CodeExecLive>,
    pub(in super::super) theme: &'a crate::render::RenderTheme,
    pub(in super::super) layout: &'a FrameLayout,
    pub(in super::super) update: &'a UpdateOutput,
}

pub(in super::super) fn render_buttons(
    widget: &mut CodeExecWidget,
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    params: CodeExecButtonsRenderParams<'_>,
) {
    let popup = code_exec_popup_layout(params.area, params.reason_target.is_some());
    let mode = resolve_code_exec_mode(params.reason_target, params.live);
    configure_buttons(widget, popup, mode, params.hover, params.theme);
    let _ = widget
        .approve_btn
        .render(frame, params.layout, params.update, popup.approve_btn);
    let _ = widget
        .deny_btn
        .render(frame, params.layout, params.update, popup.deny_btn);
    let _ = widget
        .stop_btn
        .render(frame, params.layout, params.update, popup.stop_btn);
    let _ = widget
        .exit_btn
        .render(frame, params.layout, params.update, popup.exit_btn);
}

pub(super) fn configure_buttons(
    widget: &mut CodeExecWidget,
    layout: CodeExecPopupLayout,
    mode: CodeExecButtonsMode,
    hover: Option<CodeExecHover>,
    theme: &crate::render::RenderTheme,
) {
    widget.approve_btn.set_rect(layout.approve_btn);
    widget.deny_btn.set_rect(layout.deny_btn);
    widget.stop_btn.set_rect(layout.stop_btn);
    widget.exit_btn.set_rect(layout.exit_btn);
    widget.approve_btn.set_bordered(true);
    widget.deny_btn.set_bordered(true);
    widget.stop_btn.set_bordered(true);
    widget.exit_btn.set_bordered(true);
    if let Some(target) = mode.reason_target {
        configure_reason_buttons(widget, target, hover, theme);
        return;
    }
    if mode.finished {
        configure_finished_buttons(widget, hover, theme);
        return;
    }
    if mode.running {
        configure_running_buttons(widget, hover, theme);
        return;
    }
    configure_default_buttons(widget, hover, theme);
}

fn configure_reason_buttons(
    widget: &mut CodeExecWidget,
    target: CodeExecReasonTarget,
    hover: Option<CodeExecHover>,
    theme: &crate::render::RenderTheme,
) {
    widget.approve_btn.set_label(match target {
        CodeExecReasonTarget::Deny => "确认取消",
        CodeExecReasonTarget::Stop => "确认中止",
    });
    widget.deny_btn.set_label("返回");
    widget.approve_btn.set_visible(true);
    widget.deny_btn.set_visible(true);
    widget.stop_btn.set_visible(false);
    widget.exit_btn.set_visible(false);
    widget
        .approve_btn
        .set_style(button_style(hover, CodeExecHover::ReasonConfirm, theme));
    widget
        .deny_btn
        .set_style(button_style(hover, CodeExecHover::ReasonBack, theme));
}

fn configure_finished_buttons(
    widget: &mut CodeExecWidget,
    hover: Option<CodeExecHover>,
    theme: &crate::render::RenderTheme,
) {
    widget.exit_btn.set_visible(true);
    widget.approve_btn.set_visible(false);
    widget.deny_btn.set_visible(false);
    widget.stop_btn.set_visible(false);
    widget.exit_btn.set_label("退出");
    widget
        .exit_btn
        .set_style(button_style(hover, CodeExecHover::Exit, theme));
}

fn configure_running_buttons(
    widget: &mut CodeExecWidget,
    hover: Option<CodeExecHover>,
    theme: &crate::render::RenderTheme,
) {
    widget.stop_btn.set_visible(true);
    widget.approve_btn.set_visible(false);
    widget.deny_btn.set_visible(false);
    widget.exit_btn.set_visible(false);
    widget.stop_btn.set_label("停止执行");
    widget
        .stop_btn
        .set_style(button_style(hover, CodeExecHover::Stop, theme));
}

fn configure_default_buttons(
    widget: &mut CodeExecWidget,
    hover: Option<CodeExecHover>,
    theme: &crate::render::RenderTheme,
) {
    widget.approve_btn.set_visible(true);
    widget.deny_btn.set_visible(true);
    widget.stop_btn.set_visible(false);
    widget.exit_btn.set_visible(false);
    widget.approve_btn.set_label("确认执行");
    widget.deny_btn.set_label("取消拒绝");
    widget
        .approve_btn
        .set_style(button_style(hover, CodeExecHover::Approve, theme));
    widget
        .deny_btn
        .set_style(button_style(hover, CodeExecHover::Deny, theme));
}

fn button_style(
    hover: Option<CodeExecHover>,
    target: CodeExecHover,
    theme: &crate::render::RenderTheme,
) -> Style {
    match hover {
        Some(h) if h == target => Style::default()
            .bg(crate::ui::draw::style::selection_bg(theme.bg))
            .fg(crate::ui::draw::style::base_fg(theme))
            .add_modifier(Modifier::BOLD),
        _ => crate::ui::draw::style::base_style(theme),
    }
}
