开发约束（当前阶段：按照规范 整理代码）
1) 每次提交代码前必须执行 `just install`。
3) 单个 Rust 源文件 > 300 行必须拆分。
5) 所有函数 ≤ 30 行。
6) 必须通过 `cargo clippy --all-targets --all-features` 且无警告，不允许 `#[allow(clippy::...)]`。
