use super::{ToolResult, tool_err};
use serde_json::json;

pub(super) fn run_web_search(args_json: &str, tavily_api_key: &str) -> ToolResult {
    let (query, top_k) = match parse_web_search_args(args_json) {
        Ok(val) => val,
        Err(err) => return err,
    };
    if tavily_api_key.trim().is_empty() {
        return tool_err("缺少配置：tavily_api_key".to_string());
    }
    let client = match build_web_client() {
        Ok(val) => val,
        Err(err) => return err,
    };
    let body = match send_web_search(&client, tavily_api_key, &query, top_k) {
        Ok(val) => val,
        Err(err) => return err,
    };
    let results = parse_tavily_results(&body);
    let content = format_web_search_output(&query, &results);
    ToolResult {
        content,
        has_results: !results.is_empty(),
    }
}

fn parse_web_search_args(args_json: &str) -> Result<(String, usize), ToolResult> {
    #[derive(serde::Deserialize)]
    struct Args {
        query: String,
        top_k: Option<usize>,
    }
    let args: Args = serde_json::from_str(args_json)
        .map_err(|e| tool_err(format!("web_search 参数解析失败：{e}")))?;
    let query = args.query.trim().to_string();
    if query.is_empty() {
        return Err(tool_err("web_search 参数 query 不能为空".to_string()));
    }
    let top_k = args.top_k.unwrap_or(5).clamp(1, 10);
    Ok((query, top_k))
}

fn build_web_client() -> Result<reqwest::blocking::Client, ToolResult> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("deepchat/0.1")
        .build()
        .map_err(|e| tool_err(format!("web_search 初始化失败：{e}")))
}

fn send_web_search(
    client: &reqwest::blocking::Client,
    tavily_api_key: &str,
    query: &str,
    top_k: usize,
) -> Result<String, ToolResult> {
    let payload = json!({
        "api_key": tavily_api_key,
        "query": query,
        "max_results": top_k,
        "search_depth": "basic"
    });
    let resp = client
        .post("https://api.tavily.com/search")
        .json(&payload)
        .send()
        .map_err(|e| tool_err(format!("web_search 请求失败：{e}")))?;
    resp.text()
        .map_err(|e| tool_err(format!("web_search 读取失败：{e}")))
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
