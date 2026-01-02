use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// 模型名称
    #[arg(long, default_value = "deepseek-chat")]
    pub model: String,

    /// 系统提示词
    #[arg(long, default_value = "你是一个有帮助的助手。")]
    pub system: String,

    /// API Base URL
    #[arg(long, default_value = "https://api.deepseek.com")]
    pub base_url: String,

    /// 显示 reasoning_content（如果返回）
    #[arg(long, default_value_t = false)]
    pub show_reasoning: bool,

    /// 配置文件路径（JSON），默认：~/.config/deepseek/config.json
    #[arg(long)]
    pub config: Option<String>,

    /// 恢复会话 ID/路径（恢复后可继续对话）
    #[arg(long, alias = "replay")]
    pub resume: Option<String>,

    /// replay 后自动分叉最后一个 tab 并重试最后一条用户消息
    #[arg(long, default_value_t = false)]
    pub replay_fork_last: bool,

    /// 工具开关表达式（逗号分隔，前缀 - 表示禁用）
    #[arg(long, allow_hyphen_values = true)]
    pub enable: Option<String>,

    /// 记录发送给模型的请求与输出到目录（每条消息单独文件）
    #[arg(long)]
    pub log_requests: Option<String>,

    /// 填充历史消息用于性能手工测试
    #[arg(long, default_value_t = false)]
    pub perf: bool,

    /// 启动时批量创建 10 个 tab 并发起提问
    #[arg(long)]
    pub question_set: Option<String>,
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
        self.resolve_enabled().4
    }

    fn resolve_enabled(&self) -> (bool, bool, bool, bool, bool) {
        let mut web_search = false;
        let mut code_exec = true;
        let mut read_file = true;
        let mut read_code = true;
        let mut modify_file = true;
        let Some(expr) = self.enable.as_deref() else {
            return (web_search, code_exec, read_file, read_code, modify_file);
        };
        for raw in expr.split(',') {
            let item = raw.trim();
            if item.is_empty() {
                continue;
            }
            let (name, enable) = if let Some(rest) = item.strip_prefix('-') {
                (rest.trim(), false)
            } else {
                (item, true)
            };
            match name {
                "web_search" => web_search = enable,
                "code_exec" => code_exec = enable,
                "read_file" => read_file = enable,
                "read_code" => read_code = enable,
                "modify_file" => modify_file = enable,
                _ => {}
            }
        }
        (web_search, code_exec, read_file, read_code, modify_file)
    }
}
