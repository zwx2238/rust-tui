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

    /// 启用网页搜索工具
    #[arg(long, default_value_t = false)]
    pub enable_web_search: bool,

    /// 启用代码执行工具
    #[arg(long, default_value_t = false)]
    pub enable_code_exec: bool,

    /// 记录发送给模型的请求内容到文件
    #[arg(long)]
    pub log_requests: Option<String>,

    /// 填充历史消息用于性能手工测试
    #[arg(long, default_value_t = false)]
    pub perf: bool,

    /// 启动时批量创建 10 个 tab 并发起提问
    #[arg(long, default_value_t = false)]
    pub perf_batch: bool,
}
