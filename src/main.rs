mod args;
mod config;
mod conversation;
mod debug;
mod llm;
mod model_registry;
mod question_set;
mod render;
mod session;
mod types;
mod ui;
mod framework;
mod services;

mod cli;
use args::{Args, Cli, Command, ModelCommand};
use clap::Parser;
use config::{Config, default_config_path, load_config};
use question_set::{list_question_sets, question_sets_dir};
use render::theme_from_config;
use std::env;
use std::path::{Path, PathBuf};

fn run_with_args(
    args: crate::args::Args,
    cfg_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.wait_gdb {
        debug::wait_for_gdb_attach()?;
    }
    apply_env_from_args(&args);
    crate::services::workspace::resolve_workspace(&args)
        .map_err(|e| format!("workspace 校验失败：{e}"))?;
    if maybe_list_question_sets(&args)? {
        return Ok(());
    }
    let cfg_path = config_path_from_cli(cfg_override)?;
    let cfg = load_config_with_path(&cfg_path)?;
    let theme = theme_from_config(&cfg)?;
    ui::run(args, cfg, &theme)?;
    Ok(())
}

fn apply_env_from_args(args: &Args) {
    let enable_net_guard = args.yolo_enabled() || args.read_only_enabled();
    if enable_net_guard && env::var("DEEPCHAT_CODE_EXEC_NETWORK").is_err() {
        unsafe {
            env::set_var("DEEPCHAT_CODE_EXEC_NETWORK", "none");
        }
    }
    if args.read_only_enabled() && env::var("DEEPCHAT_READ_ONLY").is_err() {
        unsafe {
            env::set_var("DEEPCHAT_READ_ONLY", "1");
        }
    }
}

fn maybe_list_question_sets(args: &Args) -> Result<bool, Box<dyn std::error::Error>> {
    if args.question_set.as_deref() != Some("list") {
        return Ok(false);
    }
    let sets = list_question_sets()?;
    let dir = question_sets_dir()?;
    print_question_sets(&sets, &dir);
    Ok(true)
}

fn print_question_sets(sets: &[String], dir: &Path) {
    if sets.is_empty() {
        println!("未找到问题集目录或目录为空：{}", dir.display());
        return;
    }
    println!("可用问题集（{}）：", sets.len());
    for name in sets {
        println!("{name}");
    }
    println!("目录：{}", dir.display());
}

fn config_path_from_cli(cfg: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    match cfg {
        Some(p) => Ok(PathBuf::from(p)),
        None => default_config_path(),
    }
}

fn load_config_with_path(cfg_path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    load_config(cfg_path)
        .map_err(|e| format!("配置文件错误：{} ({})", cfg_path.display(), e).into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Model { command }) => match command {
            ModelCommand::Add => cli::model::run_add(cli.config.as_deref()),
        },
        None => run_with_args(cli.args, cli.config.as_deref()),
    }
}
