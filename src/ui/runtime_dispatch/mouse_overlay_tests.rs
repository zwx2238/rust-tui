use super::{handle_code_exec_overlay_mouse, handle_file_patch_overlay_mouse};
use crate::ui::code_exec_popup_layout::code_exec_popup_layout;
use crate::ui::file_patch_popup_layout::file_patch_popup_layout;

mod mouse_overlay_test_support;
use mouse_overlay_test_support::{
    base_state, code_exec_live, code_exec_tab, ctx_and_view, file_patch_tab, layout,
    mouse_click_at, mouse_move_at, mouse_scroll_down_at,
};

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
