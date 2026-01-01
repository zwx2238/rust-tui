use crate::types::ToolCall;
use reqwest::Url;
use serde_json::json;

pub(crate) struct ToolResult {
    pub content: String,
    pub has_results: bool,
}

pub(crate) fn run_tool(call: &ToolCall) -> ToolResult {
    if call.function.name == "web_search" {
        return run_web_search(&call.function.arguments);
    }
    ToolResult {
        content: format!("未知工具：{}", call.function.name),
        has_results: false,
    }
}

fn run_web_search(args_json: &str) -> ToolResult {
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
    let url = match Url::parse_with_params("https://duckduckgo.com/html/", [("q", query)]) {
        Ok(val) => val,
        Err(e) => return tool_err(format!("web_search 构造 URL 失败：{e}")),
    };
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("deepchat/0.1")
        .build()
    {
        Ok(val) => val,
        Err(e) => return tool_err(format!("web_search 初始化失败：{e}")),
    };
    let body = match client.get(url).send() {
        Ok(resp) => match resp.text() {
            Ok(text) => text,
            Err(e) => return tool_err(format!("web_search 读取失败：{e}")),
        },
        Err(e) => return tool_err(format!("web_search 请求失败：{e}")),
    };
    let results = parse_duckduckgo_results(&body, top_k);
    let content = format_web_search_output(query, &results);
    ToolResult {
        content,
        has_results: !results.is_empty(),
    }
}

fn parse_duckduckgo_results(body: &str, top_k: usize) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    let mut cursor = body;
    while results.len() < top_k {
        let Some(pos) = cursor.find("result__a") else {
            break;
        };
        cursor = &cursor[pos..];
        let Some(href_pos) = cursor.find("href=\"") else {
            cursor = &cursor["result__a".len()..];
            continue;
        };
        let href_start = href_pos + "href=\"".len();
        let Some(href_end) = cursor[href_start..].find('"') else {
            break;
        };
        let href = &cursor[href_start..href_start + href_end];
        let Some(gt_pos) = cursor[href_start + href_end..].find('>') else {
            break;
        };
        let title_start = href_start + href_end + gt_pos + 1;
        let Some(title_end) = cursor[title_start..].find("</a>") else {
            break;
        };
        let title_raw = &cursor[title_start..title_start + title_end];
        let title = html_unescape(title_raw);
        let snippet = extract_snippet(cursor).unwrap_or_default();
        results.push(json!({
            "title": title,
            "url": html_unescape(href),
            "snippet": snippet,
        }));
        cursor = &cursor[title_start + title_end..];
    }
    results
}

fn extract_snippet(block: &str) -> Option<String> {
    let Some(pos) = block.find("result__snippet") else {
        return None;
    };
    let block = &block[pos..];
    let Some(gt) = block.find('>') else {
        return None;
    };
    let start = gt + 1;
    let Some(end) = block[start..].find("</a>") else {
        return None;
    };
    Some(html_unescape(&block[start..start + end]))
}

fn html_unescape(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
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
