#!/bin/bash

# CodeNothing 性能基准测试脚本
# 比较CodeNothing与其他编程语言的执行性能

# 设置颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 获取当前时间戳
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# 创建结果目录
RESULTS_DIR="benchmarks/results"
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}=== CodeNothing 性能基准测试套件 ===${NC}"
echo "测试时间: $(date)"
echo "结果保存目录: $RESULTS_DIR"
echo ""

# 检查必要的工具
echo -e "${YELLOW}检查测试环境...${NC}"

# 检查Python
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version)
    echo "✓ Python: $PYTHON_VERSION"
else
    echo -e "${RED}✗ Python3 未安装${NC}"
    exit 1
fi

# 检查PHP
if command -v php &> /dev/null; then
    PHP_VERSION=$(php --version | head -n1)
    echo "✓ PHP: $PHP_VERSION"
else
    echo -e "${RED}✗ PHP 未安装${NC}"
    exit 1
fi

# 检查Bash
BASH_VERSION=$(bash --version | head -n1)
echo "✓ Bash: $BASH_VERSION"

# 检查CodeNothing
if [ -f "target/debug/CodeNothing" ]; then
    echo "✓ CodeNothing Debug 版本存在"
else
    echo -e "${YELLOW}! CodeNothing Debug 版本不存在，正在编译...${NC}"
    cargo build
fi

if [ -f "target/release/CodeNothing" ]; then
    echo "✓ CodeNothing Release 版本存在"
else
    echo -e "${YELLOW}! CodeNothing Release 版本不存在，正在编译...${NC}"
    cargo build --release
fi

echo ""

# 运行测试的函数
run_test() {
    local test_name=$1
    local command=$2
    local output_file=$3
    
    echo -e "${BLUE}运行 $test_name...${NC}"
    
    # 使用time命令测量时间
    echo "=== $test_name 测试结果 ===" > "$output_file"
    echo "测试时间: $(date)" >> "$output_file"
    echo "命令: $command" >> "$output_file"
    echo "" >> "$output_file"
    
    # 运行测试并记录时间
    echo "--- 程序输出 ---" >> "$output_file"
    /usr/bin/time -v $command >> "$output_file" 2>&1
    echo "" >> "$output_file"
    
    # 简单的time命令
    echo "--- 简单时间测量 ---" >> "$output_file"
    time $command >> "$output_file" 2>&1
    
    echo "✓ $test_name 完成"
}

# 斐波那契数列测试
echo -e "${GREEN}=== 斐波那契数列测试 (简单版本) ===${NC}"

# CodeNothing Debug
run_test "CodeNothing Debug (斐波那契)" \
    "./target/debug/CodeNothing benchmarks/tests/fibonacci_simple.cn --cn-time" \
    "$RESULTS_DIR/codenothing_debug_fibonacci_$TIMESTAMP.txt"

# CodeNothing Release
run_test "CodeNothing Release (斐波那契)" \
    "./target/release/CodeNothing benchmarks/tests/fibonacci_simple.cn --cn-time" \
    "$RESULTS_DIR/codenothing_release_fibonacci_$TIMESTAMP.txt"

# Python
run_test "Python (斐波那契)" \
    "python3 benchmarks/tests/fibonacci_simple.py" \
    "$RESULTS_DIR/python_fibonacci_$TIMESTAMP.txt"

# PHP
run_test "PHP (斐波那契)" \
    "php benchmarks/tests/fibonacci_simple.php" \
    "$RESULTS_DIR/php_fibonacci_$TIMESTAMP.txt"

echo ""

# 数学计算测试
echo -e "${GREEN}=== 数学计算测试 ===${NC}"

# CodeNothing Debug
run_test "CodeNothing Debug (数学计算)" \
    "./target/debug/CodeNothing benchmarks/tests/math_calculation.cn --cn-time" \
    "$RESULTS_DIR/codenothing_debug_math_$TIMESTAMP.txt"

# CodeNothing Release
run_test "CodeNothing Release (数学计算)" \
    "./target/release/CodeNothing benchmarks/tests/math_calculation.cn --cn-time" \
    "$RESULTS_DIR/codenothing_release_math_$TIMESTAMP.txt"

# Python
run_test "Python (数学计算)" \
    "python3 benchmarks/tests/math_calculation.py" \
    "$RESULTS_DIR/python_math_$TIMESTAMP.txt"

echo ""
echo -e "${GREEN}=== 所有测试完成 ===${NC}"
echo "结果文件保存在: $RESULTS_DIR"
echo ""
echo "运行以下命令生成汇总报告:"
echo "bash benchmarks/scripts/generate_report.sh $TIMESTAMP"
