use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, subcommand_negates_reqs = true)]
pub struct Cli {
    /// 配置文件路径（JSON），默认：~/.config/deepseek/config.json
    #[arg(long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub args: Args,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// 模型管理（写入 config.json）
    Model {
        #[command(subcommand)]
        command: ModelCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum ModelCommand {
    /// 交互式添加/更新模型（包含 api_key）
    Add,
}

#[derive(clap::Args, Debug, Clone)]
pub struct Args {
    /// 模型 key 或模型名称（来自 config.json 的 models）
    ///
    /// - 优先按 key 匹配：models[].key
    /// - 若未命中 key，则按模型名匹配：models[].model（要求唯一）
    ///
    /// 不传则使用配置中的 default_model。
    #[arg(long)]
    pub model: Option<String>,

    /// 系统提示词
    #[arg(long, default_value = "你是一个有帮助的助手。")]
    pub system: String,

    /// API Base URL
    #[arg(long, default_value = "https://api.deepseek.com")]
    pub base_url: String,

    /// 显示 reasoning_content（如果返回）
    #[arg(long, default_value_t = false)]
    pub show_reasoning: bool,

    /// 恢复会话 ID/路径（恢复后可继续对话）
    #[arg(long, alias = "replay")]
    pub resume: Option<String>,

    /// replay 后自动分叉最后一个 tab 并重试最后一条用户消息
    #[arg(long, default_value_t = false)]
    pub replay_fork_last: bool,

    /// 工具开关表达式（逗号分隔，前缀 - 表示禁用）
    ///
    /// 默认：全部关闭（不向模型暴露任何 tools）。
    /// 示例：--enable "read_file,read_code" 或 --enable "code_exec,-modify_file"
    #[arg(long, allow_hyphen_values = true)]
    pub enable: Option<String>,

    /// 记录发送给模型的请求与输出到目录（每条消息单独文件）
    #[arg(long)]
    pub log_requests: Option<String>,

    /// 填充历史消息用于性能手工测试
    #[arg(long, default_value_t = false)]
    pub perf: bool,

    /// 启动时批量创建 10 个 tab 并发起提问（传 list 可列出可用问题集）
    #[arg(long)]
    pub question_set: Option<String>,

    /// 显式指定工作区目录（将挂载到容器内 /workspace）
    ///
    /// 说明：当使用子命令（如 `model add`）时，workspace 不需要设置。
    #[arg(long, default_value = "")]
    pub workspace: String,

    /// YOLO 模式：工具调用无需用户同意（包含代码执行/文件修改等）
    #[arg(long, default_value_t = false)]
    pub yolo: bool,

    /// 只读模式：禁止所有写入/修改类工具调用（包含代码执行/文件修改）
    #[arg(long, default_value_t = false)]
    pub read_only: bool,

    /// 等待 gdb attach 后再继续执行（用于调试）
    #[arg(long, default_value_t = false)]
    pub wait_gdb: bool,
}

impl Args {
    pub fn web_search_enabled(&self) -> bool {
        self.resolve_enabled().0
    }

    pub fn code_exec_enabled(&self) -> bool {
        self.resolve_enabled().1
    }

    pub fn read_file_enabled(&self) -> bool {
        self.resolve_enabled().2
    }

    pub fn read_code_enabled(&self) -> bool {
        self.resolve_enabled().3
    }

    pub fn modify_file_enabled(&self) -> bool {
        if self.read_only {
            return false;
        }
        self.resolve_enabled().4
    }

    pub fn yolo_enabled(&self) -> bool {
        self.yolo
    }

    pub fn read_only_enabled(&self) -> bool {
        self.read_only
    }

    fn resolve_enabled(&self) -> (bool, bool, bool, bool, bool) {
        let Some(expr) = self.enable.as_deref() else {
            return default_tool_flags().to_tuple();
        };
        resolve_enabled_from_expr(expr).to_tuple()
    }
}

#[derive(Clone, Copy)]
struct ToolFlags {
    web_search: bool,
    code_exec: bool,
    read_file: bool,
    read_code: bool,
    modify_file: bool,
}

impl ToolFlags {
    fn to_tuple(self) -> (bool, bool, bool, bool, bool) {
        (
            self.web_search,
            self.code_exec,
            self.read_file,
            self.read_code,
            self.modify_file,
        )
    }
}

fn default_tool_flags() -> ToolFlags {
    ToolFlags {
        web_search: false,
        code_exec: false,
        read_file: false,
        read_code: false,
        modify_file: false,
    }
}

fn resolve_enabled_from_expr(expr: &str) -> ToolFlags {
    let mut flags = default_tool_flags();
    for raw in expr.split(',') {
        if let Some((name, enable)) = parse_enable_item(raw) {
            apply_tool_flag(&mut flags, name, enable);
        }
    }
    flags
}

fn parse_enable_item(raw: &str) -> Option<(&str, bool)> {
    let item = raw.trim();
    if item.is_empty() {
        return None;
    }
    if let Some(rest) = item.strip_prefix('-') {
        Some((rest.trim(), false))
    } else {
        Some((item, true))
    }
}

fn apply_tool_flag(flags: &mut ToolFlags, name: &str, enable: bool) {
    match name {
        "web_search" => flags.web_search = enable,
        "code_exec" => flags.code_exec = enable,
        "read_file" => flags.read_file = enable,
        "read_code" => flags.read_code = enable,
        "modify_file" => flags.modify_file = enable,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, ModelCommand};
    use clap::Parser;

    #[test]
    fn default_flags() {
        let cli = Cli::parse_from(["bin", "--workspace", "/tmp"]);
        assert!(!cli.args.web_search_enabled());
        assert!(!cli.args.code_exec_enabled());
        assert!(!cli.args.read_file_enabled());
        assert!(!cli.args.read_code_enabled());
        assert!(!cli.args.modify_file_enabled());
        assert!(!cli.args.yolo_enabled());
        assert!(!cli.args.read_only_enabled());
    }

    #[test]
    fn enable_expression_toggles() {
        let cli = Cli::parse_from([
            "bin",
            "--enable",
            "web_search,-read_file",
            "--workspace",
            "/tmp",
        ]);
        assert!(cli.args.web_search_enabled());
        assert!(!cli.args.read_file_enabled());
        assert!(!cli.args.code_exec_enabled());
    }

    #[test]
    fn read_only_disables_modify_file_only() {
        let cli = Cli::parse_from([
            "bin",
            "--enable",
            "code_exec,modify_file",
            "--read-only",
            "--workspace",
            "/tmp",
        ]);
        assert!(cli.args.code_exec_enabled());
        assert!(!cli.args.modify_file_enabled());
    }

    #[test]
    fn subcommand_does_not_require_workspace() {
        let cli = Cli::parse_from(["bin", "model", "add"]);
        let Some(super::Command::Model { command }) = cli.command else {
            panic!("expected model subcommand");
        };
        assert!(matches!(command, ModelCommand::Add));
        assert_eq!(cli.args.workspace, "");
    }
}
