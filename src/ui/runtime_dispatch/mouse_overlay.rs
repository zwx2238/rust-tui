use crate::ui::code_exec_popup_layout::code_exec_popup_layout;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::file_patch_popup_layout::file_patch_popup_layout;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};
use crate::ui::runtime_view::ViewState;
use crate::ui::scroll::SCROLL_STEP_I32;
use crossterm::event::{MouseEvent, MouseEventKind};

pub(crate) fn handle_code_exec_overlay_mouse(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if let Some(pending) = tab_state.app.pending_code_exec.clone() {
            let popup = code_exec_popup_layout(
                layout.size,
                tab_state.app.code_exec_reason_target.is_some(),
            );
            let in_popup = point_in_rect(m.column, m.row, popup.popup);
            if matches!(m.kind, MouseEventKind::Moved) {
                let reason_mode = tab_state.app.code_exec_reason_target.is_some();
                tab_state.app.code_exec_hover = if point_in_rect(m.column, m.row, popup.approve_btn)
                {
                    Some(if reason_mode {
                        crate::ui::state::CodeExecHover::ReasonConfirm
                    } else {
                        crate::ui::state::CodeExecHover::Approve
                    })
                } else if point_in_rect(m.column, m.row, popup.deny_btn) {
                    Some(if reason_mode {
                        crate::ui::state::CodeExecHover::ReasonBack
                    } else {
                        crate::ui::state::CodeExecHover::Deny
                    })
                } else if point_in_rect(m.column, m.row, popup.stop_btn) {
                    Some(crate::ui::state::CodeExecHover::Stop)
                } else if point_in_rect(m.column, m.row, popup.exit_btn) {
                    Some(crate::ui::state::CodeExecHover::Exit)
                } else {
                    None
                };
                return true;
            }
            if in_popup
                && matches!(
                    m.kind,
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                )
            {
                let delta = match m.kind {
                    MouseEventKind::ScrollUp => -SCROLL_STEP_I32,
                    MouseEventKind::ScrollDown => SCROLL_STEP_I32,
                    _ => 0,
                };
                if point_in_rect(m.column, m.row, popup.code_text_area) {
                    let max_scroll = code_max_scroll(
                        &pending.code,
                        popup.code_text_area.width,
                        popup.code_text_area.height,
                        ctx.theme,
                    );
                    let next = (tab_state.app.code_exec_scroll as i32 + delta).max(0) as usize;
                    tab_state.app.code_exec_scroll = next.min(max_scroll);
                    return true;
                }
                let (stdout, stderr) = tab_state
                    .app
                    .code_exec_live
                    .as_ref()
                    .and_then(|l| l.lock().ok().map(|l| (l.stdout.clone(), l.stderr.clone())))
                    .unwrap_or_else(|| (String::new(), String::new()));
                if point_in_rect(m.column, m.row, popup.stdout_text_area) {
                    let max_scroll = stdout_max_scroll(
                        &stdout,
                        popup.stdout_text_area.width,
                        popup.stdout_text_area.height,
                        ctx.theme,
                    );
                    let next =
                        (tab_state.app.code_exec_stdout_scroll as i32 + delta).max(0) as usize;
                    tab_state.app.code_exec_stdout_scroll = next.min(max_scroll);
                    return true;
                }
                if point_in_rect(m.column, m.row, popup.stderr_text_area) {
                    let max_scroll = stderr_max_scroll(
                        &stderr,
                        popup.stderr_text_area.width,
                        popup.stderr_text_area.height,
                        ctx.theme,
                    );
                    let next =
                        (tab_state.app.code_exec_stderr_scroll as i32 + delta).max(0) as usize;
                    tab_state.app.code_exec_stderr_scroll = next.min(max_scroll);
                    return true;
                }
            }
            if in_popup && matches!(m.kind, MouseEventKind::Down(_)) {
                let finished = tab_state
                    .app
                    .code_exec_live
                    .as_ref()
                    .and_then(|l| l.lock().ok().map(|l| l.done || l.exit_code.is_some()))
                    .unwrap_or(false);
                let running = tab_state.app.code_exec_live.is_some() && !finished;
                let reason_target = tab_state.app.code_exec_reason_target;
                if let Some(target) = reason_target {
                    if point_in_rect(m.column, m.row, popup.approve_btn) {
                        tab_state.app.pending_command = Some(match target {
                            crate::ui::state::CodeExecReasonTarget::Deny => {
                                crate::ui::state::PendingCommand::DenyCodeExec
                            }
                            crate::ui::state::CodeExecReasonTarget::Stop => {
                                crate::ui::state::PendingCommand::StopCodeExec
                            }
                        });
                        if matches!(target, crate::ui::state::CodeExecReasonTarget::Deny) {
                            view.overlay.close();
                        }
                        return true;
                    }
                    if point_in_rect(m.column, m.row, popup.deny_btn) {
                        tab_state.app.code_exec_reason_target = None;
                        tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
                        tab_state.app.code_exec_hover = None;
                        return true;
                    }
                }
                if finished && point_in_rect(m.column, m.row, popup.exit_btn) {
                    tab_state.app.pending_command =
                        Some(crate::ui::state::PendingCommand::ExitCodeExec);
                    return true;
                }
                if running && point_in_rect(m.column, m.row, popup.stop_btn) {
                    tab_state.app.code_exec_reason_target =
                        Some(crate::ui::state::CodeExecReasonTarget::Stop);
                    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
                    tab_state.app.code_exec_hover = None;
                    return true;
                }
                if point_in_rect(m.column, m.row, popup.approve_btn) {
                    tab_state.app.pending_command =
                        Some(crate::ui::state::PendingCommand::ApproveCodeExec);
                    tab_state.app.code_exec_hover = None;
                    view.overlay.close();
                    return true;
                }
                if point_in_rect(m.column, m.row, popup.deny_btn) {
                    tab_state.app.code_exec_reason_target =
                        Some(crate::ui::state::CodeExecReasonTarget::Deny);
                    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
                    tab_state.app.code_exec_hover = None;
                    return true;
                }
            }
        }
    }
    if matches!(m.kind, MouseEventKind::Down(_)) {
        if point_in_rect(m.column, m.row, layout.tabs_area)
            || point_in_rect(m.column, m.row, layout.category_area)
        {
            return false;
        }
        view.overlay.close();
    }
    if matches!(m.kind, MouseEventKind::Moved) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            tab_state.app.code_exec_hover = None;
        }
    }
    true
}

