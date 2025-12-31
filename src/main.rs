mod args;
mod config;
mod model_registry;
mod render;
mod session;
mod ui;
mod types;

use args::Args;
use clap::Parser;
use config::{default_config_path, load_config, load_config_optional};
use render::{messages_to_plain_lines, theme_from_config};
use session::load_session;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let cfg_path = match args.config.as_deref() {
        Some(p) => PathBuf::from(p),
        None => default_config_path()?,
    };
    let cfg_for_theme = load_config_optional(&cfg_path);
    let theme = theme_from_config(cfg_for_theme.as_ref());

    if let Some(id) = args.replay.as_deref() {
        let session_messages = load_session(id)
            .map_err(|_| format!("无法读取回放会话：{id}"))?;
        println!("回放模式已开启：{id}");
        let width = crossterm::terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(80)
            .saturating_sub(2);
        for line in messages_to_plain_lines(&session_messages, width, &theme) {
            println!("{line}");
        }
        return Ok(());
    }
    let cfg = load_config(&cfg_path)
        .map_err(|_| format!("无法读取配置文件：{}", cfg_path.display()))?;
    let api_key = cfg.api_key.clone().unwrap_or_default();

    ui::run(args, api_key, cfg_for_theme, &theme)?;
    Ok(())
}
