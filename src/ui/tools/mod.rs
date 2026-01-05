use crate::types::ToolCall;
use crate::ui::code_exec_container::ensure_container_cached;
use crate::ui::workspace::{WorkspaceConfig, resolve_container_path};
use std::io::Write;
use std::process::Command;

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

pub(super) fn tool_err(msg: String) -> ToolResult {
    ToolResult {
        content: msg,
        has_results: false,
    }
}

fn run_read_file(
    args_json: &str,
    with_line_numbers: bool,
    workspace: &WorkspaceConfig,
) -> ToolResult {
    let args = match parse_read_file_args(args_json) {
        Ok(val) => val,
        Err(err) => return err,
    };
    let path = match resolve_container_path(&args.path, workspace) {
        Ok(val) => val,
        Err(err) => return tool_err(format!("read_file 读取失败：{err}")),
    };
    let content = match read_file_content(&path, args.max_bytes, workspace) {
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

fn read_file_content(
    path: &str,
    max_bytes: usize,
    workspace: &WorkspaceConfig,
) -> Result<String, ToolResult> {
    let container_id = match ensure_container_cached(workspace) {
        Ok(id) => id,
        Err(err) => return Err(tool_err(err)),
    };
    let script = r#"
import json, os, sys
args = json.load(sys.stdin)
path = args["path"]
max_bytes = int(args.get("max_bytes", 0))
try:
    st = os.stat(path)
except Exception as e:
    print(f"read_file 读取失败：{e}", file=sys.stderr)
    sys.exit(2)
if max_bytes > 0 and st.st_size > max_bytes:
    print(f"read_file 文件过大：{st.st_size} bytes", file=sys.stderr)
    sys.exit(3)
try:
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        data = f.read()
except Exception as e:
    print(f"read_file 读取失败：{e}", file=sys.stderr)
    sys.exit(4)
print(data)
"#;
    let args_json = serde_json::json!({
        "path": path,
        "max_bytes": max_bytes,
    })
    .to_string();
    let output = run_container_python(&container_id, script, args_json.as_bytes())?;
    Ok(output)
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

pub(crate) fn parse_bash_exec_args(args_json: &str) -> Result<CodeExecRequest, String> {
    #[derive(serde::Deserialize)]
    struct Args {
        command: Option<String>,
        code: Option<String>,
    }
    let args: Args =
        serde_json::from_str(args_json).map_err(|e| format!("bash_exec 参数解析失败：{e}"))?;
    let command = args
        .command
        .or(args.code)
        .unwrap_or_default()
        .trim()
        .to_string();
    if command.is_empty() {
        return Err("bash_exec 参数 command 不能为空".to_string());
    }
    Ok(CodeExecRequest {
        language: "bash".to_string(),
        code: command,
    })
}

struct ListDirArgs {
    path: String,
    recursive: bool,
    max_entries: usize,
    max_depth: usize,
    include_hidden: bool,
}

fn run_list_dir(args_json: &str, workspace: &WorkspaceConfig) -> ToolResult {
    let args = match parse_list_dir_args(args_json) {
        Ok(val) => val,
        Err(err) => return err,
    };
    let path = match resolve_container_path(&args.path, workspace) {
        Ok(val) => val,
        Err(err) => return tool_err(format!("list_dir 读取失败：{err}")),
    };
    let (entries, truncated) = match list_dir_container(&path, &args, workspace) {
        Ok(val) => val,
        Err(err) => return err,
    };
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

fn list_dir_container(
    path: &str,
    args: &ListDirArgs,
    workspace: &WorkspaceConfig,
) -> Result<(Vec<String>, bool), ToolResult> {
    let container_id = match ensure_container_cached(workspace) {
        Ok(id) => id,
        Err(err) => return Err(tool_err(err)),
    };
    let script = r#"
import json, os, sys
args = json.load(sys.stdin)
path = args["path"]
recursive = bool(args.get("recursive", False))
max_entries = int(args.get("max_entries", 2000))
max_depth = int(args.get("max_depth", 4))
include_hidden = bool(args.get("include_hidden", False))
if not os.path.isdir(path):
    print("list_dir 读取失败：不是目录", file=sys.stderr)
    sys.exit(2)
entries = []
truncated = False
stack = [(path, "", 0)]
while stack:
    base, rel_prefix, depth = stack.pop()
    try:
        items = list(os.scandir(base))
    except Exception as e:
        print(f"list_dir 读取失败：{e}", file=sys.stderr)
        sys.exit(3)
    for item in items:
        name = item.name
        if not include_hidden and name.startswith('.'):
            continue
        rel = name if not rel_prefix else f"{rel_prefix}/{name}"
        if item.is_dir(follow_symlinks=False):
            display = rel + "/"
        else:
            display = rel
        entries.append(display)
        if len(entries) >= max_entries:
            truncated = True
            break
        if recursive and item.is_dir(follow_symlinks=False) and depth < max_depth:
            stack.append((item.path, rel, depth + 1))
    if truncated:
        break
print(json.dumps({"entries": entries, "truncated": truncated}, ensure_ascii=False))
"#;
    let args_json = serde_json::json!({
        "path": path,
        "recursive": args.recursive,
        "max_entries": args.max_entries,
        "max_depth": args.max_depth,
        "include_hidden": args.include_hidden,
    })
    .to_string();
    let output = run_container_python(&container_id, script, args_json.as_bytes())?;
    let parsed: serde_json::Value =
        serde_json::from_str(&output).map_err(|e| tool_err(format!("list_dir 解析失败：{e}")))?;
    let entries = parsed
        .get("entries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| tool_err("list_dir 解析失败：entries 无效".to_string()))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let truncated = parsed
        .get("truncated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok((entries, truncated))
}

fn run_container_python(
    container_id: &str,
    script: &str,
    input: &[u8],
) -> Result<String, ToolResult> {
    let output = Command::new("docker")
        .arg("exec")
        .arg("-i")
        .arg(container_id)
        .arg("python")
        .arg("-c")
        .arg(script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input)?;
            }
            child.wait_with_output()
        })
        .map_err(|e| tool_err(format!("Docker 执行失败：{e}")))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(tool_err(if err.is_empty() {
            "Docker 执行失败".to_string()
        } else {
            err
        }))
    }
}
