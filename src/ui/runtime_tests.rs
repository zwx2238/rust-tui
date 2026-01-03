use crate::args::Args;
use crate::config::{Config, ModelItem};
use crate::render::RenderTheme;
use crate::test_support::{env_lock, restore_env, set_env};
use crate::ui::runtime::run;
use clap::Parser;
use ratatui::style::Color;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let dir = std::env::temp_dir().join(format!("deepchat_runtime_{name}_{ts}"));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn config_with_prompts(dir: &std::path::Path) -> Config {
    Config {
        theme: "default".to_string(),
        models: vec![ModelItem {
            key: "m1".to_string(),
            base_url: "http://example.com".to_string(),
            api_key: "k".to_string(),
            model: "model".to_string(),
        }],
        default_model: "m1".to_string(),
        prompts_dir: dir.to_string_lossy().to_string(),
        tavily_api_key: "tavily".to_string(),
    }
}

fn theme() -> RenderTheme {
    RenderTheme {
        bg: Color::Black,
        fg: Some(Color::White),
        code_bg: Color::Black,
        code_theme: "base16-ocean.dark",
        heading_fg: Some(Color::Cyan),
    }
}

#[test]
fn run_rejects_resume_and_question_set() {
    let _guard = env_lock().lock().unwrap();
    let dir = temp_dir("prompts");
    fs::write(dir.join("default.txt"), "sys").unwrap();
    let cfg = config_with_prompts(&dir);
    let mut args = Args::parse_from(["bin"]);
    args.resume = Some("abc".to_string());
    args.question_set = Some("list".to_string());
    let result = run(args, cfg, &theme());
    assert!(result.is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn run_fails_on_missing_session() {
    let _guard = env_lock().lock().unwrap();
    let dir = temp_dir("prompts");
    fs::write(dir.join("default.txt"), "sys").unwrap();
    let cfg = config_with_prompts(&dir);
    let mut args = Args::parse_from(["bin"]);
    args.resume = Some("missing-session".to_string());
    let result = run(args, cfg, &theme());
    assert!(result.is_err());
    let _ = fs::remove_dir_all(&dir);
    let prev = set_env("DEEPCHAT_TEST_RUN_LOOP_ONCE", "1");
    restore_env("DEEPCHAT_TEST_RUN_LOOP_ONCE", prev);
}
