mod args;
mod config;
mod model_registry;
mod question_set;
mod render;
mod session;
mod types;
mod ui;
mod llm;

use args::Args;
use clap::Parser;
use config::{default_config_path, load_config};
use render::theme_from_config;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let cfg_path = match args.config.as_deref() {
        Some(p) => PathBuf::from(p),
        None => default_config_path()?,
    };
    let cfg = load_config(&cfg_path)
        .map_err(|e| format!("配置文件错误：{} ({})", cfg_path.display(), e))?;
    let theme = theme_from_config(&cfg)?;

    ui::run(args, cfg, &theme)?;
    Ok(())
}
