mod args;
mod config;
mod conversation;
mod llm;
mod model_registry;
mod question_set;
mod render;
mod session;
#[cfg(test)]
mod test_support;
mod types;
mod ui;

use args::Args;
use clap::Parser;
use config::{default_config_path, load_config};
use question_set::{list_question_sets, question_sets_dir};
use render::theme_from_config;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if (args.yolo_enabled() || args.read_only_enabled())
        && env::var("DEEPCHAT_CODE_EXEC_NETWORK").is_err()
    {
        unsafe {
            env::set_var("DEEPCHAT_CODE_EXEC_NETWORK", "none");
        }
    }
    if args.read_only_enabled() && env::var("DEEPCHAT_READ_ONLY").is_err() {
        unsafe {
            env::set_var("DEEPCHAT_READ_ONLY", "1");
        }
    }

    if args.question_set.as_deref() == Some("list") {
        let sets = list_question_sets()?;
        let dir = question_sets_dir()?;
        if sets.is_empty() {
            println!("未找到问题集目录或目录为空：{}", dir.display());
        } else {
            println!("可用问题集（{}）：", sets.len());
            for name in sets {
                println!("{name}");
            }
            println!("目录：{}", dir.display());
        }
        return Ok(());
    }

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
