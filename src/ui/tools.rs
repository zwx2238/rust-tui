use crate::types::ToolCall;
use reqwest::Url;
use serde_json::json;

pub(crate) fn run_tool(call: &ToolCall) -> Result<String, String> {
    if call.function.name == "web_search" {
        return run_web_search(&call.function.arguments);
    }
    Err(format!("未知工具：{}", call.function.name))
}

fn run_web_search(args_json: &str) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct Args {
        query: String,
        top_k: Option<usize>,
    }
    let args: Args = serde_json::from_str(args_json)
        .map_err(|e| format!("web_search 参数解析失败：{e}"))?;
    let query = args.query.trim();
    if query.is_empty() {
        return Err("web_search 参数 query 不能为空".to_string());
    }
    let top_k = args.top_k.unwrap_or(5).clamp(1, 10);
    let url = Url::parse_with_params("https://duckduckgo.com/html/", [("q", query)])
        .map_err(|e| format!("web_search 构造 URL 失败：{e}"))?;
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("deepchat/0.1")
        .build()
        .map_err(|e| format!("web_search 初始化失败：{e}"))?;
    let body = client
        .get(url)
        .send()
        .map_err(|e| format!("web_search 请求失败：{e}"))?
        .text()
        .map_err(|e| format!("web_search 读取失败：{e}"))?;
    let results = parse_duckduckgo_results(&body, top_k);
    let payload = json!({
        "query": query,
        "results": results,
    });
    Ok(payload.to_string())
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
