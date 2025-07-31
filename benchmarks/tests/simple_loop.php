<?php
/**
 * PHP 简单循环性能测试
 * 对应 CodeNothing v0.5.11 循环优化测试
 */

function simpleWhileLoop($n) {
    $sum = 0;
    $i = 1;
    
    while ($i <= $n) {
        $sum = $sum + $i;
        $i = $i + 1;
    }
    
    return $sum;
}

function simpleForLoop($n) {
    $sum = 0;
    
    for ($i = 1; $i <= $n; $i++) {
        $sum = $sum + $i;
    }
    
    return $sum;
}

function nestedLoop($n) {
    $sum = 0;
    $i = 1;
    
    while ($i <= $n) {
        $j = 1;
        while ($j <= $n) {
            $sum = $sum + 1;
            $j = $j + 1;
        }
        $i = $i + 1;
    }
    
    return $sum;
}

function main() {
    echo "=== PHP 简单循环性能测试 ===\n";
    
    $n = 1000;
    
    echo "测试参数: n = " . $n . "\n";
    echo "\n";
    
    // while循环测试
    echo "while循环测试:\n";
    $whileResult = simpleWhileLoop($n);
    echo "结果: " . $whileResult . "\n";
    echo "\n";
    
    // for循环测试
    echo "for循环测试:\n";
    $forResult = simpleForLoop($n);
    echo "结果: " . $forResult . "\n";
    echo "\n";
    
    // 嵌套循环测试
    echo "嵌套循环测试:\n";
    $nestedResult = nestedLoop(50);
    echo "结果: " . $nestedResult . "\n";
    echo "\n";
    
    echo "=== 测试完成 ===\n";
    
    return 0;
}

// 执行主函数
main();
?>
