#!/bin/bash
# deepchat wrapper - 自动检测 worktree 并使用对应的安装版本

set -e

# 默认安装路径（主 worktree）
DEFAULT_BIN="$HOME/.cargo/bin/deepchat"

# 检测当前目录所在的 git worktree
detect_worktree() {
    local current_dir="$PWD"
    
    # 向上查找 git 仓库根目录
    while [ "$current_dir" != "/" ]; do
        if [ -d "$current_dir/.git" ] || [ -f "$current_dir/.git" ]; then
            # 获取 worktree 根目录
            local git_root
            git_root=$(cd "$current_dir" && git rev-parse --show-toplevel 2>/dev/null) || return 1
            
            # 获取分支名
            local branch
            branch=$(cd "$git_root" && git rev-parse --abbrev-ref HEAD 2>/dev/null) || return 1
            
            # 生成 worktree-id: basename(worktree-path) + "-" + branch-name
            local worktree_basename
            worktree_basename=$(basename "$git_root")
            local worktree_id="${worktree_basename}-${branch}"
            
            # 生成安装路径
            local install_path="$HOME/.cargo/worktrees/${worktree_id}/bin/deepchat"
            
            # 如果存在，返回路径
            if [ -f "$install_path" ] && [ -x "$install_path" ]; then
                echo "$install_path"
                return 0
            fi
        fi
        
        current_dir=$(dirname "$current_dir")
    done
    
    return 1
}

# 尝试检测 worktree 并获取对应的安装路径
WORKTREE_BIN=$(detect_worktree 2>/dev/null) || WORKTREE_BIN=""

# 选择要执行的二进制
if [ -n "$WORKTREE_BIN" ]; then
    # 使用 worktree 特定的版本
    exec "$WORKTREE_BIN" "$@"
elif [ -f "$DEFAULT_BIN" ] && [ -x "$DEFAULT_BIN" ]; then
    # Fallback 到默认版本
    exec "$DEFAULT_BIN" "$@"
else
    # 都找不到，显示错误
    echo "错误: 找不到 deepchat 二进制文件" >&2
    echo "请先运行 'just install' 或 'just install-wrapper'" >&2
    exit 1
fi
