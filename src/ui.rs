mod clipboard;
mod code_exec_container;
mod code_exec_container_env;
mod code_exec_popup;
mod code_exec_popup_layout;
mod code_exec_popup_text;
mod command_input;
mod command_suggestions;
mod commands;
mod draw;
mod draw_input;
mod file_patch_popup;
mod file_patch_popup_layout;
mod file_patch_popup_text;
mod input;
mod input_click;
mod jump;
mod logic;
mod model_popup;
mod net;
mod notice;
mod overlay;
mod overlay_render;
mod overlay_render_base;
mod overlay_render_tool;
mod overlay_table;
mod overlay_table_state;
mod perf;
mod prompt_popup;
mod render_context;
mod runtime;
mod runtime_code_exec;
mod runtime_code_exec_helpers;
mod runtime_code_exec_output;
mod runtime_context;
mod runtime_dispatch;
mod runtime_events;
mod runtime_events_helpers;
mod runtime_file_patch;
mod runtime_helpers;
mod runtime_layout;
mod runtime_loop;
mod runtime_loop_helpers;
mod runtime_render;
mod runtime_requests;
mod runtime_session;
mod runtime_tick;
mod runtime_view;
mod runtime_view_handlers;
mod runtime_yolo;
mod scroll;
mod scroll_debug;
mod selection;
mod selection_state;
mod shortcut_help;
mod shortcuts;
mod state;
mod summary;
mod text_utils;
mod tool_service;
mod tools;
#[cfg(test)]
mod tools_tests;
#[cfg(test)]
mod command_input_tests;
#[cfg(test)]
mod command_suggestions_tests;
#[cfg(test)]
mod commands_tests;
#[cfg(test)]
mod code_exec_container_tests;
#[cfg(test)]
mod code_exec_popup_layout_tests;
#[cfg(test)]
mod code_exec_popup_tests;
#[cfg(test)]
mod code_exec_popup_text_tests;
#[cfg(test)]
mod draw_tests;
#[cfg(test)]
mod file_patch_popup_layout_tests;
#[cfg(test)]
mod file_patch_popup_tests;
#[cfg(test)]
mod file_patch_popup_text_tests;
#[cfg(test)]
mod logic_tests;
#[cfg(test)]
mod overlay_table_state_tests;
#[cfg(test)]
mod overlay_table_tests;
#[cfg(test)]
mod overlay_tests;
#[cfg(test)]
mod runtime_helpers_tests;
#[cfg(test)]
mod runtime_layout_tests;
#[cfg(test)]
mod runtime_tick_tests;
#[cfg(test)]
mod runtime_view_tests;
#[cfg(test)]
mod runtime_view_handlers_tests;
#[cfg(test)]
mod runtime_yolo_tests;
#[cfg(test)]
mod runtime_dispatch_tests;
#[cfg(test)]
mod runtime_dispatch_key_tests;
#[cfg(test)]
mod runtime_dispatch_key_more_tests;
#[cfg(test)]
mod runtime_dispatch_key_more2_tests;
#[cfg(test)]
mod runtime_dispatch_mouse_tests;
#[cfg(test)]
mod runtime_dispatch_nav_tests;
#[cfg(test)]
mod runtime_dispatch_tabs_tests;
#[cfg(test)]
mod runtime_events_helpers_tests;
#[cfg(test)]
mod runtime_events_tests;
#[cfg(test)]
mod runtime_events_more_tests;
#[cfg(test)]
mod runtime_loop_helpers_tests;
#[cfg(test)]
mod runtime_loop_helpers_more_tests;
#[cfg(test)]
mod runtime_loop_tests;
#[cfg(test)]
mod runtime_render_tests;
#[cfg(test)]
mod runtime_context_tests;
#[cfg(test)]
mod runtime_tests;
#[cfg(test)]
mod runtime_file_patch_tests;
#[cfg(test)]
mod runtime_requests_tests;
#[cfg(test)]
mod runtime_session_tests;
#[cfg(test)]
mod runtime_code_exec_output_tests;
#[cfg(test)]
mod runtime_code_exec_tests;
#[cfg(test)]
mod scroll_debug_tests;
#[cfg(test)]
mod scroll_tests;
#[cfg(test)]
mod selection_tests;
#[cfg(test)]
mod shortcut_help_tests;
#[cfg(test)]
mod shortcuts_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod input_tests;
#[cfg(test)]
mod input_click_tests;
#[cfg(test)]
mod notice_tests;
#[cfg(test)]
mod summary_tests;
#[cfg(test)]
mod tool_service_tests;
#[cfg(test)]
mod jump_tests;
#[cfg(test)]
mod model_prompt_popup_tests;

pub use runtime::run;
