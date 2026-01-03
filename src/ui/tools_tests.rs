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
}
