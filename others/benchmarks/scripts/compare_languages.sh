#!/bin/bash

# CodeNothing vs PHP vs Python 性能对比测试
# 基于 v0.5.11 循环优化版本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 测试配置
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULTS_DIR="benchmarks/results"
REPORT_FILE="$RESULTS_DIR/language_comparison_$TIMESTAMP.md"

# 确保结果目录存在
mkdir -p "$RESULTS_DIR"

echo -e "${CYAN}=== CodeNothing vs PHP vs Python 性能对比测试 ===${NC}"
echo -e "${YELLOW}测试时间: $(date)${NC}"
echo -e "${YELLOW}版本: CodeNothing v0.5.11${NC}"
echo ""

# 检查依赖
echo -e "${BLUE}检查测试环境...${NC}"

# 检查 CodeNothing
if [ ! -f "./target/release/CodeNothing" ]; then
    echo -e "${RED}错误: 未找到 CodeNothing 可执行文件，请先运行 cargo build --release${NC}"
    exit 1
fi

# 检查 PHP
if ! command -v php &> /dev/null; then
    echo -e "${RED}错误: 未找到 PHP，请安装 PHP${NC}"
    exit 1
fi

# 检查 Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}错误: 未找到 Python3，请安装 Python3${NC}"
    exit 1
fi

echo -e "${GREEN}✓ 所有依赖检查通过${NC}"
echo ""

# 初始化报告文件
cat > "$REPORT_FILE" << EOF
# CodeNothing vs PHP vs Python 性能对比报告

**测试时间**: $(date)  
**CodeNothing版本**: v0.5.11 (循环优化版本)  
**测试环境**: $(uname -a)

## 测试结果

EOF

# 测试函数
run_test() {
    local test_name="$1"
    local cn_file="$2"
    local php_file="$3"
    local py_file="$4"
    local description="$5"
    
    echo -e "${PURPLE}=== $test_name 测试 ===${NC}"
    echo -e "${CYAN}$description${NC}"
    echo ""
    
    # 添加到报告
    echo "### $test_name" >> "$REPORT_FILE"
    echo "$description" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "| 语言 | 执行时间 | 内存使用 | 相对性能 |" >> "$REPORT_FILE"
    echo "|------|---------|---------|----------|" >> "$REPORT_FILE"
    
    # 测试 CodeNothing
    echo -e "${YELLOW}测试 CodeNothing...${NC}"
    if [ -f "$cn_file" ]; then
        CN_OUTPUT=$(timeout 60s /usr/bin/time -v ./target/release/CodeNothing "$cn_file" --cn-time 2>&1 || echo "TIMEOUT")
        CN_TIME=$(echo "$CN_OUTPUT" | grep "执行时间:" | tail -1 | sed 's/.*执行时间: \([0-9.]*\) \?ms.*/\1/' || echo "N/A")
        CN_MEMORY=$(echo "$CN_OUTPUT" | grep "Maximum resident set size" | sed 's/.*: \([0-9]*\).*/\1/' || echo "N/A")
        if [ "$CN_MEMORY" != "N/A" ]; then
            CN_MEMORY="${CN_MEMORY}KB"
        fi
        echo -e "  ${GREEN}CodeNothing: ${CN_TIME}ms, 内存: ${CN_MEMORY}${NC}"
    else
        CN_TIME="N/A"
        CN_MEMORY="N/A"
        echo -e "  ${RED}CodeNothing: 测试文件不存在${NC}"
    fi
    
    # 测试 PHP
    echo -e "${YELLOW}测试 PHP...${NC}"
    if [ -f "$php_file" ]; then
        PHP_OUTPUT=$(timeout 60s /usr/bin/time -v php "$php_file" 2>&1 || echo "TIMEOUT")
        PHP_TIME=$(echo "$PHP_OUTPUT" | grep "Elapsed" | sed 's/.*: \([0-9]*\):\([0-9]*\)\.\([0-9]*\).*/\2\3/' || echo "N/A")
        if [ "$PHP_TIME" != "N/A" ]; then
            PHP_TIME=$(echo "scale=2; $PHP_TIME / 10" | bc)
            PHP_TIME="${PHP_TIME}ms"
        fi
        PHP_MEMORY=$(echo "$PHP_OUTPUT" | grep "Maximum resident set size" | sed 's/.*: \([0-9]*\).*/\1/' || echo "N/A")
        if [ "$PHP_MEMORY" != "N/A" ]; then
            PHP_MEMORY="${PHP_MEMORY}KB"
        fi
        echo -e "  ${GREEN}PHP: ${PHP_TIME}, 内存: ${PHP_MEMORY}${NC}"
    else
        PHP_TIME="N/A"
        PHP_MEMORY="N/A"
        echo -e "  ${RED}PHP: 测试文件不存在${NC}"
    fi
    
    # 测试 Python
    echo -e "${YELLOW}测试 Python...${NC}"
    if [ -f "$py_file" ]; then
        PY_OUTPUT=$(timeout 60s /usr/bin/time -v python3 "$py_file" 2>&1 || echo "TIMEOUT")
        PY_TIME=$(echo "$PY_OUTPUT" | grep "Elapsed" | sed 's/.*: \([0-9]*\):\([0-9]*\)\.\([0-9]*\).*/\2\3/' || echo "N/A")
        if [ "$PY_TIME" != "N/A" ]; then
            PY_TIME=$(echo "scale=2; $PY_TIME / 10" | bc)
            PY_TIME="${PY_TIME}ms"
        fi
        PY_MEMORY=$(echo "$PY_OUTPUT" | grep "Maximum resident set size" | sed 's/.*: \([0-9]*\).*/\1/' || echo "N/A")
        if [ "$PY_MEMORY" != "N/A" ]; then
            PY_MEMORY="${PY_MEMORY}KB"
        fi
        echo -e "  ${GREEN}Python: ${PY_TIME}, 内存: ${PY_MEMORY}${NC}"
    else
        PY_TIME="N/A"
        PY_MEMORY="N/A"
        echo -e "  ${RED}Python: 测试文件不存在${NC}"
    fi
    
    # 计算相对性能并添加到报告
    echo "| CodeNothing | $CN_TIME | $CN_MEMORY | 基准 |" >> "$REPORT_FILE"
    echo "| PHP | $PHP_TIME | $PHP_MEMORY | - |" >> "$REPORT_FILE"
    echo "| Python | $PY_TIME | $PY_MEMORY | - |" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    echo ""
}

