mod code_exec_container;
mod code_exec_container_env;
pub(crate) mod draw;
mod net;
pub(crate) mod notice;
pub(crate) mod overlay;
pub(crate) mod overlay_table;
pub(crate) mod overlay_table_state;
mod runtime;
mod runtime_code_exec;
pub(crate) mod runtime_dispatch;
pub(crate) mod runtime_events_mouse_handlers;
mod runtime_loop;
pub(crate) mod runtime_loop_helpers;
mod runtime_session;
pub(crate) mod runtime_tick;
pub(crate) mod summary;
pub(crate) mod tab_bar;
pub(crate) mod terminal;
mod tool_service;
mod tools;

mod commands;
mod interaction;
mod popup;
mod runtime_impl;

pub(crate) use commands::{command_input, command_suggestions};
pub(crate) use interaction::{
    clipboard, input, input_click, input_thread, jump, scroll, scroll_debug, selection,
    selection_state, shortcut_help, shortcuts, text_utils,
};
pub(crate) use popup::{
    code_exec_popup_layout, code_exec_popup_text, file_patch_popup_layout, file_patch_popup_text,
    model_popup, prompt_popup, question_review_popup, terminal_popup_layout,
};
pub(crate) use runtime_impl::workspace;
pub(crate) use runtime_impl::{
    events, logic, perf, runtime_code_exec_helpers, runtime_code_exec_output, runtime_events,
    runtime_events_helpers, runtime_file_patch, runtime_helpers, runtime_layout,
    runtime_loop_steps, runtime_question_review, runtime_requests, runtime_view,
    runtime_view_handlers, runtime_yolo, state,
};

pub use runtime::run;
