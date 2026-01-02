use crate::types::ToolCall;
use serde_json::json;
use std::path::{Path, PathBuf};

pub(crate) struct ToolResult {
    pub content: String,
    pub has_results: bool,
}

pub(crate) struct CodeExecRequest {
    pub language: String,
    pub code: String,
}

pub(crate) fn run_tool(call: &ToolCall, tavily_api_key: &str, root: Option<&Path>) -> ToolResult {
    if call.function.name == "web_search" {
        return run_web_search(&call.function.arguments, tavily_api_key);
    }
    if call.function.name == "read_file" {
        return run_read_file(&call.function.arguments, false, root);
    }
    if call.function.name == "read_code" {
        return run_read_file(&call.function.arguments, true, root);
    }
    ToolResult {
        content: format!("未知工具：{}", call.function.name),
        has_results: false,
    }
}

fn run_web_search(args_json: &str, tavily_api_key: &str) -> ToolResult {
    #[derive(serde::Deserialize)]
    struct Args {
        query: String,
        top_k: Option<usize>,
    }
    let args: Args = match serde_json::from_str(args_json) {
        Ok(val) => val,
        Err(e) => return tool_err(format!("web_search 参数解析失败：{e}")),
    };
    let query = args.query.trim();
    if query.is_empty() {
        return ToolResult {
            content: "web_search 参数 query 不能为空".to_string(),
            has_results: false,
        };
    }
    let top_k = args.top_k.unwrap_or(5).clamp(1, 10);
    if tavily_api_key.trim().is_empty() {
        return tool_err("缺少配置：tavily_api_key".to_string());
    }

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("deepchat/0.1")
        .build()
    {
        Ok(val) => val,
        Err(e) => return tool_err(format!("web_search 初始化失败：{e}")),
    };

    let payload = json!({
        "api_key": tavily_api_key,
        "query": query,
        "max_results": top_k,
        "search_depth": "basic"
    });

    let body = match client
        .post("https://api.tavily.com/search")
        .json(&payload)
        .send()
    {
        Ok(resp) => match resp.text() {
            Ok(text) => text,
            Err(e) => return tool_err(format!("web_search 读取失败：{e}")),
        },
        Err(e) => return tool_err(format!("web_search 请求失败：{e}")),
    };

    let results = parse_tavily_results(&body);
    let content = format_web_search_output(query, &results);
    ToolResult {
        content,
        has_results: !results.is_empty(),
    }
}

fn parse_tavily_results(body: &str) -> Vec<serde_json::Value> {
    #[derive(serde::Deserialize)]
    struct TavilyResult {
        title: String,
        url: String,
        content: String,
    }
    #[derive(serde::Deserialize)]
    struct TavilyResponse {
        #[serde(default)]
        results: Vec<TavilyResult>,
    }

    let parsed: TavilyResponse = match serde_json::from_str(body) {
        Ok(val) => val,
        Err(_) => return Vec::new(),
    };
    parsed
        .results
        .into_iter()
        .map(|item| {
            json!({
                "title": item.title,
                "url": item.url,
                "snippet": item.content,
            })
        })
        .collect()
}

fn format_web_search_output(query: &str, results: &[serde_json::Value]) -> String {
    let mut out = String::new();
    out.push_str(&format!("[web_search] query: {query}\n"));
    out.push_str("请仅基于下列结果回答，并使用 [1] [2] 形式引用。若结果为空，必须回答“未找到可靠结果，无法确认”。\n");
    if results.is_empty() {
        out.push_str("结果为空。\n");
        return out;
    }
    for (idx, item) in results.iter().enumerate() {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("-");
        let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("-");
        let snippet = item.get("snippet").and_then(|v| v.as_str()).unwrap_or("");
        out.push_str(&format!("[{}] {}\n", idx + 1, title));
        out.push_str(&format!("    {}\n", url));
        if !snippet.trim().is_empty() {
            out.push_str(&format!("    {}\n", snippet.trim()));
        }
    }
    out
}

fn tool_err(msg: String) -> ToolResult {
    ToolResult {
        content: msg,
        has_results: false,
    }
}

fn run_read_file(args_json: &str, with_line_numbers: bool, root: Option<&Path>) -> ToolResult {
    #[derive(serde::Deserialize)]
    struct Args {
        path: String,
        start_line: Option<usize>,
        end_line: Option<usize>,
        max_bytes: Option<usize>,
    }
    let args: Args = match serde_json::from_str(args_json) {
        Ok(val) => val,
        Err(e) => return tool_err(format!("read_file 参数解析失败：{e}")),
    };
    let path = args.path.trim();
    if path.is_empty() {
        return tool_err("read_file 参数 path 不能为空".to_string());
    }
    if let Some(root) = root {
        match enforce_root(path, root) {
            Ok(()) => {}
            Err(err) => return tool_err(format!("read_file 读取失败：{err}")),
        }
    }
    let max_bytes = args.max_bytes.unwrap_or(200_000).clamp(1, 2_000_000);
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => return tool_err(format!("read_file 读取失败：{e}")),
    };
    if meta.is_file() && meta.len() as usize > max_bytes {
        return tool_err(format!("read_file 文件过大：{} bytes", meta.len()));
    }
    let content = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => return tool_err(format!("read_file 读取失败：{e}")),
    };
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len().max(1);
    let start = args.start_line.unwrap_or(1).max(1);
    let end = args.end_line.unwrap_or(total_lines).max(start);
    let end = end.min(total_lines);
    let slice = if lines.is_empty() {
        Vec::new()
    } else {
        lines[start - 1..end].to_vec()
    };
    let mut out = String::new();
    out.push_str(if with_line_numbers {
        "[read_code]\n"
    } else {
        "[read_file]\n"
    });
    out.push_str(&format!("path: {}\n", path));
    out.push_str(&format!(
        "lines: {}-{} (total {})\n",
        start, end, total_lines
    ));
    out.push_str("content:\n");
    out.push_str("```text\n");
    if with_line_numbers {
        for (idx, line) in slice.iter().enumerate() {
            let line_no = start + idx;
            out.push_str(&format!("{:>4} | {}\n", line_no, line));
        }
    } else {
        for line in slice {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("```\n");
    ToolResult {
        content: out,
        has_results: true,
    }
}

pub(crate) fn parse_code_exec_args(args_json: &str) -> Result<CodeExecRequest, String> {
    #[derive(serde::Deserialize)]
    struct Args {
        language: String,
        code: String,
    }
    let args: Args =
        serde_json::from_str(args_json).map_err(|e| format!("code_exec 参数解析失败：{e}"))?;
    let language = args.language.trim().to_string();
    if language.is_empty() {
        return Err("code_exec 参数 language 不能为空".to_string());
    }
    if language != "python" {
        return Err("当前仅支持 python".to_string());
    }
    if args.code.trim().is_empty() {
        return Err("code_exec 参数 code 不能为空".to_string());
    }
    Ok(CodeExecRequest {
        language,
        code: args.code,
    })
}

fn enforce_root(path: &str, root: &Path) -> Result<(), String> {
    let target = PathBuf::from(path);
    let root = root
        .canonicalize()
        .map_err(|e| format!("根目录不可用：{e}"))?;
    let canonical = target
        .canonicalize()
        .map_err(|e| format!("路径不可用：{e}"))?;
    if canonical.starts_with(&root) {
        Ok(())
    } else {
        Err("禁止读取工作区外的文件".to_string())
    }
}
