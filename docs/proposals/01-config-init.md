# 改进点 1：增加 `config init` 初始化向导

## 背景/问题
当前配置依赖手写 JSON，默认路径在 `~/.config/deepseek/config.json`（见 `src/config.rs` 与 `src/args.rs`）。首次使用时需要自己拼装 `models`、`prompts_dir`、`tavily_api_key` 等字段，容易出错，也缺少引导。

## 目标
- 一条命令完成配置初始化。
- 自动创建 prompts 目录并写入示例系统提示词。
- 生成可直接运行的默认配置文件。

## 方案概要
1. 增加 CLI 子命令：`deepchat config init`。
2. 交互式采集必要字段（模型 key/base_url/api_key/model、默认主题、prompts_dir、tavily_api_key）。
3. 若目标文件已存在，要求 `--force` 才可覆盖。
4. 可选生成 `config.example.json`，便于分享与审计。

## 涉及模块
- CLI 路由：`src/args.rs`（新增 `Command::Config` / `ConfigCommand::Init`）
- 配置写入：`src/config.rs`（复用 `save_config`，补充默认值生成）
- 交互逻辑：`src/cli/` 下新增 `config.rs`

## 验收标准
- `deepchat config init` 能在空环境下成功生成配置并提示路径。
- `prompts_dir` 自动创建且包含至少 1 个示例提示词文件。
- 使用生成的配置启动应用无错误。

## 非目标
- 不改动现有 `model add` 流程；两者并行存在。
