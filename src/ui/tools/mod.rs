mod container;
mod core;
mod exec_args;
mod list_dir;
mod read_file;
mod web_search;

pub(crate) struct ToolResult {
    pub content: String,
    pub has_results: bool,
}

pub(crate) struct CodeExecRequest {
    pub language: String,
    pub code: String,
}

pub(crate) use core::run_tool;
pub(crate) use core::tool_err;
pub(crate) use exec_args::{parse_bash_exec_args, parse_code_exec_args};
