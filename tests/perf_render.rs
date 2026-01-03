use ratatui::style::Color;
use rust_tui::render::{RenderCacheEntry, RenderTheme, messages_to_viewport_text_cached};
use rust_tui::types::Message;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

fn build_long_messages(count: usize) -> Vec<Message> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let role = if i % 2 == 0 { "user" } else { "assistant" };
        let mut content = String::new();
        for line in 0..100 {
            if line % 7 == 0 {
                content.push_str(&format!("```rust\nfn demo_{i}_{line}() {{\n    // 注释 {i}-{line}\n    let x = {line};\n    println!(\"{}\", x);\n}}\n```\n", "{x}"));
            } else if line % 7 == 1 {
                content.push_str(&format!(
                    "这是一段较长的正文 {i}-{line}。包含多行文字，模拟真实对话内容。\n"
                ));
            } else {
                content.push_str(&format!("普通行 {i}-{line}\n"));
            }
        }
        out.push(Message {
            role: role.to_string(),
            content,
            tool_call_id: None,
            tool_calls: None,
        });
    }
    out
}

fn assert_duration_under(label: &str, d: Duration, limit: Duration) {
    assert!(
        d <= limit,
        "{label} 超时：{d:?} > {limit:?}，请检查渲染性能",
    );
}

#[test]
#[ignore]
fn long_conversation_render_latency() {
    let theme = RenderTheme {
        bg: Color::Black,
        fg: None,
        code_bg: Color::Black,
        code_theme: "base16-ocean.dark",
        heading_fg: None,
    };
    let messages = build_long_messages(50);
    let width = 100;
    let mut cache: Vec<RenderCacheEntry> = Vec::new();

    let t0 = Instant::now();
    let height = 30u16;
    let (cold_text, total_lines) = messages_to_viewport_text_cached(
        &messages,
        width,
        &theme,
        &[],
        None,
        0,
        height,
        &mut cache,
    );
    let cold = t0.elapsed();

    let t1 = Instant::now();
    let _ = messages_to_viewport_text_cached(
        &messages,
        width,
        &theme,
        &[],
        None,
        total_lines
            .saturating_sub(height as usize)
            .min(u16::MAX as usize) as u16,
        height,
        &mut cache,
    );
    let warm = t1.elapsed();

    let max_cold = Duration::from_millis(
        std::env::var("PERF_COLD_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2500),
    );
    let max_warm = Duration::from_millis(
        std::env::var("PERF_WARM_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(400),
    );

    eprintln!(
        "long_conversation_render_latency: cold={:?}, warm={:?}",
        cold, warm
    );

    if let Ok(path) = std::env::var("PERF_RENDER_OUTPUT") {
        if let Ok(mut file) = File::create(path) {
            for line in cold_text.lines {
                let mut s = String::new();
                for span in line.spans {
                    s.push_str(&span.content);
                }
                let _ = writeln!(file, "{s}");
            }
        }
    }

    assert_duration_under("cold render", cold, max_cold);
    assert_duration_under("warm render", warm, max_warm);
}
