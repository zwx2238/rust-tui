pub(crate) mod draw;
pub(crate) mod notice;
pub(crate) mod overlay;
pub(crate) mod overlay_table;
pub(crate) mod overlay_table_state;
mod runtime;
pub(crate) mod runtime_dispatch;
mod runtime_loop;
pub(crate) mod runtime_loop_helpers;
mod runtime_session;
pub(crate) mod runtime_tick;
pub(crate) mod summary;
pub(crate) mod tab_bar;
pub(crate) mod terminal;

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
pub(crate) use runtime_impl::{
    events, logic, perf, runtime_helpers, runtime_loop_steps, runtime_view,
    runtime_view_handlers, state,
};

pub use runtime::run;
