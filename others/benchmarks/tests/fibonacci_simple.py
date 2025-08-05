#!/usr/bin/env python3
"""
Python 简单斐波那契数列性能测试
只测试迭代版本，计算第1000个斐波那契数
"""

def iterative_fibonacci(n):
    """迭代版本的斐波那契数列"""
    if n <= 1:
        return n
    
    a, b = 0, 1
    for i in range(2, n + 1):
        a, b = b, a + b
    
    return b

def main():
    print("=== Python 简单斐波那契数列性能测试 ===")
    
    # 测试参数
    n = 30
    
    print(f"计算第 {n} 个斐波那契数 (迭代版本)")
    
    # 迭代版本测试
    result = iterative_fibonacci(n)
    print(f"iterative_fibonacci({n}) = {result}")
    
    print("✓ 测试完成")

if __name__ == "__main__":
    main()
