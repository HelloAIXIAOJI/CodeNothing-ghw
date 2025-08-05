#!/bin/bash

# run_all_tests.sh - 运行所有语句测试

echo "🚀 CodeNothing v0.6.2 语句测试套件"
echo "=================================="

# 确保在正确的目录
cd "$(dirname "$0")/.."

# 编译项目
echo "📦 编译项目..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi

echo "✅ 编译成功"
echo ""

# 测试文件列表
tests=(
    "01_basic_statements.cn"
    "02_control_flow.cn"
    "03_function_calls.cn"
    "04_import_statements.cn"
    "05_arithmetic_statements.cn"
    "06_file_import.cn"
    "07_advanced_loops.cn"
    "08_switch_statement.cn"
    "09_exception_handling.cn"
    "10_class_declaration.cn"
    "11_interface_declaration.cn"
    "12_enum_declaration.cn"
)

# 🚀 v0.6.3 修复：改进错误处理逻辑
passed_tests=0
failed_tests=0

# 运行每个测试
for test in "${tests[@]}"; do
    echo "🧪 运行测试: $test"
    echo "----------------------------------------"

    # 捕获输出和错误码
    output=$(./target/release/CodeNothing "backtest/$test" 2>&1)
    exit_code=$?

    # 显示输出
    echo "$output"

    # 检查是否有解析错误或类型错误
    if echo "$output" | grep -q "发现.*个解析错误\|发现.*个类型错误\|由于存在.*错误，程序无法执行"; then
        echo "❌ $test 测试失败 (发现错误)"
        failed_tests=$((failed_tests + 1))
    elif [ $exit_code -ne 0 ]; then
        echo "❌ $test 测试失败 (退出码: $exit_code)"
        failed_tests=$((failed_tests + 1))
    else
        echo "✅ $test 测试通过"
        passed_tests=$((passed_tests + 1))
    fi

    echo ""
done

# 🚀 v0.6.3 修复：显示测试统计
echo "🎉 所有测试完成！"
echo "=================================="
echo "✅ 通过: $passed_tests"
echo "❌ 失败: $failed_tests"
echo "📊 总计: $((passed_tests + failed_tests))"

if [ $failed_tests -eq 0 ]; then
    echo "🏆 所有测试都通过了！"
    exit 0
else
    echo "⚠️  有 $failed_tests 个测试失败"
    exit 1
fi
