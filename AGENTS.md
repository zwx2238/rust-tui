开发约束

1) 每次改动代码后，必须执行 `just install`，确保用户可直接用 bin 名称调用。
2) 若安装或运行出现问题，必须定位并修复后再交付。
3) 单个 Rust 源文件超过 300 行必须拆分重构。

## Git Worktree 使用说明

当使用 git worktree 时，需要确保不同 worktree 的 bin 隔离，避免相互覆盖。
所有 worktree 都使用统一的 `deepchat` 命令，通过全局 wrapper 自动检测并调用对应的版本。

### 首次设置（只需一次）

安装全局 wrapper，这样可以在任何地方使用 `deepchat` 命令：

```bash
just install-wrapper
```

### 在每个 worktree 中安装

在每个 worktree 中运行：

```bash
just install
```

这个命令会：
1. 自动检测当前 worktree 和分支名
2. 将 deepchat 安装到独立目录：`~/.cargo/worktrees/<worktree-name>-<branch>/bin/deepchat`
3. 自动安装/更新全局 wrapper（如果尚未安装）

### 日常使用

安装完成后，在任何地方直接使用：

```bash
deepchat
```

Wrapper 会自动：
- 检测当前目录所在的 git worktree
- 根据 worktree 路径和分支名找到对应的安装版本
- 如果找不到 worktree 版本，fallback 到默认的 `~/.cargo/bin/deepchat`（主 worktree）
- 如果默认版本也不存在，显示错误信息

### 工作原理

1. **全局 Wrapper**：安装在 `~/.cargo/bin/deepchat`，所有 worktree 共用
2. **自动检测**：Wrapper 向上查找 `.git` 目录，获取 worktree 根目录和分支名
3. **路径生成**：`worktree-id = basename(worktree-path) + "-" + branch-name`
4. **安装路径**：`~/.cargo/worktrees/<worktree-id>/bin/deepchat`
5. **优雅降级**：不在 git 仓库中或找不到对应版本时，使用默认版本

### 优势

- **零配置** - 安装 wrapper 后，直接使用 `deepchat` 命令
- **自动切换** - 自动检测 worktree，无需手动操作
- **单一入口** - 所有 worktree 使用同一个 `deepchat` 命令
- **优雅降级** - 不在 worktree 中时使用默认版本
- **Justfile 管理** - 所有安装逻辑集中在 justfile 中
