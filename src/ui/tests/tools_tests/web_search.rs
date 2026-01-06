use crate::types::{ToolCall, ToolFunctionCall};
use crate::ui::tools::run_tool;

use super::support::workspace;

#[test]
fn web_search_requires_query_and_key() {
    let call = ToolCall {
        id: "5".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "web_search".to_string(),
            arguments: r#"{"query":""}"#.to_string(),
        },
    };
    let result = run_tool(&call, "", &workspace());
    assert!(result.content.contains("query 不能为空"));

    let call = ToolCall {
        id: "6".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "web_search".to_string(),
            arguments: r#"{"query":"hi"}"#.to_string(),
        },
    };
    let result = run_tool(&call, "", &workspace());
    assert!(result.content.contains("tavily_api_key"));
}

#[test]
fn web_search_invalid_args_reports_error() {
    let call = ToolCall {
        id: "6b".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "web_search".to_string(),
            arguments: "{".to_string(),
        },
    };
    let result = run_tool(&call, "", &workspace());
    assert!(result.content.contains("参数解析失败"));
}

#[test]
fn unknown_tool_returns_message() {
    let call = ToolCall {
        id: "7".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: "unknown".to_string(),
            arguments: "{}".to_string(),
        },
    };
    let result = run_tool(&call, "", &workspace());
    assert!(result.content.contains("未知工具"));
}
