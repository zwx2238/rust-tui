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
        wait_gdb: false,
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

struct CtxParams<'a> {
    tabs: &'a mut Vec<TabState>,
    active_tab: &'a mut usize,
    categories: &'a mut Vec<String>,
    active_category: &'a mut usize,
    theme: &'a RenderTheme,
    registry: &'a ModelRegistry,
    prompt_registry: &'a PromptRegistry,
    args: &'a Args,
}

fn ctx<'a>(params: CtxParams<'a>) -> DispatchContext<'a> {
    DispatchContext {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        msg_width: 60,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
    }
}

struct OverlayTestState {
    tabs: Vec<TabState>,
    active_tab: usize,
    categories: Vec<String>,
    active_category: usize,
    theme: RenderTheme,
    registry: ModelRegistry,
    prompt_registry: PromptRegistry,
    args: Args,
    view: ViewState,
}

fn base_state(tab: TabState, overlay: crate::ui::overlay::OverlayKind) -> OverlayTestState {
    let theme = theme();
    let registry = registry();
    let prompt_registry = prompt_registry();
    let args = args();
    let mut view = ViewState::new();
    view.overlay.open(overlay);
    OverlayTestState {
        tabs: vec![tab],
        active_tab: 0,
        categories: vec!["默认".to_string()],
        active_category: 0,
        theme,
        registry,
        prompt_registry,
        args,
        view,
    }
}

fn ctx_and_view<'a>(state: &'a mut OverlayTestState) -> (DispatchContext<'a>, &'a mut ViewState) {
    let OverlayTestState {
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        args,
        view,
    } = state;
    let ctx = ctx(CtxParams {
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        args,
    });
    (ctx, view)
}

fn code_exec_tab(code: &str) -> TabState {
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    tab.app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
        call_id: "call".to_string(),
        language: "python".to_string(),
        code: code.to_string(),
        exec_code: None,
        requested_at: std::time::Instant::now(),
        stop_reason: None,
    });
    tab
}

fn code_exec_live(tab: &mut TabState) {
    tab.app.code_exec_live = Some(std::sync::Arc::new(std::sync::Mutex::new(
        crate::ui::state::CodeExecLive {
            started_at: std::time::Instant::now(),
            finished_at: None,
            stdout: "out\n".repeat(50),
            stderr: "err\n".repeat(50),
            exit_code: None,
            done: false,
        },
    )));
}

fn file_patch_tab() -> TabState {
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    tab.app.pending_file_patch = Some(crate::ui::state::PendingFilePatch {
        call_id: "call".to_string(),
        path: None,
        diff: "diff --git a/a b/a\n".to_string(),
        preview: "preview".to_string(),
    });
    tab
}

fn mouse_move_at(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Moved,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

fn mouse_click_at(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

fn mouse_scroll_down_at(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn code_exec_overlay_hover_and_click() {
    let tab = code_exec_tab("print(1)\nprint(2)\nprint(3)");
    let mut state = base_state(tab, crate::ui::overlay::OverlayKind::CodeExec);
    let (mut ctx, view) = ctx_and_view(&mut state);
    let layout = layout();
    let popup = code_exec_popup_layout(layout.size, false);
    let hover = mouse_move_at(popup.approve_btn.x, popup.approve_btn.y);
    assert!(handle_code_exec_overlay_mouse(
        hover, &mut ctx, layout, view
    ));
    assert_eq!(
        ctx.tabs[0].app.code_exec_hover,
        Some(crate::ui::state::CodeExecHover::Approve)
    );
    let click = mouse_click_at(popup.approve_btn.x, popup.approve_btn.y);
    assert!(handle_code_exec_overlay_mouse(
        click, &mut ctx, layout, view
    ));
    assert_eq!(
        ctx.tabs[0].app.pending_command,
        Some(crate::ui::state::PendingCommand::ApproveCodeExec)
    );
    assert!(view.overlay.is_chat());
}

#[test]
fn file_patch_overlay_click_apply() {
    let tab = file_patch_tab();
    let mut state = base_state(tab, crate::ui::overlay::OverlayKind::FilePatch);
    let (mut ctx, view) = ctx_and_view(&mut state);
    let layout = layout();
    let popup = file_patch_popup_layout(layout.size);
    let click = mouse_click_at(popup.apply_btn.x, popup.apply_btn.y);
    assert!(handle_file_patch_overlay_mouse(
        click, &mut ctx, layout, view
    ));
    assert_eq!(
        ctx.tabs[0].app.pending_command,
        Some(crate::ui::state::PendingCommand::ApplyFilePatch)
    );
    assert!(view.overlay.is_chat());
}

#[test]
fn code_exec_overlay_scroll_updates_offsets() {
    let mut tab = code_exec_tab(&"line\n".repeat(50));
    code_exec_live(&mut tab);
    let mut state = base_state(tab, crate::ui::overlay::OverlayKind::CodeExec);
    let (mut ctx, view) = ctx_and_view(&mut state);
    let layout = layout();
    let popup = code_exec_popup_layout(layout.size, false);
    let scroll = mouse_scroll_down_at(popup.code_text_area.x, popup.code_text_area.y);
    handle_code_exec_overlay_mouse(scroll, &mut ctx, layout, view);
    assert!(ctx.tabs[0].app.code_exec_scroll > 0);
    let scroll = mouse_scroll_down_at(popup.stdout_text_area.x, popup.stdout_text_area.y);
    handle_code_exec_overlay_mouse(scroll, &mut ctx, layout, view);
    assert!(ctx.tabs[0].app.code_exec_stdout_scroll > 0);
    let scroll = mouse_scroll_down_at(popup.stderr_text_area.x, popup.stderr_text_area.y);
    handle_code_exec_overlay_mouse(scroll, &mut ctx, layout, view);
    assert!(ctx.tabs[0].app.code_exec_stderr_scroll > 0);
}

#[test]
fn code_exec_overlay_reason_flow() {
    let mut tab = code_exec_tab("print(1)");
    tab.app.code_exec_reason_target = Some(crate::ui::state::CodeExecReasonTarget::Deny);
    let mut state = base_state(tab, crate::ui::overlay::OverlayKind::CodeExec);
    let (mut ctx, view) = ctx_and_view(&mut state);
    let layout = layout();
    let popup = code_exec_popup_layout(layout.size, true);
    let click = mouse_click_at(popup.approve_btn.x, popup.approve_btn.y);
    handle_code_exec_overlay_mouse(click, &mut ctx, layout, view);
    assert_eq!(
        ctx.tabs[0].app.pending_command,
        Some(crate::ui::state::PendingCommand::DenyCodeExec)
    );
    let click = mouse_click_at(popup.deny_btn.x, popup.deny_btn.y);
    handle_code_exec_overlay_mouse(click, &mut ctx, layout, view);
    assert!(ctx.tabs[0].app.code_exec_reason_target.is_none());
}
