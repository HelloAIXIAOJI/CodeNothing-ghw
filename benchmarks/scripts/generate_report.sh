#!/bin/bash

# 性能测试报告生成脚本

if [ $# -eq 0 ]; then
    echo "用法: $0 <时间戳>"
    echo "例如: $0 20231201_143022"
    exit 1
fi

TIMESTAMP=$1
RESULTS_DIR="benchmarks/results"
REPORT_FILE="$RESULTS_DIR/performance_report_$TIMESTAMP.md"

# 提取时间信息的函数
extract_time() {
    local file=$1
    local pattern=$2
    
    if [ -f "$file" ]; then
        grep "$pattern" "$file" | head -n1 || echo "N/A"
    else
        echo "文件不存在"
    fi
}

# 提取内存信息的函数
extract_memory() {
    local file=$1
    
    if [ -f "$file" ]; then
        grep "Maximum resident set size" "$file" | head -n1 || echo "N/A"
    else
        echo "文件不存在"
    fi
}

# 提取CodeNothing内置时间
extract_cn_time() {
    local file=$1
    
    if [ -f "$file" ]; then
        grep "执行时间:" "$file" | head -n1 || echo "N/A"
    else
        echo "文件不存在"
    fi
}

echo "生成性能测试报告..."

cat > "$REPORT_FILE" << EOF
# CodeNothing 性能基准测试报告

**测试时间**: $(date)  
**测试环境**: $(uname -a)  
**CPU信息**: $(grep "model name" /proc/cpuinfo | head -n1 | cut -d: -f2 | xargs)  
**内存信息**: $(grep MemTotal /proc/meminfo)  

## 测试概述

本报告比较了CodeNothing与其他主流编程语言在相同算法实现下的性能表现。

### 测试项目

1. **斐波那契数列计算** - 递归算法计算第40个斐波那契数
2. **快速排序算法** - 对10000个随机数进行排序

### 测试语言

- CodeNothing (Debug版本)
- CodeNothing (Release版本)  
- Python 3
- PHP
- Shell (Bash)

## 详细测试结果

### 1. 斐波那契数列测试

#### 执行时间对比

| 语言 | 用户时间 | 系统时间 | 总时间 | CodeNothing内置时间 |
|------|----------|----------|--------|-------------------|
EOF

# 斐波那契测试结果
echo "| CodeNothing Debug | $(extract_time "$RESULTS_DIR/codenothing_debug_fibonacci_$TIMESTAMP.txt" "user") | $(extract_time "$RESULTS_DIR/codenothing_debug_fibonacci_$TIMESTAMP.txt" "sys") | $(extract_time "$RESULTS_DIR/codenothing_debug_fibonacci_$TIMESTAMP.txt" "real") | $(extract_cn_time "$RESULTS_DIR/codenothing_debug_fibonacci_$TIMESTAMP.txt") |" >> "$REPORT_FILE"

echo "| CodeNothing Release | $(extract_time "$RESULTS_DIR/codenothing_release_fibonacci_$TIMESTAMP.txt" "user") | $(extract_time "$RESULTS_DIR/codenothing_release_fibonacci_$TIMESTAMP.txt" "sys") | $(extract_time "$RESULTS_DIR/codenothing_release_fibonacci_$TIMESTAMP.txt" "real") | $(extract_cn_time "$RESULTS_DIR/codenothing_release_fibonacci_$TIMESTAMP.txt") |" >> "$REPORT_FILE"

echo "| Python 3 | $(extract_time "$RESULTS_DIR/python_fibonacci_$TIMESTAMP.txt" "user") | $(extract_time "$RESULTS_DIR/python_fibonacci_$TIMESTAMP.txt" "sys") | $(extract_time "$RESULTS_DIR/python_fibonacci_$TIMESTAMP.txt" "real") | N/A |" >> "$REPORT_FILE"

echo "| PHP | $(extract_time "$RESULTS_DIR/php_fibonacci_$TIMESTAMP.txt" "user") | $(extract_time "$RESULTS_DIR/php_fibonacci_$TIMESTAMP.txt" "sys") | $(extract_time "$RESULTS_DIR/php_fibonacci_$TIMESTAMP.txt" "real") | N/A |" >> "$REPORT_FILE"

echo "| Shell | $(extract_time "$RESULTS_DIR/shell_fibonacci_$TIMESTAMP.txt" "user") | $(extract_time "$RESULTS_DIR/shell_fibonacci_$TIMESTAMP.txt" "sys") | $(extract_time "$RESULTS_DIR/shell_fibonacci_$TIMESTAMP.txt" "real") | N/A |" >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << EOF

#### 内存使用对比

| 语言 | 最大内存使用 |
|------|-------------|
EOF

echo "| CodeNothing Debug | $(extract_memory "$RESULTS_DIR/codenothing_debug_fibonacci_$TIMESTAMP.txt") |" >> "$REPORT_FILE"
echo "| CodeNothing Release | $(extract_memory "$RESULTS_DIR/codenothing_release_fibonacci_$TIMESTAMP.txt") |" >> "$REPORT_FILE"
echo "| Python 3 | $(extract_memory "$RESULTS_DIR/python_fibonacci_$TIMESTAMP.txt") |" >> "$REPORT_FILE"
echo "| PHP | $(extract_memory "$RESULTS_DIR/php_fibonacci_$TIMESTAMP.txt") |" >> "$REPORT_FILE"
echo "| Shell | $(extract_memory "$RESULTS_DIR/shell_fibonacci_$TIMESTAMP.txt") |" >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << EOF

### 2. 快速排序测试

#### 执行时间对比

| 语言 | 用户时间 | 系统时间 | 总时间 | CodeNothing内置时间 |
|------|----------|----------|--------|-------------------|
EOF

# 快速排序测试结果
echo "| CodeNothing Debug | $(extract_time "$RESULTS_DIR/codenothing_debug_quicksort_$TIMESTAMP.txt" "user") | $(extract_time "$RESULTS_DIR/codenothing_debug_quicksort_$TIMESTAMP.txt" "sys") | $(extract_time "$RESULTS_DIR/codenothing_debug_quicksort_$TIMESTAMP.txt" "real") | $(extract_cn_time "$RESULTS_DIR/codenothing_debug_quicksort_$TIMESTAMP.txt") |" >> "$REPORT_FILE"

echo "| CodeNothing Release | $(extract_time "$RESULTS_DIR/codenothing_release_quicksort_$TIMESTAMP.txt" "user") | $(extract_time "$RESULTS_DIR/codenothing_release_quicksort_$TIMESTAMP.txt" "sys") | $(extract_time "$RESULTS_DIR/codenothing_release_quicksort_$TIMESTAMP.txt" "real") | $(extract_cn_time "$RESULTS_DIR/codenothing_release_quicksort_$TIMESTAMP.txt") |" >> "$REPORT_FILE"

echo "| Python 3 | $(extract_time "$RESULTS_DIR/python_quicksort_$TIMESTAMP.txt" "user") | $(extract_time "$RESULTS_DIR/python_quicksort_$TIMESTAMP.txt" "sys") | $(extract_time "$RESULTS_DIR/python_quicksort_$TIMESTAMP.txt" "real") | N/A |" >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << EOF

#### 内存使用对比

| 语言 | 最大内存使用 |
|------|-------------|
EOF

echo "| CodeNothing Debug | $(extract_memory "$RESULTS_DIR/codenothing_debug_quicksort_$TIMESTAMP.txt") |" >> "$REPORT_FILE"
echo "| CodeNothing Release | $(extract_memory "$RESULTS_DIR/codenothing_release_quicksort_$TIMESTAMP.txt") |" >> "$REPORT_FILE"
echo "| Python 3 | $(extract_memory "$RESULTS_DIR/python_quicksort_$TIMESTAMP.txt") |" >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << EOF

## 性能分析

### 主要发现

1. **编译优化效果**: Release版本相比Debug版本的性能提升
2. **语言对比**: CodeNothing与其他解释型语言的性能差异
3. **内存效率**: 各语言的内存使用情况对比

### 结论

[基于测试结果的性能分析和结论]

## 测试文件

### 斐波那契数列实现

- [CodeNothing版本](../tests/fibonacci.cn)
- [Python版本](../tests/fibonacci.py)
- [PHP版本](../tests/fibonacci.php)
- [Shell版本](../tests/fibonacci.sh)

### 快速排序实现

- [CodeNothing版本](../tests/quicksort.cn)
- [Python版本](../tests/quicksort.py)

## 原始测试数据

所有原始测试数据文件都保存在 \`benchmarks/results/\` 目录中，文件名包含时间戳 \`$TIMESTAMP\`。

---

*报告生成时间: $(date)*
EOF

echo "✓ 性能测试报告已生成: $REPORT_FILE"
