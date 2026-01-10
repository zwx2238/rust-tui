pub(super) fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

pub(super) fn is_mouse_down(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Down(_))
}

pub(super) fn is_mouse_up(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Up(_))
}

pub(super) fn is_mouse_moved(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Moved)
}

pub(super) fn is_mouse_drag(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Drag(_))
}

pub(super) fn scroll_delta(kind: crossterm::event::MouseEventKind) -> Option<i32> {
    match kind {
        crossterm::event::MouseEventKind::ScrollUp => Some(-crate::ui::scroll::SCROLL_STEP_I32),
        crossterm::event::MouseEventKind::ScrollDown => Some(crate::ui::scroll::SCROLL_STEP_I32),
        _ => None,
    }
}

pub(super) fn apply_scroll(current: &mut usize, delta: i32, max: usize) {
    let next = (i32::try_from(*current).unwrap_or(0) + delta).max(0) as usize;
    *current = next.min(max);
}

pub(super) fn is_ctrl_c(key: crossterm::event::KeyEvent) -> bool {
    key.modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL)
        && key.code == crossterm::event::KeyCode::Char('c')
}

pub(super) fn code_exec_output(
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

pub(super) fn snapshot_live(
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
