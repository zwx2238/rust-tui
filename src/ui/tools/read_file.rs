use crate::ui::code_exec_container::ensure_container_cached;
use crate::ui::workspace::{WorkspaceConfig, resolve_container_path};

use super::container::run_container_python;
use super::{ToolResult, tool_err};

pub(super) fn run_read_file(
    args_json: &str,
    with_line_numbers: bool,
    workspace: &WorkspaceConfig,
) -> ToolResult {
    let args = match parse_read_file_args(args_json) {
        Ok(val) => val,
        Err(err) => return err,
    };
    let content = match read_file_for_args(&args, workspace) {
        Ok(val) => val,
        Err(err) => return err,
    };
    let out = format_read_file_result(&args, with_line_numbers, &content);
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
    let script = read_file_script();
    let args_json = read_file_args_json(path, max_bytes);
    let output = run_container_python(&container_id, script, args_json.as_bytes())?;
    Ok(output)
}

const READ_FILE_SCRIPT: &str = r#"
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

fn read_file_script() -> &'static str {
    READ_FILE_SCRIPT
}

fn read_file_args_json(path: &str, max_bytes: usize) -> String {
    serde_json::json!({
        "path": path,
        "max_bytes": max_bytes,
    })
    .to_string()
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

fn read_file_for_args(
    args: &ReadFileArgs,
    workspace: &WorkspaceConfig,
) -> Result<String, ToolResult> {
    let path = match resolve_container_path(&args.path, workspace) {
        Ok(val) => val,
        Err(err) => return Err(tool_err(format!("read_file 读取失败：{err}"))),
    };
    read_file_content(&path, args.max_bytes, workspace)
}

fn format_read_file_result(
    args: &ReadFileArgs,
    with_line_numbers: bool,
    content: &str,
) -> String {
    let lines = content.lines().collect::<Vec<_>>();
    let (start, end, total_lines, slice) = slice_lines(&lines, args.start_line, args.end_line);
    format_read_file_output(
        &args.path,
        with_line_numbers,
        start,
        end,
        total_lines,
        &slice,
    )
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
