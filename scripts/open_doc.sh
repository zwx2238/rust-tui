#!/bin/bash
set -e

DOC_PATH="target/doc/rust_tui/index.html"

if [ ! -f "$DOC_PATH" ]; then
    echo "错误: 文档文件不存在: $DOC_PATH"
    echo "当前目录: $(pwd)"
    ls -la target/doc/rust_tui/ 2>&1 | head -5 || true
    exit 1
fi

ABS_PATH=$(realpath "$DOC_PATH")

echo "✓ 文档已生成"
echo ""
echo "📖 使用提示："
echo "  - 左侧边栏可以按模块浏览（config, conversation, render, types）"
echo "  - 点击模块名称可以查看该模块下的所有类型和函数"
echo "  - 使用顶部的搜索框可以快速查找"
echo ""

if command -v wslview >/dev/null 2>&1; then
    wslview "$ABS_PATH"
elif command -v explorer.exe >/dev/null 2>&1; then
    explorer.exe "$(wslpath -w "$ABS_PATH")"
else
    echo "文档路径: $ABS_PATH"
    echo "请手动在浏览器中打开该文件"
fi
