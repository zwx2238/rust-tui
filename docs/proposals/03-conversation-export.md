# 改进点 3：对话导出（Markdown/JSON）

## 背景/问题
对话数据保存在 `~/.local/share/deepseek/conversations/*.json`（见 `src/conversation.rs`），但缺少正式导出能力，难以分享或归档。

## 目标
- 支持将当前对话导出为 Markdown 或 JSON。
- 导出结果可直接用于文档或后续检索。

## 方案概要
1. 新增命令：`/export`（或 `Ctrl+E`）弹出导出对话框。
2. 支持格式选择：`md` / `json`，并可指定输出路径。
3. Markdown 格式包含时间/角色/模型/系统提示词等元信息（可选开关）。
4. JSON 导出复用 `ConversationData`，并追加导出时间戳与版本号字段。

## 涉及模块
- 对话数据：`src/conversation.rs`、`src/types.rs`
- 命令解析与交互：`src/ui/commands.rs`、`src/ui/interaction/*`
- 导出渲染：`src/render/markdown/*`（可复用部分格式化能力）

## 验收标准
- 触发导出后生成目标文件且内容可读。
- Markdown 导出在普通阅读器中格式正常。
- JSON 导出可再次导入为对话（保持字段兼容）。

## 非目标
- 不提供在线分享或上传能力。
