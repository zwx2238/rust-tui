use crate::ui::code_exec_popup::draw_code_exec_popup_base;
use crate::ui::code_exec_popup_layout::{CodeExecPopupLayout, code_exec_popup_layout};
use crate::ui::code_exec_popup_text::{
    build_code_text, build_stderr_text, build_stdout_text, code_max_scroll, code_plain_lines,
    stderr_max_scroll, stderr_plain_lines, stdout_max_scroll, stdout_plain_lines,
};
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::selection::{Selection, chat_position_from_mouse, extract_selection};
use crate::ui::state::{
    CodeExecHover, CodeExecReasonTarget, CodeExecSelectionTarget, PendingCommand,
};
use ratatui::style::{Modifier, Style};
use std::error::Error;

use crate::ui::clipboard;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::button::ButtonWidget;
use super::overlay_table::OverlayTableController;

pub(crate) struct CodeExecWidget {
    approve_btn: ButtonWidget,
    deny_btn: ButtonWidget,
    stop_btn: ButtonWidget,
    exit_btn: ButtonWidget,
}

impl CodeExecWidget {
    pub(crate) fn new() -> Self {
        Self {
            approve_btn: ButtonWidget::new("确认执行"),
            deny_btn: ButtonWidget::new("取消拒绝"),
            stop_btn: ButtonWidget::new("停止执行"),
            exit_btn: ButtonWidget::new("退出"),
        }
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
                let active_tab = *ctx.active_tab;
                let Some(pending) = ctx
                    .tabs
                    .get(active_tab)
                    .and_then(|tab| tab.app.pending_code_exec.clone())
                else {
                    return Ok(EventResult::ignored());
                };
                let reason_target = ctx
                    .tabs
                    .get(active_tab)
                    .map(|tab| tab.app.code_exec_reason_target)
                    .unwrap_or(None);
                let popup = code_exec_popup_layout(layout.size, reason_target.is_some());
                if is_mouse_drag(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab)
                        && handle_code_exec_selection_drag(
                            tab_state,
                            &pending,
                            popup,
                            ctx.theme,
                            *m,
                        )
                    {
                        return Ok(EventResult::handled());
                    }
                }
                if is_mouse_up(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab)
                        && clear_code_exec_selection(tab_state)
                    {
                        return Ok(EventResult::handled());
                    }
                }
                if is_mouse_moved(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
                        tab_state.app.code_exec_hover =
                            hover_at(*m, popup, reason_target.is_some());
                    }
                    return Ok(EventResult::handled());
                }
                if let Some(delta) = scroll_delta(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab)
                        && handle_code_exec_scroll(
                            *m,
                            ctx.theme,
                            tab_state,
                            &pending,
                            popup,
                            delta,
                        )
                    {
                        return Ok(EventResult::handled());
                    }
                }
                if is_mouse_down(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab)
                        && handle_code_exec_selection_start(
                            tab_state,
                            &pending,
                            popup,
                            ctx.theme,
                            *m,
                        )
                    {
                        return Ok(EventResult::handled());
                    }
                    if handle_code_exec_buttons(
                        self,
                        ctx,
                        active_tab,
                        *m,
                        popup,
                        ctx.theme,
                        layout,
                        update,
                    ) {
                        return Ok(EventResult::handled());
                    }
                    if !point_in_rect(m.column, m.row, layout.layout.tabs_area)
                        && !point_in_rect(m.column, m.row, layout.layout.category_area)
                    {
                        ctx.view.overlay.close();
                        if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
                            tab_state.app.code_exec_hover = None;
                        }
                        return Ok(EventResult::handled());
                    }
                }
                Ok(EventResult::ignored())
            }
            crossterm::event::Event::Key(_) => {
                if let crossterm::event::Event::Key(key) = event
                    && is_ctrl_c(*key)
                {
                    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
                        && let Some(pending) = tab_state.app.pending_code_exec.clone()
                        && copy_code_exec_selection(tab_state, &pending, layout, ctx.theme)
                    {
                        return Ok(EventResult::ignored());
                    }
                }
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
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let active_tab = frame.state.active_tab;
        let (hover, reason_target, live_snapshot) = {
            let Some(tab_state) = frame.state.tabs.get_mut(active_tab) else {
                return Ok(());
            };
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
            let mut params = crate::ui::code_exec_popup::CodeExecPopupParams {
                area: frame.frame.area(),
                pending: &pending,
                scroll: ui_state.0,
                stdout_scroll: ui_state.1,
                stderr_scroll: ui_state.2,
                hover: ui_state.3,
                reason_target: ui_state.4,
                reason_input: &mut reason_input,
                live: live_snapshot.as_ref(),
                code_selection: tab_state.app.code_exec_code_selection,
                stdout_selection: tab_state.app.code_exec_stdout_selection,
                stderr_selection: tab_state.app.code_exec_stderr_selection,
                theme: frame.state.theme,
            };
            draw_code_exec_popup_base(frame.frame, &mut params);
            tab_state.app.code_exec_reason_input = reason_input;
            (ui_state.3, ui_state.4, live_snapshot)
        };
        render_buttons(
            self,
            frame.frame.area(),
            hover,
            reason_target,
            live_snapshot.as_ref(),
            frame.state.theme,
            frame,
            layout,
            update,
        );
        Ok(())
    }
}

