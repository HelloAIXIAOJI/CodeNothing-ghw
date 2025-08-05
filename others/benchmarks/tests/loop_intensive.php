<?php
/**
 * PHP 循环密集型性能测试
 * 对应 CodeNothing v0.5.11 循环优化测试
 */

function nestedLoops($n) {
    $sum = 0;
    $i = 1;
    
    while ($i <= $n) {
        $j = 1;
        while ($j <= $n) {
            $sum = $sum + ($i * $j);
            $j = $j + 1;
        }
        $i = $i + 1;
    }
    
    return $sum;
}

function forLoopSum($n) {
    $sum = 0;
    
    for ($i = 1; $i <= $n; $i++) {
        $sum = $sum + $i;
    }
    
    return $sum;
}

function whileLoopSum($n) {
    $sum = 0;
    $i = 1;
    
    while ($i <= $n) {
        $sum = $sum + $i;
        $i = $i + 1;
    }
    
    return $sum;
}

function arrayIteration($size) {
    // 简化版本：直接计算1到size的和
    $sum = 0;
    $i = 0;
    
    while ($i < $size) {
        $sum = $sum + $i;
        $i = $i + 1;
    }
    
    return $sum;
}

function complexLoopCalculation($n) {
    $result = 0;
    
    for ($i = 1; $i <= $n; $i++) {
        $temp = 0;
        $j = 1;
        
        while ($j <= $i) {
            if ($j % 2 == 0) {
                $temp = $temp + $j * $j;
            } else {
                $temp = $temp + $j;
            }
            $j = $j + 1;
        }
        
        $result = $result + $temp;
    }
    
    return $result;
}

function main() {
    echo "=== PHP 循环密集型性能测试 ===\n";
    
    // 测试参数
    $n = 100;
    $arraySize = 200;
    
    echo "测试参数:\n";
    echo "n = " . $n . "\n";
    echo "arraySize = " . $arraySize . "\n";
    echo "\n";
    
    // 1. 嵌套循环测试
    echo "1. 嵌套循环测试:\n";
    $nestedResult = nestedLoops($n);
    echo "nestedLoops(" . $n . ") = " . $nestedResult . "\n";
    echo "\n";
    
    // 2. for循环测试
    echo "2. for循环测试:\n";
    $forResult = forLoopSum($n);
    echo "forLoopSum(" . $n . ") = " . $forResult . "\n";
    echo "\n";
    
    // 3. while循环测试
    echo "3. while循环测试:\n";
    $whileResult = whileLoopSum($n);
    echo "whileLoopSum(" . $n . ") = " . $whileResult . "\n";
    echo "\n";
    
    // 验证for和while循环结果一致
    if ($forResult == $whileResult) {
        echo "✓ for和while循环结果一致\n";
    } else {
        echo "✗ for和while循环结果不一致\n";
    }
    echo "\n";
    
    // 4. 数组迭代测试
    echo "4. 数组迭代测试:\n";
    $arrayResult = arrayIteration($arraySize);
    echo "arrayIteration(" . $arraySize . ") = " . $arrayResult . "\n";
    echo "\n";
    
    // 5. 复杂循环计算测试
    echo "5. 复杂循环计算测试:\n";
    $complexResult = complexLoopCalculation($n);
    echo "complexLoopCalculation(" . $n . ") = " . $complexResult . "\n";
    echo "\n";
    
    echo "=== 循环密集型测试完成 ===\n";
    
    return 0;
}

// 执行主函数
main();
?>
