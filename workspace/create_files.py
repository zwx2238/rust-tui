#!/usr/bin/env python3
"""
生成10个示例文件的脚本
"""

import os
import json
import csv
from datetime import datetime

def create_text_file(filename, content):
    """创建文本文件"""
    with open(filename, 'w', encoding='utf-8') as f:
        f.write(content)
    print(f"✓ 已创建: {filename}")

def main():
    print("开始生成10个文件...")
    print("-" * 40)
    
    # 创建10个文本文件
    for i in range(1, 11):
        filename = f"file_{i:02d}.txt"
        content = f"""这是第 {i} 个示例文件
创建时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
文件编号: {i:02d}
内容: 这是一个自动生成的示例文件，用于演示文件创建功能。
"""
        create_text_file(filename, content)
    