fn render_buttons(
    widget: &mut CodeExecWidget,
    area: ratatui::layout::Rect,
    hover: Option<CodeExecHover>,
    reason_target: Option<CodeExecReasonTarget>,
    live: Option<&crate::ui::state::CodeExecLive>,
    theme: &crate::render::RenderTheme,
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
) {
    let popup = code_exec_popup_layout(area, reason_target.is_some());
    let mode = resolve_code_exec_mode(reason_target, live);
    configure_buttons(widget, popup, mode, hover, theme);
    let _ = widget.approve_btn.render(frame, layout, update, popup.approve_btn);
    let _ = widget.deny_btn.render(frame, layout, update, popup.deny_btn);
    let _ = widget.stop_btn.render(frame, layout, update, popup.stop_btn);
    let _ = widget.exit_btn.render(frame, layout, update, popup.exit_btn);
}

fn configure_buttons(
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
        return;
    }
    if mode.finished {
        widget.exit_btn.set_visible(true);
        widget.approve_btn.set_visible(false);
        widget.deny_btn.set_visible(false);
        widget.stop_btn.set_visible(false);
        widget.exit_btn.set_label("退出");
        widget
            .exit_btn
            .set_style(button_style(hover, CodeExecHover::Exit, theme));
        return;
    }
    if mode.running {
        widget.stop_btn.set_visible(true);
        widget.approve_btn.set_visible(false);
        widget.deny_btn.set_visible(false);
        widget.exit_btn.set_visible(false);
        widget.stop_btn.set_label("停止执行");
        widget
            .stop_btn
            .set_style(button_style(hover, CodeExecHover::Stop, theme));
        return;
    }
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

