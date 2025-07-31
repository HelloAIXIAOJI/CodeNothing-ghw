<?php
/**
 * PHP 斐波那契数列性能测试
 * 计算第25个斐波那契数
 */

function fibonacci($n) {
    if ($n <= 1) {
        return $n;
    }
    return fibonacci($n - 1) + fibonacci($n - 2);
}

function iterativeFibonacci($n) {
    if ($n <= 1) {
        return $n;
    }
    
    $a = 0;
    $b = 1;
    
    for ($i = 2; $i <= $n; $i++) {
        $temp = $a + $b;
        $a = $b;
        $b = $temp;
    }
    
    return $b;
}

function main() {
    echo "=== PHP 斐波那契数列性能测试 ===\n";
    
    // 测试参数
    $n = 25;
    
    echo "计算第 {$n} 个斐波那契数\n";
    echo "\n";
    
    // 递归版本测试
    echo "递归版本:\n";
    $result1 = fibonacci($n);
    echo "fibonacci({$n}) = {$result1}\n";
    echo "\n";
    
    // 迭代版本测试
    echo "迭代版本:\n";
    $result2 = iterativeFibonacci($n);
    echo "iterativeFibonacci({$n}) = {$result2}\n";
    echo "\n";
    
    // 验证结果一致性
    if ($result1 == $result2) {
        echo "✓ 结果验证通过\n";
    } else {
        echo "✗ 结果验证失败\n";
    }
}

main();
?>
