use crate::services::code_exec_container::ensure_container_cached;
use crate::services::workspace::{WorkspaceConfig, resolve_container_path};

use super::container::run_container_python;
use super::{ToolResult, tool_err};

pub(super) fn run_list_dir(args_json: &str, workspace: &WorkspaceConfig) -> ToolResult {
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

struct ListDirArgs {
    path: String,
    recursive: bool,
    max_entries: usize,
    max_depth: usize,
    include_hidden: bool,
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
    let script = list_dir_script();
    let args_json = list_dir_args_json(path, args);
    let output = run_container_python(&container_id, script, args_json.as_bytes())?;
    parse_list_dir_output(&output)
}

const LIST_DIR_SCRIPT: &str = r#"
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

fn list_dir_script() -> &'static str {
    LIST_DIR_SCRIPT
}

fn list_dir_args_json(path: &str, args: &ListDirArgs) -> String {
    serde_json::json!({
        "path": path,
        "recursive": args.recursive,
        "max_entries": args.max_entries,
        "max_depth": args.max_depth,
        "include_hidden": args.include_hidden,
    })
    .to_string()
}

fn parse_list_dir_output(output: &str) -> Result<(Vec<String>, bool), ToolResult> {
    let parsed: serde_json::Value =
        serde_json::from_str(output).map_err(|e| tool_err(format!("list_dir 解析失败：{e}")))?;
    let entries = parse_list_entries(&parsed)?;
    let truncated = parsed
        .get("truncated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok((entries, truncated))
}

fn parse_list_entries(parsed: &serde_json::Value) -> Result<Vec<String>, ToolResult> {
    let entries = parsed
        .get("entries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| tool_err("list_dir 解析失败：entries 无效".to_string()))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    Ok(entries)
}
