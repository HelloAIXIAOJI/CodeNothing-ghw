# CodeNothing 性能基准测试报告

**测试时间**: 2025年 08月 01日 星期五 02:28:22 CST  
**测试环境**: Linux aixiaoji-Ubuntu-VMware-Virtual-Platform 6.14.0-27-generic #27~24.04.1-Ubuntu SMP PREEMPT_DYNAMIC Tue Jul 22 17:38:49 UTC 2 x86_64 x86_64 x86_64 GNU/Linux  
**CPU信息**: Intel(R) Xeon(R) CPU E3-1230 v5 @ 3.40GHz  
**内存信息**: MemTotal:        8083156 kB  

## 测试概述

本报告比较了CodeNothing与其他主流编程语言在相同算法实现下的性能表现。

### 测试项目

1. **斐波那契数列计算** - 递归算法计算第40个斐波那契数
2. **快速排序算法** - 对10000个随机数进行排序

### 测试语言

- CodeNothing (Debug版本)
- CodeNothing (Release版本)  
- Python 3
- PHP
- Shell (Bash)

## 详细测试结果

### 1. 斐波那契数列测试

#### 执行时间对比

| 语言 | 用户时间 | 系统时间 | 总时间 | CodeNothing内置时间 |
|------|----------|----------|--------|-------------------|
| CodeNothing Debug |  | 	File system inputs: 14256 |  | 执行时间: 516.875 ms |
| CodeNothing Release |  | 	File system inputs: 944 |  | 执行时间: 10.932 ms |
| Python 3 |  | 	File system inputs: 2032 |  | N/A |
| PHP |  | 	File system inputs: 424 |  | N/A |
| Shell | 文件不存在 | 文件不存在 | 文件不存在 | N/A |

#### 内存使用对比

| 语言 | 最大内存使用 |
|------|-------------|
| CodeNothing Debug | 	Maximum resident set size (kbytes): 10352 |
| CodeNothing Release | 	Maximum resident set size (kbytes): 6292 |
| Python 3 | 	Maximum resident set size (kbytes): 9552 |
| PHP | 	Maximum resident set size (kbytes): 19492 |
| Shell | 文件不存在 |

### 2. 快速排序测试

#### 执行时间对比

| 语言 | 用户时间 | 系统时间 | 总时间 | CodeNothing内置时间 |
|------|----------|----------|--------|-------------------|
| CodeNothing Debug | 文件不存在 | 文件不存在 | 文件不存在 | 文件不存在 |
| CodeNothing Release | 文件不存在 | 文件不存在 | 文件不存在 | 文件不存在 |
| Python 3 | 文件不存在 | 文件不存在 | 文件不存在 | N/A |

#### 内存使用对比

| 语言 | 最大内存使用 |
|------|-------------|
| CodeNothing Debug | 文件不存在 |
| CodeNothing Release | 文件不存在 |
| Python 3 | 文件不存在 |

## 性能分析

### 主要发现

1. **编译优化效果**: Release版本相比Debug版本的性能提升
2. **语言对比**: CodeNothing与其他解释型语言的性能差异
3. **内存效率**: 各语言的内存使用情况对比

### 结论

[基于测试结果的性能分析和结论]

## 测试文件

### 斐波那契数列实现

- [CodeNothing版本](../tests/fibonacci.cn)
- [Python版本](../tests/fibonacci.py)
- [PHP版本](../tests/fibonacci.php)
- [Shell版本](../tests/fibonacci.sh)

### 快速排序实现

- [CodeNothing版本](../tests/quicksort.cn)
- [Python版本](../tests/quicksort.py)

## 原始测试数据

所有原始测试数据文件都保存在 `benchmarks/results/` 目录中，文件名包含时间戳 `20250801_022719`。

---

*报告生成时间: 2025年 08月 01日 星期五 02:28:22 CST*
