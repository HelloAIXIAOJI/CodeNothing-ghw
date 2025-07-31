<?php
/**
 * PHP 数学计算性能测试
 * 对应 CodeNothing v0.5.11 测试
 */

function factorial($n) {
    if ($n <= 1) {
        return 1;
    }
    return $n * factorial($n - 1);
}

function iterativeFactorial($n) {
    $result = 1;
    $i = 1;
    
    while ($i <= $n) {
        $result = $result * $i;
        $i = $i + 1;
    }
    
    return $result;
}

function sumOfSquares($n) {
    $sum = 0;
    $i = 1;
    
    while ($i <= $n) {
        $sum = $sum + ($i * $i);
        $i = $i + 1;
    }
    
    return $sum;
}

function isPrime($n) {
    if ($n < 2) {
        return false;
    }
    if ($n == 2) {
        return true;
    }
    if ($n % 2 == 0) {
        return false;
    }
    
    $i = 3;
    while ($i * $i <= $n) {
        if ($n % $i == 0) {
            return false;
        }
        $i = $i + 2;
    }
    
    return true;
}

function countPrimes($limit) {
    $count = 0;
    $i = 2;
    
    while ($i <= $limit) {
        if (isPrime($i)) {
            $count = $count + 1;
        }
        $i = $i + 1;
    }
    
    return $count;
}

function main() {
    echo "=== PHP 数学计算性能测试 ===\n";
    echo "计算测试:\n";
    
    $n = 12;
    $limit = 1000;
    
    echo "n = " . $n . "\n";
    echo "limit = " . $limit . "\n";
    echo "\n";
    
    // 1. 阶乘计算
    echo "1. 阶乘计算:\n";
    $factorialResult = factorial($n);
    echo "factorial(" . $n . ") = " . $factorialResult . "\n";
    
    $iterativeFactorialResult = iterativeFactorial($n);
    echo "iterativeFactorial(" . $n . ") = " . $iterativeFactorialResult . "\n";
    
    if ($factorialResult == $iterativeFactorialResult) {
        echo "✓ 阶乘计算验证通过\n";
    } else {
        echo "✗ 阶乘计算验证失败\n";
    }
    echo "\n";
    
    // 2. 平方和计算
    echo "2. 平方和计算:\n";
    $sumSquares = sumOfSquares($n);
    echo "sumOfSquares(" . $n . ") = " . $sumSquares . "\n";
    echo "\n";
    
    // 3. 质数计算
    echo "3. 质数计算:\n";
    $primeCount = countPrimes($limit);
    echo "countPrimes(" . $limit . ") = " . $primeCount . "\n";
    echo "\n";
    
    // 4. 质数验证
    echo "4. 质数验证:\n";
    $testPrimes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
    $allCorrect = true;
    
    foreach ($testPrimes as $prime) {
        $result = isPrime($prime);
        echo $prime . " is prime: " . ($result ? "true" : "false") . "\n";
        if (!$result) {
            $allCorrect = false;
        }
    }
    
    if ($allCorrect) {
        echo "✓ 质数验证通过\n";
    } else {
        echo "✗ 质数验证失败\n";
    }
    echo "\n";
    
    echo "=== 数学计算测试完成 ===\n";
    
    return 0;
}

// 执行主函数
main();
?>
