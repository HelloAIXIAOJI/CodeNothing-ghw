#!/usr/bin/env python3
"""
Python 简单循环性能测试
对应 CodeNothing v0.5.11 循环优化测试
"""

def simple_while_loop(n):
    """简单while循环测试"""
    sum_val = 0
    i = 1
    
    while i <= n:
        sum_val = sum_val + i
        i = i + 1
    
    return sum_val

def simple_for_loop(n):
    """简单for循环测试"""
    sum_val = 0
    
    for i in range(1, n + 1):
        sum_val = sum_val + i
    
    return sum_val

def nested_loop(n):
    """嵌套循环测试"""
    sum_val = 0
    i = 1
    
    while i <= n:
        j = 1
        while j <= n:
            sum_val = sum_val + 1
            j = j + 1
        i = i + 1
    
    return sum_val

def main():
    print("=== Python 简单循环性能测试 ===")
    
    n = 1000
    
    print(f"测试参数: n = {n}")
    print("")
    
    # while循环测试
    print("while循环测试:")
    while_result = simple_while_loop(n)
    print(f"结果: {while_result}")
    print("")
    
    # for循环测试
    print("for循环测试:")
    for_result = simple_for_loop(n)
    print(f"结果: {for_result}")
    print("")
    
    # 嵌套循环测试
    print("嵌套循环测试:")
    nested_result = nested_loop(50)
    print(f"结果: {nested_result}")
    print("")
    
    print("=== 测试完成 ===")
    
    return 0

if __name__ == "__main__":
    main()