fn handle_code_exec_buttons(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    active_tab: usize,
    m: crossterm::event::MouseEvent,
    layout: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    frame_layout: &FrameLayout,
    update: &UpdateOutput,
) -> bool {
    if !point_in_rect(m.column, m.row, layout.popup) {
        return false;
    }
    let (hover, reason_target, live) = ctx
        .tabs
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
        .unwrap_or((None, None, None));
    let mode = resolve_code_exec_mode(reason_target, live.as_ref());
    configure_buttons(widget, layout, mode, hover, theme);
    let event = crossterm::event::Event::Mouse(m);
    if widget
        .approve_btn
        .event(ctx, &event, frame_layout, update, &[], layout.approve_btn)
        .map(|r| r.handled)
        .unwrap_or(false)
    {
        let mut should_close = false;
        if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
            should_close = handle_code_exec_approve(tab_state, mode);
        }
        if should_close {
            ctx.view.overlay.close();
        }
        return true;
    }
    if widget
        .deny_btn
        .event(ctx, &event, frame_layout, update, &[], layout.deny_btn)
        .map(|r| r.handled)
        .unwrap_or(false)
    {
        if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
            return handle_code_exec_deny(tab_state, mode);
        }
        return false;
    }
    if widget
        .stop_btn
        .event(ctx, &event, frame_layout, update, &[], layout.stop_btn)
        .map(|r| r.handled)
        .unwrap_or(false)
    {
        if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
            tab_state.app.code_exec_reason_target = Some(CodeExecReasonTarget::Stop);
            tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
            tab_state.app.code_exec_hover = None;
            return true;
        }
    }
    if widget
        .exit_btn
        .event(ctx, &event, frame_layout, update, &[], layout.exit_btn)
        .map(|r| r.handled)
        .unwrap_or(false)
    {
        if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
            tab_state.app.pending_command = Some(PendingCommand::ExitCodeExec);
            return true;
        }
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
    if let Some(target) = mode.reason_target {
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

fn handle_code_exec_scroll(
    m: crossterm::event::MouseEvent,
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    delta: i32,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    if point_in_rect(m.column, m.row, popup.code_text_area) {
        let max_scroll = code_max_scroll(
            &pending.code,
            popup.code_text_area.width,
            popup.code_text_area.height,
            theme,
        );
        apply_scroll(&mut tab_state.app.code_exec_scroll, delta, max_scroll);
        return true;
    }
    let (stdout, stderr) = code_exec_output(tab_state);
    if point_in_rect(m.column, m.row, popup.stdout_text_area) {
        let max_scroll = stdout_max_scroll(
            &stdout,
            popup.stdout_text_area.width,
            popup.stdout_text_area.height,
            theme,
        );
        apply_scroll(&mut tab_state.app.code_exec_stdout_scroll, delta, max_scroll);
        return true;
    }
    if point_in_rect(m.column, m.row, popup.stderr_text_area) {
        let max_scroll = stderr_max_scroll(
            &stderr,
            popup.stderr_text_area.width,
            popup.stderr_text_area.height,
            theme,
        );
        apply_scroll(&mut tab_state.app.code_exec_stderr_scroll, delta, max_scroll);
        return true;
    }
    false
}

fn handle_code_exec_selection_start(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    if point_in_rect(m.column, m.row, popup.code_text_area) {
        let (text, _) = build_code_text(
            &pending.code,
            popup.code_text_area.width,
            popup.code_text_area.height,
            tab_state.app.code_exec_scroll,
            theme,
        );
        let pos = selection_position_for_panel(
            &text,
            tab_state.app.code_exec_scroll,
            popup.code_text_area,
            m,
        );
        start_code_exec_selection(tab_state, CodeExecSelectionTarget::Code, pos);
        return true;
    }
    let (stdout, stderr) = code_exec_output(tab_state);
    if point_in_rect(m.column, m.row, popup.stdout_text_area) {
        let (text, _) = build_stdout_text(
            Some(&stdout),
            popup.stdout_text_area.width,
            popup.stdout_text_area.height,
            tab_state.app.code_exec_stdout_scroll,
            theme,
        );
        let pos = selection_position_for_panel(
            &text,
            tab_state.app.code_exec_stdout_scroll,
            popup.stdout_text_area,
            m,
        );
        start_code_exec_selection(tab_state, CodeExecSelectionTarget::Stdout, pos);
        return true;
    }
    if point_in_rect(m.column, m.row, popup.stderr_text_area) {
        let (text, _) = build_stderr_text(
            Some(&stderr),
            popup.stderr_text_area.width,
            popup.stderr_text_area.height,
            tab_state.app.code_exec_stderr_scroll,
            theme,
        );
        let pos = selection_position_for_panel(
            &text,
            tab_state.app.code_exec_stderr_scroll,
            popup.stderr_text_area,
            m,
        );
        start_code_exec_selection(tab_state, CodeExecSelectionTarget::Stderr, pos);
        return true;
    }
    false
}

fn handle_code_exec_selection_drag(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    let Some(target) = tab_state.app.code_exec_selecting else {
        return false;
    };
    let (stdout, stderr) = code_exec_output(tab_state);
    let (pos, sel) = match target {
        CodeExecSelectionTarget::Code => {
            let (text, _) = build_code_text(
                &pending.code,
                popup.code_text_area.width,
                popup.code_text_area.height,
                tab_state.app.code_exec_scroll,
                theme,
            );
            (
                selection_position_for_panel(
                    &text,
                    tab_state.app.code_exec_scroll,
                    popup.code_text_area,
                    m,
                ),
                &mut tab_state.app.code_exec_code_selection,
            )
        }
        CodeExecSelectionTarget::Stdout => {
            let (text, _) = build_stdout_text(
                Some(&stdout),
                popup.stdout_text_area.width,
                popup.stdout_text_area.height,
                tab_state.app.code_exec_stdout_scroll,
                theme,
            );
            (
                selection_position_for_panel(
                    &text,
                    tab_state.app.code_exec_stdout_scroll,
                    popup.stdout_text_area,
                    m,
                ),
                &mut tab_state.app.code_exec_stdout_selection,
            )
        }
        CodeExecSelectionTarget::Stderr => {
            let (text, _) = build_stderr_text(
                Some(&stderr),
                popup.stderr_text_area.width,
                popup.stderr_text_area.height,
                tab_state.app.code_exec_stderr_scroll,
                theme,
            );
            (
                selection_position_for_panel(
                    &text,
                    tab_state.app.code_exec_stderr_scroll,
                    popup.stderr_text_area,
                    m,
                ),
                &mut tab_state.app.code_exec_stderr_selection,
            )
        }
    };
    let next = match *sel {
        Some(existing) => Selection {
            start: existing.start,
            end: pos,
        },
        None => Selection { start: pos, end: pos },
    };
    *sel = Some(next);
    true
}

fn clear_code_exec_selection(tab_state: &mut crate::ui::runtime_helpers::TabState) -> bool {
    if tab_state.app.code_exec_selecting.is_none() {
        return false;
    }
    tab_state.app.code_exec_selecting = None;
    true
}

fn start_code_exec_selection(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    target: CodeExecSelectionTarget,
    pos: (usize, usize),
) {
    tab_state.app.code_exec_selecting = Some(target);
    let selection = Some(Selection { start: pos, end: pos });
    match target {
        CodeExecSelectionTarget::Code => {
            tab_state.app.code_exec_code_selection = selection;
            tab_state.app.code_exec_stdout_selection = None;
            tab_state.app.code_exec_stderr_selection = None;
        }
        CodeExecSelectionTarget::Stdout => {
            tab_state.app.code_exec_stdout_selection = selection;
            tab_state.app.code_exec_code_selection = None;
            tab_state.app.code_exec_stderr_selection = None;
        }
        CodeExecSelectionTarget::Stderr => {
            tab_state.app.code_exec_stderr_selection = selection;
            tab_state.app.code_exec_code_selection = None;
            tab_state.app.code_exec_stdout_selection = None;
        }
    }
}

fn selection_position_for_panel(
    text: &ratatui::text::Text<'static>,
    scroll: usize,
    area: ratatui::layout::Rect,
    m: crossterm::event::MouseEvent,
) -> (usize, usize) {
    let scroll_u16 = scroll.min(u16::MAX as usize) as u16;
    chat_position_from_mouse(text, scroll_u16, area, m.column, m.row)
}

fn code_exec_output(
    tab_state: &crate::ui::runtime_helpers::TabState,
) -> (String, String) {
    tab_state
        .app
        .code_exec_live
        .as_ref()
        .and_then(|live| {
            live.lock()
                .ok()
                .map(|live| (live.stdout.clone(), live.stderr.clone()))
        })
        .unwrap_or_else(|| (String::new(), String::new()))
}

fn resolve_code_exec_mode(
    reason_target: Option<CodeExecReasonTarget>,
    live: Option<&crate::ui::state::CodeExecLive>,
) -> CodeExecButtonsMode {
    let finished = live
        .map(|l| l.done || l.exit_code.is_some())
        .unwrap_or(false);
    let running = live.is_some() && !finished;
    CodeExecButtonsMode {
        reason_target,
        running,
        finished,
    }
}

#[derive(Copy, Clone)]
struct CodeExecButtonsMode {
    reason_target: Option<CodeExecReasonTarget>,
    running: bool,
    finished: bool,
}

fn hover_at(
    m: crossterm::event::MouseEvent,
    popup: CodeExecPopupLayout,
    reason_mode: bool,
) -> Option<CodeExecHover> {
    if point_in_rect(m.column, m.row, popup.approve_btn) {
        Some(if reason_mode {
            CodeExecHover::ReasonConfirm
        } else {
            CodeExecHover::Approve
        })
    } else if point_in_rect(m.column, m.row, popup.deny_btn) {
        Some(if reason_mode {
            CodeExecHover::ReasonBack
        } else {
            CodeExecHover::Deny
        })
    } else if point_in_rect(m.column, m.row, popup.stop_btn) {
        Some(CodeExecHover::Stop)
    } else if point_in_rect(m.column, m.row, popup.exit_btn) {
        Some(CodeExecHover::Exit)
    } else {
        None
    }
}

fn button_style(
    hover: Option<CodeExecHover>,
    target: CodeExecHover,
    theme: &crate::render::RenderTheme,
) -> Style {
    match hover {
        Some(h) if h == target => Style::default()
            .bg(selection_bg(theme.bg))
            .fg(base_fg(theme))
            .add_modifier(Modifier::BOLD),
        _ => base_style(theme),
    }
}

fn prepare_code_exec_overlay(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    layout: CodeExecPopupLayout,
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
    layout: CodeExecPopupLayout,
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
    layout: CodeExecPopupLayout,
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

fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

fn is_mouse_down(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Down(_))
}

fn is_ctrl_c(key: crossterm::event::KeyEvent) -> bool {
    key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
        && key.code == crossterm::event::KeyCode::Char('c')
}

fn is_mouse_up(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Up(_))
}

