use crate::types::ToolCall;
use std::path::{Path, PathBuf};

mod web_search;
use web_search::run_web_search;

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
    if call.function.name == "list_dir" {
        return run_list_dir(&call.function.arguments, root);
    }
    ToolResult {
        content: format!("未知工具：{}", call.function.name),
        has_results: false,
    }
}

pub(super) fn tool_err(msg: String) -> ToolResult {
    ToolResult {
        content: msg,
        has_results: false,
    }
}

fn run_read_file(args_json: &str, with_line_numbers: bool, root: Option<&Path>) -> ToolResult {
    let args = match parse_read_file_args(args_json) {
        Ok(val) => val,
        Err(err) => return err,
    };
    if let Some(root) = root
        && let Err(err) = enforce_root(&args.path, root)
    {
        return tool_err(format!("read_file 读取失败：{err}"));
    }
    let content = match read_file_content(&args.path, args.max_bytes) {
        Ok(val) => val,
        Err(err) => return err,
    };
    let lines = content.lines().collect::<Vec<_>>();
    let (start, end, total_lines, slice) = slice_lines(&lines, args.start_line, args.end_line);
    let out = format_read_file_output(
        &args.path,
        with_line_numbers,
        start,
        end,
        total_lines,
        &slice,
    );
    ToolResult {
        content: out,
        has_results: true,
    }
}

struct ReadFileArgs {
    path: String,
    start_line: Option<usize>,
    end_line: Option<usize>,
    max_bytes: usize,
}

fn parse_read_file_args(args_json: &str) -> Result<ReadFileArgs, ToolResult> {
    #[derive(serde::Deserialize)]
    struct Args {
        path: String,
        start_line: Option<usize>,
        end_line: Option<usize>,
        max_bytes: Option<usize>,
    }
    let args: Args = serde_json::from_str(args_json)
        .map_err(|e| tool_err(format!("read_file 参数解析失败：{e}")))?;
    let path = args.path.trim().to_string();
    if path.is_empty() {
        return Err(tool_err("read_file 参数 path 不能为空".to_string()));
    }
    let max_bytes = args.max_bytes.unwrap_or(200_000).clamp(1, 2_000_000);
    Ok(ReadFileArgs {
        path,
        start_line: args.start_line,
        end_line: args.end_line,
        max_bytes,
    })
}

fn read_file_content(path: &str, max_bytes: usize) -> Result<String, ToolResult> {
    let meta = std::fs::metadata(path).map_err(|e| tool_err(format!("read_file 读取失败：{e}")))?;
    if meta.is_file() && meta.len() as usize > max_bytes {
        return Err(tool_err(format!(
            "read_file 文件过大：{} bytes",
            meta.len()
        )));
    }
    std::fs::read_to_string(path).map_err(|e| tool_err(format!("read_file 读取失败：{e}")))
}

fn slice_lines<'a>(
    lines: &'a [&'a str],
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> (usize, usize, usize, Vec<&'a str>) {
    let total_lines = lines.len().max(1);
    let start = start_line.unwrap_or(1).max(1);
    let end = end_line.unwrap_or(total_lines).max(start).min(total_lines);
    let slice = if lines.is_empty() {
        Vec::new()
    } else {
        lines[start - 1..end].to_vec()
    };
    (start, end, total_lines, slice)
}

fn format_read_file_output(
    path: &str,
    with_line_numbers: bool,
    start: usize,
    end: usize,
    total_lines: usize,
    slice: &[&str],
) -> String {
    let mut out = format_read_file_header(path, with_line_numbers, start, end, total_lines);
    append_read_file_content(&mut out, with_line_numbers, start, slice);
    out
}

fn format_read_file_header(
    path: &str,
    with_line_numbers: bool,
    start: usize,
    end: usize,
    total_lines: usize,
) -> String {
    let mut out = String::new();
    out.push_str(if with_line_numbers {
        "[read_code]\n"
    } else {
        "[read_file]\n"
    });
    out.push_str(&format!("path: {}\n", path));
    out.push_str(&format!("lines: {}-{} (total {})\n", start, end, total_lines));
    out.push_str("content:\n");
    out.push_str("```text\n");
    out
}

