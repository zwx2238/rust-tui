# Justfile for deepchat installation and wrapper management

# 安装到当前 worktree 的独立目录，并安装/更新全局 wrapper
install:
	@bash -c 'set -e; GIT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || { echo "错误: 当前目录不在 git 仓库中"; exit 1; }; BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null) || { echo "错误: 无法获取当前分支名"; exit 1; }; WORKTREE_BASENAME=$(basename "$GIT_ROOT"); WORKTREE_ID="${WORKTREE_BASENAME}-${BRANCH}"; INSTALL_ROOT="$HOME/.cargo/worktrees/${WORKTREE_ID}"; echo "检测到 worktree: $WORKTREE_ID"; echo "安装路径: $INSTALL_ROOT"; cargo install --path . --root "$INSTALL_ROOT" --force; echo "✓ 已安装到 $INSTALL_ROOT/bin/deepchat"; just install-wrapper'

# 仅安装/更新全局 wrapper
install-wrapper:
	@bash -c 'set -e; WRAPPER_SOURCE="scripts/deepchat-wrapper.sh"; WRAPPER_TARGET="$HOME/.cargo/bin/deepchat"; if [ ! -f "$WRAPPER_SOURCE" ]; then echo "错误: 找不到 wrapper 脚本: $WRAPPER_SOURCE"; exit 1; fi; cp "$WRAPPER_SOURCE" "$WRAPPER_TARGET"; chmod +x "$WRAPPER_TARGET"; echo "✓ 已安装全局 wrapper 到 $WRAPPER_TARGET"; echo "现在可以在任何地方使用 deepchat 命令"'

# 生成 crate 文档（仅公共 API）
doc:
	cargo doc --no-deps
	@echo "文档已生成到: target/doc/rust_tui/index.html"
	@echo "提示: 使用 'just doc-all' 可以包含私有项"

# 生成 crate 文档（包含私有项和所有模块）
doc-all:
	cargo doc --document-private-items --no-deps
	@echo "文档已生成到: target/doc/rust_tui/index.html（包含私有项）"

# 生成并打开 crate 文档（WSL 兼容，仅公共 API）
doc-open:
	cargo doc --no-deps
	@bash scripts/open_doc.sh

# 生成并打开 crate 文档（WSL 兼容，包含私有项）
doc-open-all:
	cargo doc --document-private-items --no-deps
	@bash scripts/open_doc.sh

# 统计函数，按行数排序
# 用法: just count-functions [最小行数]
# 示例: just count-functions 30  # 只显示超过30行的函数
count-functions min_lines='0':
	@python3 scripts/count_functions.py {{min_lines}}