fn is_mouse_moved(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Moved)
}

fn is_mouse_drag(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Drag(_))
}

fn scroll_delta(kind: crossterm::event::MouseEventKind) -> Option<i32> {
    match kind {
        crossterm::event::MouseEventKind::ScrollUp => Some(-crate::ui::scroll::SCROLL_STEP_I32),
        crossterm::event::MouseEventKind::ScrollDown => Some(crate::ui::scroll::SCROLL_STEP_I32),
        _ => None,
    }
}

fn apply_scroll(current: &mut usize, delta: i32, max: usize) {
    let next = (i32::try_from(*current).unwrap_or(0) + delta).max(0) as usize;
    *current = next.min(max);
}

fn copy_code_exec_selection(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    layout: &FrameLayout,
    theme: &crate::render::RenderTheme,
) -> bool {
    let popup = code_exec_popup_layout(
        layout.size,
        tab_state.app.code_exec_reason_target.is_some(),
    );
    if let Some(selection) = tab_state.app.code_exec_code_selection {
        let lines = code_plain_lines(&pending.code, popup.code_text_area.width, theme);
        return copy_selection_text(lines, selection);
    }
    let (stdout, stderr) = code_exec_output(tab_state);
    if let Some(selection) = tab_state.app.code_exec_stdout_selection {
        let lines = stdout_plain_lines(&stdout, popup.stdout_text_area.width, theme);
        return copy_selection_text(lines, selection);
    }
    if let Some(selection) = tab_state.app.code_exec_stderr_selection {
        let lines = stderr_plain_lines(&stderr, popup.stderr_text_area.width, theme);
        return copy_selection_text(lines, selection);
    }
    false
}

fn copy_selection_text(lines: Vec<String>, selection: Selection) -> bool {
    let text = extract_selection(&lines, selection);
    if !text.is_empty() {
        clipboard::set(&text);
    }
    true
}
