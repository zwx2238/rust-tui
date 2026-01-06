#[path = "interaction/clipboard.rs"]
mod clipboard;
mod code_exec_container;
mod code_exec_container_env;
#[path = "popup/code_exec_popup.rs"]
mod code_exec_popup;
#[path = "popup/code_exec_popup_layout.rs"]
mod code_exec_popup_layout;
#[path = "popup/code_exec_popup_text.rs"]
mod code_exec_popup_text;
#[path = "commands/command_input.rs"]
mod command_input;
#[path = "commands/command_suggestions.rs"]
mod command_suggestions;
#[path = "commands/commands.rs"]
mod commands;
mod draw;
mod draw_input;
#[path = "runtime_impl/events.rs"]
mod events;
#[path = "popup/file_patch_popup.rs"]
mod file_patch_popup;
#[path = "popup/file_patch_popup_layout.rs"]
mod file_patch_popup_layout;
#[path = "popup/file_patch_popup_text.rs"]
mod file_patch_popup_text;
#[path = "interaction/input.rs"]
mod input;
#[path = "interaction/input_click.rs"]
mod input_click;
#[path = "interaction/input_thread.rs"]
mod input_thread;
#[path = "interaction/jump.rs"]
mod jump;
#[path = "runtime_impl/logic.rs"]
mod logic;
#[path = "popup/model_popup.rs"]
mod model_popup;
mod net;
mod notice;
mod overlay;
mod overlay_table;
mod overlay_table_state;
#[path = "runtime_impl/perf.rs"]
mod perf;
#[path = "popup/prompt_popup.rs"]
mod prompt_popup;
mod runtime;
mod runtime_code_exec;
#[path = "runtime_impl/runtime_code_exec_helpers.rs"]
mod runtime_code_exec_helpers;
#[path = "runtime_impl/runtime_code_exec_output.rs"]
mod runtime_code_exec_output;
#[cfg(test)]
#[path = "runtime_impl/runtime_context.rs"]
mod runtime_context;
mod runtime_dispatch;
#[path = "runtime_impl/runtime_events.rs"]
mod runtime_events;
#[path = "runtime_impl/runtime_events_helpers.rs"]
mod runtime_events_helpers;
#[path = "runtime_impl/runtime_file_patch.rs"]
mod runtime_file_patch;
#[path = "runtime_impl/runtime_helpers.rs"]
mod runtime_helpers;
#[path = "runtime_impl/runtime_layout.rs"]
mod runtime_layout;
mod runtime_loop;
mod runtime_loop_helpers;
#[path = "runtime_impl/runtime_loop_steps.rs"]
mod runtime_loop_steps;
#[path = "runtime_impl/runtime_requests.rs"]
mod runtime_requests;
mod runtime_session;
mod runtime_tick;
#[path = "runtime_impl/runtime_view.rs"]
mod runtime_view;
#[path = "runtime_impl/runtime_view_handlers.rs"]
mod runtime_view_handlers;
#[path = "runtime_impl/runtime_yolo.rs"]
mod runtime_yolo;
#[path = "interaction/scroll.rs"]
mod scroll;
#[path = "interaction/scroll_debug.rs"]
mod scroll_debug;
#[path = "interaction/selection.rs"]
mod selection;
#[path = "interaction/selection_state.rs"]
mod selection_state;
#[path = "interaction/shortcut_help.rs"]
mod shortcut_help;
#[path = "interaction/shortcuts.rs"]
mod shortcuts;
#[path = "runtime_impl/state.rs"]
mod state;
mod summary;
#[path = "interaction/text_utils.rs"]
mod text_utils;
mod tool_service;
mod tools;
mod widget_system;
#[path = "runtime_impl/workspace.rs"]
pub(crate) mod workspace;

#[cfg(test)]
mod tests;

pub use runtime::run;
