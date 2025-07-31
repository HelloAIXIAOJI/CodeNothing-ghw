<?php
/**
 * PHP 简单斐波那契数列性能测试
 * 只测试迭代版本，计算第1000个斐波那契数
 */

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
    echo "=== PHP 简单斐波那契数列性能测试 ===\n";
    
    // 测试参数
    $n = 1000;
    
    echo "计算第 {$n} 个斐波那契数 (迭代版本)\n";
    
    // 迭代版本测试
    $result = iterativeFibonacci($n);
    echo "iterativeFibonacci({$n}) = {$result}\n";
    
    echo "✓ 测试完成\n";
}

main();
?>
