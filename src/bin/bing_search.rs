use rust_tui::search::bing::bing_search;
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    let query = match args.next() {
        Some(q) => q,
        None => {
            eprintln!("用法: bing_search <query> [count] [market]\n需要环境变量: BING_ENDPOINT, BING_API_KEY");
            std::process::exit(1);
        }
    };
    let count = args
        .next()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);
    let market = args.next();

    let endpoint = env::var("BING_ENDPOINT").unwrap_or_default();
    let api_key = env::var("BING_API_KEY").unwrap_or_default();

    match bing_search(&endpoint, &api_key, &query, count, market.as_deref()) {
        Ok(results) => {
            if results.is_empty() {
                println!("结果为空");
                return;
            }
            for (idx, r) in results.iter().enumerate() {
                println!("[{}] {}", idx + 1, r.title);
                println!("    {}", r.url);
                if !r.snippet.trim().is_empty() {
                    println!("    {}", r.snippet.trim());
                }
            }
        }
        Err(e) => {
            eprintln!("搜索失败: {e}");
            std::process::exit(2);
        }
    }
}
