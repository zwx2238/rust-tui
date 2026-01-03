use crate::ui::scroll::SCROLL_STEP_I32;
use crossterm::event::MouseEventKind;

#[path = "mouse_overlay_code_exec.rs"]
mod mouse_overlay_code_exec;
#[path = "mouse_overlay_file_patch.rs"]
mod mouse_overlay_file_patch;

pub(crate) use mouse_overlay_code_exec::handle_code_exec_overlay_mouse;
pub(crate) use mouse_overlay_file_patch::handle_file_patch_overlay_mouse;

fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

fn is_mouse_down(kind: MouseEventKind) -> bool {
    matches!(kind, MouseEventKind::Down(_))
}

fn is_mouse_moved(kind: MouseEventKind) -> bool {
    matches!(kind, MouseEventKind::Moved)
}

fn scroll_delta(kind: MouseEventKind) -> Option<i32> {
    match kind {
        MouseEventKind::ScrollUp => Some(-SCROLL_STEP_I32),
        MouseEventKind::ScrollDown => Some(SCROLL_STEP_I32),
        _ => None,
    }
}

fn apply_scroll(current: &mut usize, delta: i32, max: usize) {
    let next = (i32::try_from(*current).unwrap_or(0) + delta).max(0) as usize;
    *current = next.min(max);
}

#[cfg(test)]
#[path = "mouse_overlay_tests.rs"]
mod mouse_overlay_tests;
