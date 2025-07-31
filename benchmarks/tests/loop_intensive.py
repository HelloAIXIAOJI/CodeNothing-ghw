#!/usr/bin/env python3
"""
Python 循环密集型性能测试
测试各种循环的执行效率
"""

def nested_loops(n):
    """嵌套循环测试"""
    sum_val = 0
    i = 1
    
    while i <= n:
        j = 1
        while j <= n:
            sum_val += i * j
            j += 1
        i += 1
    
    return sum_val

def for_loop_sum(n):
    """for循环求和"""
    sum_val = 0
    
    for i in range(1, n + 1):
        sum_val += i
    
    return sum_val

def while_loop_sum(n):
    """while循环求和"""
    sum_val = 0
    i = 1
    
    while i <= n:
        sum_val += i
        i += 1
    
    return sum_val

def array_iteration(size):
    """数组迭代测试"""
    # 创建数组
    arr = []
    i = 0
    
    # 填充数组
    while i < size:
        arr.append(i)
        i += 1
    
    # 计算数组元素和
    sum_val = 0
    j = 0
    while j < len(arr):
        sum_val += arr[j]
        j += 1
    
    return sum_val

def complex_loop_calculation(n):
    """复杂循环计算"""
    result = 0
    
    for i in range(1, n + 1):
        temp = 0
        j = 1
        
        while j <= i:
            if j % 2 == 0:
                temp += j * j
            else:
                temp += j
            j += 1
        
        result += temp
    
    return result

def main():
    print("=== Python 循环密集型性能测试 ===")
    
    # 测试参数
    n = 500
    array_size = 1000
    
    print("测试参数:")
    print(f"n = {n}")
    print(f"array_size = {array_size}")
    print()
    
    # 1. 嵌套循环测试
    print("1. 嵌套循环测试:")
    nested_result = nested_loops(n)
    print(f"nested_loops({n}) = {nested_result}")
    print()
    
    # 2. for循环测试
    print("2. for循环测试:")
    for_result = for_loop_sum(n)
    print(f"for_loop_sum({n}) = {for_result}")
    print()
    
    # 3. while循环测试
    print("3. while循环测试:")
    while_result = while_loop_sum(n)
    print(f"while_loop_sum({n}) = {while_result}")
    print()
    
    # 验证for和while循环结果一致
    if for_result == while_result:
        print("✓ for和while循环结果一致")
    else:
        print("✗ for和while循环结果不一致")
    print()
    
    # 4. 数组迭代测试
    print("4. 数组迭代测试:")
    array_result = array_iteration(array_size)
    print(f"array_iteration({array_size}) = {array_result}")
    print()
    
    # 5. 复杂循环计算测试
    print("5. 复杂循环计算测试:")
    complex_result = complex_loop_calculation(n)
    print(f"complex_loop_calculation({n}) = {complex_result}")
    print()
    
    print("=== 循环密集型测试完成 ===")

if __name__ == "__main__":
    main()
