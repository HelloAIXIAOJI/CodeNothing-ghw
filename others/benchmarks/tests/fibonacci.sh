#!/bin/bash
# Shell 斐波那契数列性能测试
# 计算第25个斐波那契数

fibonacci() {
    local n=$1
    if [ $n -le 1 ]; then
        echo $n
        return
    fi
    
    local prev1=$(fibonacci $((n - 1)))
    local prev2=$(fibonacci $((n - 2)))
    echo $((prev1 + prev2))
}

iterative_fibonacci() {
    local n=$1
    if [ $n -le 1 ]; then
        echo $n
        return
    fi
    
    local a=0
    local b=1
    local i=2
    
    while [ $i -le $n ]; do
        local temp=$((a + b))
        a=$b
        b=$temp
        i=$((i + 1))
    done
    
    echo $b
}

main() {
    echo "=== Shell 斐波那契数列性能测试 ==="
    
    # 测试参数
    n=25
    
    echo "计算第 $n 个斐波那契数"
    echo ""
    
    # 递归版本测试
    echo "递归版本:"
    result1=$(fibonacci $n)
    echo "fibonacci($n) = $result1"
    echo ""
    
    # 迭代版本测试
    echo "迭代版本:"
    result2=$(iterative_fibonacci $n)
    echo "iterative_fibonacci($n) = $result2"
    echo ""
    
    # 验证结果一致性
    if [ "$result1" = "$result2" ]; then
        echo "✓ 结果验证通过"
    else
        echo "✗ 结果验证失败"
    fi
}

main