fn append_read_file_content(
    out: &mut String,
    with_line_numbers: bool,
    start: usize,
    slice: &[&str],
) {
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

struct ListDirArgs {
    path: String,
    recursive: bool,
    max_entries: usize,
    max_depth: usize,
    include_hidden: bool,
}

fn run_list_dir(args_json: &str, root: Option<&Path>) -> ToolResult {
    let args = match parse_list_dir_args(args_json) {
        Ok(val) => val,
        Err(err) => return err,
    };
    if let Some(root) = root
        && let Err(err) = enforce_root(&args.path, root)
    {
        return tool_err(format!("list_dir 读取失败：{err}"));
    }
    let base = PathBuf::from(&args.path);
    let meta = match std::fs::metadata(&base) {
        Ok(val) => val,
        Err(err) => return tool_err(format!("list_dir 读取失败：{err}")),
    };
    if !meta.is_dir() {
        return tool_err("list_dir 读取失败：不是目录".to_string());
    }
    let root_canon = match root {
        Some(root) => match root.canonicalize() {
            Ok(val) => Some(val),
            Err(err) => return tool_err(format!("list_dir 读取失败：根目录不可用：{err}")),
        },
        None => None,
    };
    let mut entries: Vec<String> = Vec::new();
    let mut truncated = false;
    let mut stack: Vec<(PathBuf, String, usize)> = vec![(base.clone(), String::new(), 0)];
    while let Some((dir, rel_prefix, depth)) = stack.pop() {
        let read_dir = match std::fs::read_dir(&dir) {
            Ok(val) => val,
            Err(err) => return tool_err(format!("list_dir 读取失败：{err}")),
        };
        for entry in read_dir {
            let entry = match entry {
                Ok(val) => val,
                Err(err) => return tool_err(format!("list_dir 读取失败：{err}")),
            };
            let path = entry.path();
            if let Some(root) = root_canon.as_ref()
                && !is_under_root(&path, root)
            {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !args.include_hidden && name.starts_with('.') {
                continue;
            }
            let rel_path = if rel_prefix.is_empty() {
                name
            } else {
                format!("{}/{}", rel_prefix, name)
            };
            let file_type = match entry.file_type() {
                Ok(val) => val,
                Err(err) => return tool_err(format!("list_dir 读取失败：{err}")),
            };
            let display = if file_type.is_dir() {
                format!("{}/", rel_path)
            } else {
                rel_path.clone()
            };
            entries.push(display);
            if entries.len() >= args.max_entries {
                truncated = true;
                break;
            }
            if args.recursive && file_type.is_dir() && depth < args.max_depth {
                stack.push((path, rel_path, depth + 1));
            }
        }
        if truncated {
            break;
        }
    }
    let out = format_list_dir_output(&args, entries, truncated);
    ToolResult {
        content: out,
        has_results: true,
    }
}

fn parse_list_dir_args(args_json: &str) -> Result<ListDirArgs, ToolResult> {
    #[derive(serde::Deserialize)]
    struct Args {
        path: String,
        recursive: Option<bool>,
        max_entries: Option<usize>,
        max_depth: Option<usize>,
        include_hidden: Option<bool>,
    }
    let args: Args = serde_json::from_str(args_json)
        .map_err(|e| tool_err(format!("list_dir 参数解析失败：{e}")))?;
    let path = args.path.trim().to_string();
    if path.is_empty() {
        return Err(tool_err("list_dir 参数 path 不能为空".to_string()));
    }
    let recursive = args.recursive.unwrap_or(false);
    let max_entries = args.max_entries.unwrap_or(2000).clamp(1, 20_000);
    let max_depth = args.max_depth.unwrap_or(4).clamp(1, 32);
    let include_hidden = args.include_hidden.unwrap_or(false);
    Ok(ListDirArgs {
        path,
        recursive,
        max_entries,
        max_depth,
        include_hidden,
    })
}

fn format_list_dir_output(args: &ListDirArgs, entries: Vec<String>, truncated: bool) -> String {
    let mut out = String::new();
    out.push_str("[list_dir]\n");
    out.push_str(&format!("path: {}\n", args.path));
    out.push_str(&format!("recursive: {}\n", args.recursive));
    out.push_str(&format!("max_depth: {}\n", args.max_depth));
    out.push_str(&format!("include_hidden: {}\n", args.include_hidden));
    out.push_str(&format!("entries: {}\n", entries.len()));
    if truncated {
        out.push_str("truncated: true\n");
    }
    out.push_str("content:\n");
    out.push_str("```text\n");
    for entry in entries {
        out.push_str(&entry);
        out.push('\n');
    }
    out.push_str("```\n");
    out
}

fn is_under_root(path: &Path, root: &Path) -> bool {
    match path.canonicalize() {
        Ok(canon) => canon.starts_with(root),
        Err(_) => false,
    }
}
