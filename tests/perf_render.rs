use ratatui::style::Color;
use rust_tui::render::{
    RenderCacheEntry, RenderTheme, ViewportRenderParams, messages_to_viewport_text_cached,
};
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
    let theme = build_theme();
    let messages = build_long_messages(50);
    let width = 100;
    let mut cache: Vec<RenderCacheEntry> = Vec::new();

    let height = 30u16;
    let (cold, warm, cold_text) = measure_render(&messages, &theme, width, height, &mut cache);
    let max_cold = read_duration_env("PERF_COLD_MS", 2500);
    let max_warm = read_duration_env("PERF_WARM_MS", 400);

    eprintln!(
        "long_conversation_render_latency: cold={:?}, warm={:?}",
        cold, warm
    );
    write_render_output(&cold_text);

    assert_duration_under("cold render", cold, max_cold);
    assert_duration_under("warm render", warm, max_warm);
}

fn build_theme() -> RenderTheme {
    RenderTheme {
        bg: Color::Black,
        fg: None,
        code_bg: Color::Black,
        code_theme: "base16-ocean.dark",
        heading_fg: None,
    }
}

fn measure_render(
    messages: &[Message],
    theme: &RenderTheme,
    width: u16,
    height: u16,
    cache: &mut Vec<RenderCacheEntry>,
) -> (Duration, Duration, ratatui::text::Text<'static>) {
    let (cold_text, total_lines, cold) = render_once(messages, theme, width, height, cache);
    let warm = render_warm(messages, theme, width, height, cache, total_lines);
    (cold, warm, cold_text)
}

fn render_once(
    messages: &[Message],
    theme: &RenderTheme,
    width: u16,
    height: u16,
    cache: &mut Vec<RenderCacheEntry>,
) -> (ratatui::text::Text<'static>, usize, Duration) {
    let t0 = Instant::now();
    let (cold_text, total_lines) = messages_to_viewport_text_cached(
        ViewportRenderParams {
            messages,
            width: width as usize,
            theme,
            label_suffixes: &[],
            streaming_idx: None,
            scroll: 0,
            height,
        },
        cache,
    );
    (cold_text, total_lines, t0.elapsed())
}

fn render_warm(
    messages: &[Message],
    theme: &RenderTheme,
    width: u16,
    height: u16,
    cache: &mut Vec<RenderCacheEntry>,
    total_lines: usize,
) -> Duration {
    let t1 = Instant::now();
    let _ = messages_to_viewport_text_cached(
        ViewportRenderParams {
            messages,
            width: width as usize,
            theme,
            label_suffixes: &[],
            streaming_idx: None,
            scroll: total_lines
                .saturating_sub(height as usize)
                .min(u16::MAX as usize) as u16,
            height,
        },
        cache,
    );
    t1.elapsed()
}

fn read_duration_env(key: &str, default_ms: u64) -> Duration {
    let ms = std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default_ms);
    Duration::from_millis(ms)
}

fn write_render_output(cold_text: &ratatui::text::Text<'_>) {
    let Ok(path) = std::env::var("PERF_RENDER_OUTPUT") else {
        return;
    };
    let Ok(mut file) = File::create(path) else {
        return;
    };
    for line in &cold_text.lines {
        let mut s = String::new();
        for span in &line.spans {
            s.push_str(&span.content);
        }
        let _ = writeln!(file, "{s}");
    }
}
