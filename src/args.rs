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

    /// 回放会话 ID（不触发真实 API）
    #[arg(long, alias = "resume")]
    pub replay: Option<String>,

    /// 填充历史消息用于性能手工测试
    #[arg(long, default_value_t = false)]
    pub perf: bool,

    /// 启动时批量创建 10 个 tab 并发起提问
    #[arg(long, default_value_t = false)]
    pub perf_batch: bool,
}
