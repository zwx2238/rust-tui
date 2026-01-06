开发约束（当前阶段：按照规范 整理代码）
1) 每次提交代码前必须执行 `just install`。
2) 禁止使用 `#[path = "..."]` 等“模块挂载黑魔法”；模块组织必须使用标准 Rust 约定（`mod.rs` 或 `foo.rs` + `foo/` 目录）。
3) 单个 Rust 源文件 > 300 行必须拆分。
4) 所有函数 ≤ 30 行。
5) 必须通过 `cargo clippy --all-targets --all-features` 且无警告，不允许 `#[allow(clippy::...)]`。
