#[cfg(test)]
mod tests {
    use crate::types::{ToolCall, ToolFunctionCall};
    use crate::ui::tools::{parse_code_exec_args, run_tool};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("deepchat-{name}-{id}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parse_code_exec_rejects_empty() {
        assert!(parse_code_exec_args(r#"{"language":"","code":""}"#).is_err());
    }

    #[test]
    fn parse_code_exec_rejects_invalid_json() {
        let err = parse_code_exec_args("{").err().unwrap();
        assert!(err.contains("参数解析失败"));
    }

    #[test]
    fn parse_code_exec_rejects_non_python() {
        let err = parse_code_exec_args(r#"{"language":"js","code":"1"}"#)
            .err()
            .unwrap();
        assert!(err.contains("仅支持"));
    }

    #[test]
    fn parse_code_exec_rejects_empty_code() {
        let err = parse_code_exec_args(r#"{"language":"python","code":"  "}"#)
            .err()
            .unwrap();
        assert!(err.contains("code 不能为空"));
    }

    #[test]
    fn parse_code_exec_accepts_python() {
        let req = parse_code_exec_args(r#"{"language":"python","code":"print(1)"}"#).unwrap();
        assert_eq!(req.language, "python");
        assert!(req.code.contains("print"));
    }

    #[test]
    fn read_file_respects_root() {
        let root = temp_dir("tools-root");
        let good_path = root.join("a.txt");
        fs::write(&good_path, "hello").unwrap();
        let call = ToolCall {
            id: "1".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, good_path.display()),
            },
        };
        let result = run_tool(&call, "", Some(&root));
        assert!(result.content.contains("hello"));
        let bad_path = temp_dir("tools-outside").join("b.txt");
        fs::write(&bad_path, "nope").unwrap();
        let call = ToolCall {
            id: "2".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, bad_path.display()),
            },
        };
        let result = run_tool(&call, "", Some(&root));
        assert!(result.content.contains("禁止读取"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn read_file_empty_path_errors() {
        let call = ToolCall {
            id: "3".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: r#"{"path":""}"#.to_string(),
            },
        };
        let result = run_tool(&call, "", None);
        assert!(result.content.contains("path 不能为空"));
    }

    #[test]
    fn read_file_invalid_json_errors() {
        let call = ToolCall {
            id: "3b".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: "{".to_string(),
            },
        };
        let result = run_tool(&call, "", None);
        assert!(result.content.contains("参数解析失败"));
    }

    #[test]
    fn read_file_too_large_errors() {
        let dir = temp_dir("tools-large");
        let path = dir.join("big.txt");
        let data = "a".repeat(1024);
        fs::write(&path, data).unwrap();
        let call = ToolCall {
            id: "3c".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: format!(r#"{{"path":"{}","max_bytes":1}}"#, path.display()),
            },
        };
        let result = run_tool(&call, "", None);
        assert!(result.content.contains("文件过大"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_file_respects_line_range() {
        let dir = temp_dir("tools-range");
        let path = dir.join("lines.txt");
        fs::write(&path, "a\nb\nc").unwrap();
        let call = ToolCall {
            id: "3d".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: format!(
                    r#"{{"path":"{}","start_line":2,"end_line":2}}"#,
                    path.display()
                ),
            },
        };
        let result = run_tool(&call, "", None);
        assert!(result.content.contains("\nb\n"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_code_includes_line_numbers() {
        let dir = temp_dir("tools-code");
        let path = dir.join("a.rs");
        fs::write(&path, "line1\nline2").unwrap();
        let call = ToolCall {
            id: "4".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_code".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, path.display()),
            },
        };
        let result = run_tool(&call, "", None);
        assert!(result.content.contains("1 | line1"));
        let _ = fs::remove_dir_all(&dir);
    }

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
        let result = run_tool(&call, "", None);
        assert!(result.content.contains("query 不能为空"));

        let call = ToolCall {
            id: "6".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "web_search".to_string(),
                arguments: r#"{"query":"hi"}"#.to_string(),
            },
        };
        let result = run_tool(&call, "", None);
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
        let result = run_tool(&call, "", None);
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
        let result = run_tool(&call, "", None);
        assert!(result.content.contains("未知工具"));
    }
}
