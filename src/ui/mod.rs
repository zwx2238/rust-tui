mod code_exec_container;
mod code_exec_container_env;
mod draw;
mod draw_input;
mod net;
mod notice;
mod overlay;
mod overlay_table;
mod overlay_table_state;
mod runtime;
mod runtime_code_exec;
mod runtime_dispatch;
mod runtime_events_mouse_handlers;
mod runtime_loop;
mod runtime_loop_helpers;
mod runtime_session;
mod runtime_tick;
mod summary;
mod tab_bar;
mod terminal;
mod tool_service;
mod tools;
mod widget_system;

mod commands;
mod interaction;
mod popup;
mod runtime_impl;

use commands::{command_input, command_suggestions};
use interaction::{
    clipboard, input, input_click, input_thread, jump, scroll, scroll_debug, selection,
    selection_state, shortcut_help, shortcuts, text_utils,
};
use popup::{
    code_exec_popup, code_exec_popup_layout, code_exec_popup_text, file_patch_popup,
    file_patch_popup_layout, file_patch_popup_text, model_popup, prompt_popup,
    question_review_popup, terminal_popup_layout,
};
pub(crate) use runtime_impl::workspace;
use runtime_impl::{
    events, logic, perf, runtime_code_exec_helpers, runtime_code_exec_output, runtime_events,
    runtime_events_helpers, runtime_file_patch, runtime_helpers, runtime_layout,
    runtime_loop_steps, runtime_question_review, runtime_requests, runtime_view,
    runtime_view_handlers, runtime_yolo, state,
};

pub use runtime::run;
