use crate::types::ToolCall;
use crate::ui::workspace::WorkspaceConfig;

use super::list_dir::run_list_dir;
use super::read_file::run_read_file;
use super::web_search::run_web_search;
use super::ToolResult;

pub(crate) fn run_tool(
    call: &ToolCall,
    tavily_api_key: &str,
    workspace: &WorkspaceConfig,
) -> ToolResult {
    if call.function.name == "web_search" {
        return run_web_search(&call.function.arguments, tavily_api_key);
    }
    if call.function.name == "read_file" {
        return run_read_file(&call.function.arguments, false, workspace);
    }
    if call.function.name == "read_code" {
        return run_read_file(&call.function.arguments, true, workspace);
    }
    if call.function.name == "list_dir" {
        return run_list_dir(&call.function.arguments, workspace);
    }
    ToolResult {
        content: format!("未知工具：{}", call.function.name),
        has_results: false,
    }
}

pub(crate) fn tool_err(msg: String) -> ToolResult {
    ToolResult {
        content: msg,
        has_results: false,
    }
}
