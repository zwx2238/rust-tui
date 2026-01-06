mod clipboard;
mod code_exec_container;
mod code_exec_container_env;
#[cfg(test)]
mod code_exec_container_tests;
mod code_exec_popup;
mod code_exec_popup_layout;
#[cfg(test)]
mod code_exec_popup_layout_tests;
mod code_exec_popup_text;
#[cfg(test)]
mod code_exec_popup_text_tests;
mod command_input;
#[cfg(test)]
mod command_input_tests;
mod command_suggestions;
#[cfg(test)]
mod command_suggestions_tests;
mod commands;
#[cfg(test)]
mod commands_tests;
mod draw;
mod draw_input;
#[cfg(test)]
mod draw_tests;
mod file_patch_popup;
mod file_patch_popup_layout;
#[cfg(test)]
mod file_patch_popup_layout_tests;
mod file_patch_popup_text;
#[cfg(test)]
mod file_patch_popup_text_tests;
mod input;
mod input_click;
#[cfg(test)]
mod input_click_tests;
#[cfg(test)]
mod input_tests;
mod jump;
#[cfg(test)]
mod jump_tests;
mod logic;
#[cfg(test)]
mod logic_tests;
mod model_popup;
#[cfg(test)]
mod model_prompt_popup_tests;
mod events;
mod input_thread;
mod net;
mod notice;
#[cfg(test)]
mod notice_tests;
mod overlay;
mod overlay_table;
mod overlay_table_state;
#[cfg(test)]
mod overlay_table_state_tests;
#[cfg(test)]
mod overlay_table_tests;
#[cfg(test)]
mod overlay_tests;
mod perf;
mod prompt_popup;
mod runtime;
mod runtime_code_exec;
mod runtime_code_exec_helpers;
mod runtime_code_exec_output;
#[cfg(test)]
mod runtime_code_exec_output_tests;
#[cfg(test)]
mod runtime_code_exec_tests;
#[cfg(test)]
mod runtime_context_tests;
#[cfg(test)]
mod runtime_context;
mod runtime_dispatch;
#[cfg(test)]
mod runtime_dispatch_nav_tests;
#[cfg(test)]
mod runtime_dispatch_tabs_tests;
mod runtime_events;
mod runtime_events_helpers;
#[cfg(test)]
mod runtime_events_helpers_tests;
#[cfg(test)]
mod runtime_events_more_tests;
#[cfg(test)]
mod runtime_events_tests;
mod runtime_file_patch;
#[cfg(test)]
mod runtime_file_patch_tests;
mod runtime_helpers;
#[cfg(test)]
mod runtime_helpers_tests;
mod runtime_layout;
#[cfg(test)]
mod runtime_layout_tests;
mod runtime_loop;
mod runtime_loop_helpers;
#[cfg(test)]
mod runtime_loop_helpers_more_tests;
#[cfg(test)]
mod runtime_loop_helpers_tests;
mod runtime_loop_steps;
#[cfg(test)]
mod runtime_loop_tests;
mod runtime_requests;
#[cfg(test)]
mod runtime_requests_tests;
mod runtime_session;
#[cfg(test)]
mod runtime_session_tests;
#[cfg(test)]
mod runtime_tests;
mod runtime_tick;
#[cfg(test)]
mod runtime_tick_tests;
mod runtime_view;
mod runtime_view_handlers;
#[cfg(test)]
mod runtime_view_handlers_tests;
#[cfg(test)]
mod runtime_view_tests;
mod runtime_yolo;
#[cfg(test)]
mod runtime_yolo_tests;
mod scroll;
mod scroll_debug;
#[cfg(test)]
mod scroll_debug_tests;
#[cfg(test)]
mod scroll_tests;
mod selection;
mod selection_state;
#[cfg(test)]
mod selection_tests;
mod shortcut_help;
#[cfg(test)]
mod shortcut_help_tests;
mod shortcuts;
#[cfg(test)]
mod shortcuts_tests;
mod state;
#[cfg(test)]
mod state_tests;
mod summary;
#[cfg(test)]
mod summary_tests;
mod text_utils;
mod tool_service;
#[cfg(test)]
mod tool_service_tests;
mod tools;
#[cfg(test)]
mod tools_tests;
pub(crate) mod workspace;
mod widget_system;

pub use runtime::run;