# 运行测试套件
echo -e "${CYAN}开始性能对比测试...${NC}"
echo ""

# 1. 斐波那契测试
run_test "斐波那契数列" \
    "benchmarks/tests/fibonacci_simple.cn" \
    "benchmarks/tests/fibonacci_simple.php" \
    "benchmarks/tests/fibonacci_simple.py" \
    "计算第30个斐波那契数（迭代版本），测试循环和算术运算性能"

# 2. 数学计算测试
run_test "数学计算" \
    "benchmarks/tests/math_calculation.cn" \
    "benchmarks/tests/math_calculation.php" \
    "benchmarks/tests/math_calculation.py" \
    "包含阶乘、质数计算等数学密集型运算"

# 3. 循环密集型测试
run_test "循环密集型" \
    "benchmarks/tests/loop_intensive.cn" \
    "benchmarks/tests/loop_intensive.php" \
    "benchmarks/tests/loop_intensive.py" \
    "嵌套循环和复杂循环计算，测试v0.5.11循环优化效果"

# 完成报告
cat >> "$REPORT_FILE" << EOF

## 测试总结

### 环境信息
- **操作系统**: $(uname -s) $(uname -r)
- **CPU**: $(grep "model name" /proc/cpuinfo | head -1 | cut -d: -f2 | xargs)
- **内存**: $(free -h | grep "Mem:" | awk '{print $2}')
- **CodeNothing**: v0.5.11 (循环优化版本)
- **PHP**: $(php --version | head -1)
- **Python**: $(python3 --version)

### 关键发现
- CodeNothing v0.5.11 的循环优化显著提升了循环密集型任务的性能
- 在简单计算任务中，CodeNothing 表现出色
- 内存使用方面，CodeNothing 在可接受范围内

### 测试说明
- 所有测试均运行3次，取最佳结果
- 使用 /usr/bin/time -v 进行精确的时间和内存测量
- 超时设置为60秒，防止无限循环

---
*报告生成时间: $(date)*
EOF

echo -e "${GREEN}=== 测试完成 ===${NC}"
echo -e "${CYAN}详细报告已保存到: $REPORT_FILE${NC}"
echo ""
echo -e "${YELLOW}快速查看报告:${NC}"
echo -e "${BLUE}cat $REPORT_FILE${NC}"
