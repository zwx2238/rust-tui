use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Deserialize)]
struct BingResponse {
    #[serde(rename = "webPages")]
    web_pages: Option<WebPages>,
}

#[derive(Deserialize)]
struct WebPages {
    value: Vec<WebPage>,
}

#[derive(Deserialize)]
struct WebPage {
    name: String,
    url: String,
    snippet: String,
}

pub fn bing_search(
    endpoint: &str,
    api_key: &str,
    query: &str,
    count: usize,
    market: Option<&str>,
) -> Result<Vec<SearchResult>, String> {
    if endpoint.trim().is_empty() {
        return Err("BING_ENDPOINT 不能为空".to_string());
    }
    if api_key.trim().is_empty() {
        return Err("BING_API_KEY 不能为空".to_string());
    }
    if query.trim().is_empty() {
        return Err("query 不能为空".to_string());
    }
    if count == 0 {
        return Err("count 必须大于 0".to_string());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("初始化失败：{e}"))?;
    let mut req = client
        .get(endpoint)
        .query(&[("q", query), ("count", &count.to_string())])
        .header("Ocp-Apim-Subscription-Key", api_key);
    if let Some(mkt) = market {
        if !mkt.trim().is_empty() {
            req = req.query(&[("mkt", mkt)]);
        }
    }
    let resp = req.send().map_err(|e| format!("请求失败：{e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("请求失败：{status} {body}"));
    }
    let data: BingResponse = resp.json().map_err(|e| format!("解析失败：{e}"))?;
    let results = data
        .web_pages
        .map(|pages| pages.value)
        .unwrap_or_default()
        .into_iter()
        .map(|item| SearchResult {
            title: item.name,
            url: item.url,
            snippet: item.snippet,
        })
        .collect();
    Ok(results)
}
