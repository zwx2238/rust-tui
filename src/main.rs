mod args;
mod config;
mod model_registry;
mod prompt_pack;
mod render;
mod session;
mod system_prompts;
mod types;
mod ui;

use args::Args;
use clap::Parser;
use config::{default_config_path, load_config, load_config_optional};
use render::theme_from_config;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let cfg_path = match args.config.as_deref() {
        Some(p) => PathBuf::from(p),
        None => default_config_path()?,
    };
    let cfg_for_theme = load_config_optional(&cfg_path);
    let theme = theme_from_config(cfg_for_theme.as_ref());

    let cfg =
        load_config(&cfg_path).map_err(|_| format!("无法读取配置文件：{}", cfg_path.display()))?;
    let api_key = cfg.api_key.clone().unwrap_or_default();

    ui::run(args, api_key, cfg_for_theme, &theme)?;
    Ok(())
}
