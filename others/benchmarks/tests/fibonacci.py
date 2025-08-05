#!/usr/bin/env python3
"""
Python 斐波那契数列性能测试
计算第25个斐波那契数
"""

def fibonacci(n):
    """递归版本的斐波那契数列"""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def iterative_fibonacci(n):
    """迭代版本的斐波那契数列"""
    if n <= 1:
        return n
    
    a, b = 0, 1
    for i in range(2, n + 1):
        a, b = b, a + b
    
    return b

def main():
    print("=== Python 斐波那契数列性能测试 ===")
    
    # 测试参数
    n = 25
    
    print(f"计算第 {n} 个斐波那契数")
    print()
    
    # 递归版本测试
    print("递归版本:")
    result1 = fibonacci(n)
    print(f"fibonacci({n}) = {result1}")
    print()
    
    # 迭代版本测试
    print("迭代版本:")
    result2 = iterative_fibonacci(n)
    print(f"iterative_fibonacci({n}) = {result2}")
    print()
    
    # 验证结果一致性
    if result1 == result2:
        print("✓ 结果验证通过")
    else:
        print("✗ 结果验证失败")

if __name__ == "__main__":
    main()
