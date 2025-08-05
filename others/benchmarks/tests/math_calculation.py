#!/usr/bin/env python3
"""
Python 数学计算性能测试
计算大量数学运算
"""

def factorial(n):
    """递归阶乘"""
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def iterative_factorial(n):
    """迭代阶乘"""
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result

def sum_of_squares(n):
    """平方和计算"""
    return sum(i * i for i in range(1, n + 1))

def is_prime(n):
    """质数判断"""
    if n <= 1:
        return False
    if n <= 3:
        return True
    if n % 2 == 0 or n % 3 == 0:
        return False
    
    i = 5
    while i * i <= n:
        if n % i == 0 or n % (i + 2) == 0:
            return False
        i += 6
    
    return True

def count_primes(limit):
    """计算质数个数"""
    count = 0
    for i in range(2, limit + 1):
        if is_prime(i):
            count += 1
    return count

def main():
    print("=== Python 数学计算性能测试 ===")
    
    # 测试参数
    n = 12
    limit = 1000
    
    print("计算测试:")
    print(f"n = {n}")
    print(f"limit = {limit}")
    print()
    
    # 阶乘计算
    print("1. 阶乘计算:")
    fact1 = factorial(n)
    fact2 = iterative_factorial(n)
    print(f"factorial({n}) = {fact1}")
    print(f"iterative_factorial({n}) = {fact2}")
    
    if fact1 == fact2:
        print("✓ 阶乘计算验证通过")
    else:
        print("✗ 阶乘计算验证失败")
    print()
    
    # 平方和计算
    print("2. 平方和计算:")
    squares = sum_of_squares(n)
    print(f"sum_of_squares({n}) = {squares}")
    print()
    
    # 质数计算
    print("3. 质数计算:")
    prime_count = count_primes(limit)
    print(f"count_primes({limit}) = {prime_count}")
    print()
    
    # 验证一些已知的质数
    print("4. 质数验证:")
    test_numbers = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
    all_correct = True
    
    for num in test_numbers:
        is_p = is_prime(num)
        print(f"{num} is prime: {is_p}")
        if not is_p:
            all_correct = False
    
    if all_correct:
        print("✓ 质数验证通过")
    else:
        print("✗ 质数验证失败")
    
    print()
    print("=== 数学计算测试完成 ===")

if __name__ == "__main__":
    main()
