#!/usr/bin/env python3
"""
统计 Rust 代码中的函数，按行数排序
"""

import re
import sys
import os
from pathlib import Path
from collections import namedtuple

Function = namedtuple('Function', ['name', 'file', 'start_line', 'end_line', 'lines'])

def find_functions_in_file(file_path):
    """在文件中查找所有函数定义"""
    functions = []
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            lines = content.split('\n')
    except Exception as e:
        print(f"警告: 无法读取文件 {file_path}: {e}", file=sys.stderr)
        return functions
    
    # 匹配函数定义的正则表达式
    # 匹配: pub fn, pub(crate) fn, fn, pub async fn, async fn, unsafe fn 等
    # 支持: pub, pub(crate), pub(super), pub(in path), unsafe, async 等修饰符
    func_pattern = re.compile(r'^\s*(pub(\([^)]+\))?\s+|unsafe\s+)?(async\s+)?fn\s+(\w+)')
    
    i = 0
    while i < len(lines):
        line = lines[i]
        match = func_pattern.match(line)
        
        if match:
            func_name = match.group(4)  # 函数名是第4个分组（因为增加了 pub(...) 的匹配）
            func_start_line = i + 1  # 函数定义行号（从1开始）
            
            # 找到函数体的开始（第一个 {）
            # 需要跳过字符串和注释中的 {
            brace_start_line = None
            in_string = False
            string_char = None
            in_comment = False
            
            # 从函数定义行开始查找第一个真正的 {
            for j in range(i, len(lines)):
                current_line = lines[j]
                k = 0
                while k < len(current_line):
                    char = current_line[k]
                    
                    # 处理字符串
                    if not in_string and not in_comment:
                        if char == '"':
                            in_string = True
                            string_char = char
                        elif char == '/' and k + 1 < len(current_line):
                            if current_line[k+1] == '/':
                                # 行注释，跳过这一行剩余部分
                                break
                            elif current_line[k+1] == '*':
                                # 块注释开始
                                in_comment = True
                                k += 1
                        elif char == '{':
                            # 找到函数体开始
                            brace_start_line = j + 1
                            break
                    elif in_string:
                        if char == string_char and (k == 0 or current_line[k-1] != '\\'):
                            in_string = False
                            string_char = None
                    elif in_comment:
                        if char == '*' and k + 1 < len(current_line) and current_line[k+1] == '/':
                            in_comment = False
                            k += 1
                    
                    k += 1
                
                if brace_start_line is not None:
                    break
            
            if brace_start_line is None:
                i += 1
                continue
            
            # 计算大括号匹配来找到函数结束位置
            brace_count = 0
            end_line = brace_start_line
            in_string = False
            string_char = None
            in_comment = False
            
            for j in range(brace_start_line - 1, len(lines)):  # -1 因为行号从1开始
                current_line = lines[j]
                k = 0
                while k < len(current_line):
                    char = current_line[k]
                    
                    # 处理字符串和注释
                    if not in_string and not in_comment:
                        if char == '"':
                            in_string = True
                            string_char = char
                        elif char == '/' and k + 1 < len(current_line):
                            if current_line[k+1] == '/':
                                break  # 行注释
                            elif current_line[k+1] == '*':
                                in_comment = True
                                k += 1
                        elif char == '{':
                            brace_count += 1
                        elif char == '}':
                            brace_count -= 1
                            if brace_count == 0:
                                end_line = j + 1
                                lines_count = end_line - func_start_line + 1
                                functions.append(Function(
                                    name=func_name,
                                    file=str(file_path),
                                    start_line=func_start_line,
                                    end_line=end_line,
                                    lines=lines_count
                                ))
                                i = j
                                break
                    elif in_string:
                        if char == string_char and (k == 0 or current_line[k-1] != '\\'):
                            in_string = False
                            string_char = None
                    elif in_comment:
                        if char == '*' and k + 1 < len(current_line) and current_line[k+1] == '/':
                            in_comment = False
                            k += 1
                    
                    k += 1
                
                if brace_count == 0:
                    break
            
        i += 1
    
    return functions

def main():
    min_lines = 0
    if len(sys.argv) > 1:
        try:
            min_lines = int(sys.argv[1])
        except ValueError:
            print(f"错误: 无效的行数参数: {sys.argv[1]}", file=sys.stderr)
            sys.exit(1)
    
    src_dir = Path('src')
    if not src_dir.exists():
        print(f"错误: 找不到 src 目录", file=sys.stderr)
        sys.exit(1)
    
    all_functions = []
    
    # 遍历所有 .rs 文件
    for rs_file in src_dir.rglob('*.rs'):
        functions = find_functions_in_file(rs_file)
        all_functions.extend(functions)
    
    # 过滤
    if min_lines > 0:
        all_functions = [f for f in all_functions if f.lines > min_lines]
    
    # 按行数排序（降序）
    all_functions.sort(key=lambda x: x.lines, reverse=True)
    
    # 输出
    print(f"{'函数名':<30} | {'完整路径':<60} | {'行号范围':<15} | {'行数':>6}")
    print("-" * 120)
    
    for func in all_functions:
        range_str = f"{func.start_line}-{func.end_line}"
        print(f"{func.name:<30} | {func.file:<60} | {range_str:<15} | {func.lines:>6}")

if __name__ == '__main__':
    main()
