# 改进点 2：会话自动保存与崩溃恢复

## 背景/问题
会话结构在 `src/session.rs` 中已支持保存/加载，但当前保存依赖显式触发，异常退出可能丢失打开的对话列表、分类与当前聚焦状态。

## 目标
- 提供周期性自动保存（可配置）。
- 崩溃或强制退出后，启动时可提示恢复最近会话。

## 方案概要
1. 新增自动保存开关与间隔配置（例如 `session_autosave_interval_secs`）。
2. 在运行时循环内按间隔写入 `~/.local/share/deepseek/sessions/auto.json`。
3. 启动时若检测到 `auto.json` 更新时间晚于最后显式保存，则弹窗询问是否恢复。
4. 在界面底部展示最近一次自动保存时间或保存失败提示。

## 涉及模块
- 会话持久化：`src/session.rs`
- 运行时保存触发：`src/ui/runtime_session`、`src/ui/runtime_impl/state`
- 弹窗与提示：`src/ui/notice.rs` / `src/ui/popup/*`

## 验收标准
- 正常运行时每 N 秒产生一次 `auto.json`，且不影响 UI 响应。
- 异常退出后再次启动可选择恢复上次会话。
- 关闭自动保存时不会产生 `auto.json`。

## 非目标
- 不做跨设备云同步。
