#!/usr/bin/env python3
"""
Python 快速排序性能测试
对5000个随机数进行排序
"""

def quick_sort(arr, low, high):
    """快速排序算法"""
    if low < high:
        pi = partition(arr, low, high)
        quick_sort(arr, low, pi - 1)
        quick_sort(arr, pi + 1, high)

def partition(arr, low, high):
    """分区函数"""
    pivot = arr[high]
    i = low - 1
    
    for j in range(low, high):
        if arr[j] <= pivot:
            i += 1
            arr[i], arr[j] = arr[j], arr[i]
    
    arr[i + 1], arr[high] = arr[high], arr[i + 1]
    return i + 1

def generate_random_array(size):
    """生成随机数组"""
    arr = []
    for i in range(size):
        # 简单的伪随机数生成
        value = (i * 17 + 23) % 1000
        arr.append(value)
    return arr

def is_sorted(arr):
    """检查数组是否已排序"""
    for i in range(1, len(arr)):
        if arr[i - 1] > arr[i]:
            return False
    return True

def main():
    print("=== Python 快速排序性能测试 ===")
    
    # 测试参数
    size = 5000
    
    print(f"生成 {size} 个随机数进行排序")
    
    # 生成随机数组
    arr = generate_random_array(size)
    print("数组生成完成")
    
    # 显示前10个元素
    print("排序前前10个元素:")
    for i in range(min(10, len(arr))):
        print(f"arr[{i}] = {arr[i]}")
    
    # 执行快速排序
    print()
    print("开始快速排序...")
    quick_sort(arr, 0, len(arr) - 1)
    print("排序完成")
    
    # 显示排序后前10个元素
    print()
    print("排序后前10个元素:")
    for i in range(min(10, len(arr))):
        print(f"arr[{i}] = {arr[i]}")
    
    # 验证排序结果
    print()
    if is_sorted(arr):
        print("✓ 排序验证通过")
    else:
        print("✗ 排序验证失败")

if __name__ == "__main__":
    main()
