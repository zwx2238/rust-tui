#[path = "../runtime_events_mouse_handlers/mod.rs"]
mod runtime_events_mouse_handlers;

pub(crate) use runtime_events_mouse_handlers::{MouseEventParams, handle_mouse_event};
