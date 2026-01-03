mod args;
mod config;
mod conversation;
mod debug;
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
use config::{Config, default_config_path, load_config};
use question_set::{list_question_sets, question_sets_dir};
use render::theme_from_config;
use std::env;
use std::path::{Path, PathBuf};

fn run_with_args(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if args.wait_gdb {
        debug::wait_for_gdb_attach()?;
    }
    apply_env_from_args(&args);
    if maybe_list_question_sets(&args)? {
        return Ok(());
    }
    let cfg_path = config_path_from_args(&args)?;
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

fn config_path_from_args(args: &Args) -> Result<PathBuf, Box<dyn std::error::Error>> {
    match args.config.as_deref() {
        Some(p) => Ok(PathBuf::from(p)),
        None => default_config_path(),
    }
}

fn load_config_with_path(cfg_path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    load_config(cfg_path)
        .map_err(|e| format!("配置文件错误：{} ({})", cfg_path.display(), e).into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    run_with_args(args)
}

#[cfg(test)]
mod tests {
    use super::run_with_args;
    use crate::args::Args;
    use crate::test_support::{env_lock, restore_env, set_env};
    use clap::Parser;

    #[test]
    fn run_with_args_sets_network_env_for_yolo() {
        let _guard = env_lock().lock().unwrap();
        let prev = std::env::var("DEEPCHAT_CODE_EXEC_NETWORK").ok();
        restore_env("DEEPCHAT_CODE_EXEC_NETWORK", None);
        let args = Args::parse_from(["bin", "--question-set", "list", "--yolo"]);
        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(
            std::env::var("DEEPCHAT_CODE_EXEC_NETWORK").ok().as_deref(),
            Some("none")
        );
        restore_env("DEEPCHAT_CODE_EXEC_NETWORK", prev);
    }

    #[test]
    fn run_with_args_sets_read_only_env() {
        let _guard = env_lock().lock().unwrap();
        let prev = std::env::var("DEEPCHAT_READ_ONLY").ok();
        restore_env("DEEPCHAT_READ_ONLY", None);
        let args = Args::parse_from(["bin", "--question-set", "list", "--read-only"]);
        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(
            std::env::var("DEEPCHAT_READ_ONLY").ok().as_deref(),
            Some("1")
        );
        restore_env("DEEPCHAT_READ_ONLY", prev);
    }

    #[test]
    fn run_with_args_reports_config_error() {
        let args = Args::parse_from(["bin", "--config", "/tmp/deepchat-missing-config.json"]);
        let result = run_with_args(args);
        assert!(result.is_err());
    }

    #[test]
    fn run_with_args_question_set_list_is_ok() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-question-set");
        let _ = std::fs::create_dir_all(&temp);
        let prev = set_env("HOME", &temp.to_string_lossy());
        let args = Args::parse_from(["bin", "--question-set", "list"]);
        let result = run_with_args(args);
        assert!(result.is_ok());
        restore_env("HOME", prev);
    }
}