pub(crate) fn handle_file_patch_overlay_mouse(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if let Some(pending) = tab_state.app.pending_file_patch.clone() {
            let popup = file_patch_popup_layout(layout.size);
            let in_popup = point_in_rect(m.column, m.row, popup.popup);
            if matches!(m.kind, MouseEventKind::Moved) {
                tab_state.app.file_patch_hover = if point_in_rect(m.column, m.row, popup.apply_btn)
                {
                    Some(crate::ui::state::FilePatchHover::Apply)
                } else if point_in_rect(m.column, m.row, popup.cancel_btn) {
                    Some(crate::ui::state::FilePatchHover::Cancel)
                } else {
                    None
                };
                return true;
            }
            if in_popup
                && matches!(
                    m.kind,
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                )
            {
                let delta = match m.kind {
                    MouseEventKind::ScrollUp => -SCROLL_STEP_I32,
                    MouseEventKind::ScrollDown => SCROLL_STEP_I32,
                    _ => 0,
                };
                let max_scroll = patch_max_scroll(
                    &pending.preview,
                    popup.preview_area.width,
                    popup.preview_area.height,
                    ctx.theme,
                );
                let next = (tab_state.app.file_patch_scroll as i32 + delta).max(0) as usize;
                tab_state.app.file_patch_scroll = next.min(max_scroll);
                return true;
            }
            if in_popup && matches!(m.kind, MouseEventKind::Down(_)) {
                if point_in_rect(m.column, m.row, popup.apply_btn) {
                    tab_state.app.pending_command =
                        Some(crate::ui::state::PendingCommand::ApplyFilePatch);
                    tab_state.app.file_patch_hover = None;
                    view.overlay.close();
                    return true;
                }
                if point_in_rect(m.column, m.row, popup.cancel_btn) {
                    tab_state.app.pending_command =
                        Some(crate::ui::state::PendingCommand::CancelFilePatch);
                    tab_state.app.file_patch_hover = None;
                    view.overlay.close();
                    return true;
                }
            }
        }
    }
    if matches!(m.kind, MouseEventKind::Down(_)) {
        if point_in_rect(m.column, m.row, layout.tabs_area)
            || point_in_rect(m.column, m.row, layout.category_area)
        {
            return false;
        }
        view.overlay.close();
    }
    if matches!(m.kind, MouseEventKind::Moved) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            tab_state.app.file_patch_hover = None;
        }
    }
    true
}

fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use super::{handle_code_exec_overlay_mouse, handle_file_patch_overlay_mouse};
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::ui::code_exec_popup_layout::code_exec_popup_layout;
    use crate::ui::file_patch_popup_layout::file_patch_popup_layout;
    use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_view::ViewState;
    use crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    fn registry() -> ModelRegistry {
        ModelRegistry {
            default_key: "m1".to_string(),
            models: vec![ModelProfile {
                key: "m1".to_string(),
                base_url: "http://example.com".to_string(),
                api_key: "k".to_string(),
                model: "model".to_string(),
            }],
        }
    }

    fn prompt_registry() -> PromptRegistry {
        PromptRegistry {
            default_key: "p1".to_string(),
            prompts: vec![SystemPrompt {
                key: "p1".to_string(),
                content: "sys1".to_string(),
            }],
        }
    }

    fn args() -> Args {
        Args {
            model: "m".to_string(),
            system: "sys".to_string(),
            base_url: "http://example.com".to_string(),
            show_reasoning: false,
            config: None,
            resume: None,
            replay_fork_last: false,
            enable: None,
            log_requests: None,
            perf: false,
            question_set: None,
            yolo: false,
            read_only: false,
        }
    }

    fn layout() -> LayoutContext {
        LayoutContext {
            size: Rect::new(0, 0, 120, 50),
            tabs_area: Rect::new(0, 1, 120, 1),
            msg_area: Rect::new(0, 2, 120, 40),
            input_area: Rect::new(0, 42, 120, 5),
            category_area: Rect::new(0, 1, 10, 5),
            view_height: 20,
            total_lines: 0,
        }
    }

    fn ctx<'a>(
        tabs: &'a mut Vec<TabState>,
        active_tab: &'a mut usize,
        categories: &'a mut Vec<String>,
        active_category: &'a mut usize,
        theme: &'a RenderTheme,
        registry: &'a ModelRegistry,
        prompt_registry: &'a PromptRegistry,
        args: &'a Args,
    ) -> DispatchContext<'a> {
        DispatchContext {
            tabs,
            active_tab,
            categories,
            active_category,
            msg_width: 60,
            theme,
            registry,
            prompt_registry,
            args,
        }
    }

    #[test]
    fn code_exec_overlay_hover_and_click() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "print(1)\nprint(2)\nprint(3)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::CodeExec);
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let layout = layout();
        let popup = code_exec_popup_layout(layout.size, false);
        let hover = MouseEvent {
            kind: MouseEventKind::Moved,
            column: popup.approve_btn.x,
            row: popup.approve_btn.y,
            modifiers: KeyModifiers::NONE,
        };
        assert!(handle_code_exec_overlay_mouse(hover, &mut ctx, layout, &mut view));
        assert_eq!(
            ctx.tabs[0].app.code_exec_hover,
            Some(crate::ui::state::CodeExecHover::Approve)
        );
        let click = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: popup.approve_btn.x,
            row: popup.approve_btn.y,
            modifiers: KeyModifiers::NONE,
        };
        assert!(handle_code_exec_overlay_mouse(click, &mut ctx, layout, &mut view));
        assert_eq!(
            ctx.tabs[0].app.pending_command,
            Some(crate::ui::state::PendingCommand::ApproveCodeExec)
        );
        assert!(view.overlay.is_chat());
    }

    #[test]
    fn file_patch_overlay_click_apply() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.pending_file_patch = Some(crate::ui::state::PendingFilePatch {
            call_id: "call".to_string(),
            path: None,
            diff: "diff --git a/a b/a\n".to_string(),
            preview: "preview".to_string(),
        });
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::FilePatch);
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let layout = layout();
        let popup = file_patch_popup_layout(layout.size);
        let click = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: popup.apply_btn.x,
            row: popup.apply_btn.y,
            modifiers: KeyModifiers::NONE,
        };
        assert!(handle_file_patch_overlay_mouse(click, &mut ctx, layout, &mut view));
        assert_eq!(
            ctx.tabs[0].app.pending_command,
            Some(crate::ui::state::PendingCommand::ApplyFilePatch)
        );
        assert!(view.overlay.is_chat());
    }

    #[test]
    fn code_exec_overlay_scroll_updates_offsets() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "line\n".repeat(50),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        tabs[0].app.code_exec_live = Some(std::sync::Arc::new(std::sync::Mutex::new(
            crate::ui::state::CodeExecLive {
                started_at: std::time::Instant::now(),
                finished_at: None,
                stdout: "out\n".repeat(50),
                stderr: "err\n".repeat(50),
                exit_code: None,
                done: false,
            },
        )));
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::CodeExec);
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let layout = layout();
        let popup = code_exec_popup_layout(layout.size, false);
        let scroll = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: popup.code_text_area.x,
            row: popup.code_text_area.y,
            modifiers: KeyModifiers::NONE,
        };
        handle_code_exec_overlay_mouse(scroll, &mut ctx, layout, &mut view);
        assert!(ctx.tabs[0].app.code_exec_scroll > 0);
        let scroll = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: popup.stdout_text_area.x,
            row: popup.stdout_text_area.y,
            modifiers: KeyModifiers::NONE,
        };
        handle_code_exec_overlay_mouse(scroll, &mut ctx, layout, &mut view);
        assert!(ctx.tabs[0].app.code_exec_stdout_scroll > 0);
        let scroll = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: popup.stderr_text_area.x,
            row: popup.stderr_text_area.y,
            modifiers: KeyModifiers::NONE,
        };
        handle_code_exec_overlay_mouse(scroll, &mut ctx, layout, &mut view);
        assert!(ctx.tabs[0].app.code_exec_stderr_scroll > 0);
    }

    #[test]
    fn code_exec_overlay_reason_flow() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        tabs[0].app.code_exec_reason_target =
            Some(crate::ui::state::CodeExecReasonTarget::Deny);
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::CodeExec);
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let layout = layout();
        let popup = code_exec_popup_layout(layout.size, true);
        let click = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: popup.approve_btn.x,
            row: popup.approve_btn.y,
            modifiers: KeyModifiers::NONE,
        };
        handle_code_exec_overlay_mouse(click, &mut ctx, layout, &mut view);
        assert_eq!(
            ctx.tabs[0].app.pending_command,
            Some(crate::ui::state::PendingCommand::DenyCodeExec)
        );
        let click = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: popup.deny_btn.x,
            row: popup.deny_btn.y,
            modifiers: KeyModifiers::NONE,
        };
        handle_code_exec_overlay_mouse(click, &mut ctx, layout, &mut view);
        assert!(ctx.tabs[0].app.code_exec_reason_target.is_none());
    }
}
