# CodeNothing 性能基准测试套件

这是一个全面的性能基准测试套件，用于比较CodeNothing与其他主流编程语言的执行性能。

## 目录结构

```
benchmarks/
├── tests/           # 测试程序
│   ├── fibonacci_simple.cn    # CodeNothing 斐波那契测试
│   ├── fibonacci_simple.py    # Python 斐波那契测试
│   ├── fibonacci_simple.php   # PHP 斐波那契测试
│   ├── quicksort.cn          # CodeNothing 快速排序测试
│   └── quicksort.py          # Python 快速排序测试
├── scripts/         # 测试脚本
│   ├── run_benchmarks.sh     # 主测试脚本
│   └── generate_report.sh    # 报告生成脚本
├── results/         # 测试结果
│   └── *.txt               # 原始测试数据
└── README.md        # 本文件
```

## 测试项目

### 1. 斐波那契数列计算
- **算法**: 迭代版本
- **测试参数**: 计算第30个斐波那契数
- **测试语言**: CodeNothing, Python, PHP
- **预期结果**: 832040

### 2. 快速排序算法
- **算法**: 递归快速排序
- **测试参数**: 对5000个随机数进行排序
- **测试语言**: CodeNothing, Python
- **验证**: 检查排序结果正确性

## 使用方法

### 前置要求

确保系统已安装以下工具：
- Python 3
- PHP
- Bash
- CodeNothing (Debug和Release版本)

### 运行测试

1. **执行完整基准测试**:
   ```bash
   bash benchmarks/scripts/run_benchmarks.sh
   ```

2. **生成性能报告**:
   ```bash
   bash benchmarks/scripts/generate_report.sh <时间戳>
   ```
   
   例如：
   ```bash
   bash benchmarks/scripts/generate_report.sh 20250731_143444
   ```

### 单独测试

你也可以单独运行各个测试：

```bash
# CodeNothing 测试
./target/debug/CodeNothing benchmarks/tests/fibonacci_simple.cn --cn-time
./target/release/CodeNothing benchmarks/tests/fibonacci_simple.cn --cn-time

# Python 测试
python3 benchmarks/tests/fibonacci_simple.py

# PHP 测试
php benchmarks/tests/fibonacci_simple.php

# 使用time命令测量性能
time python3 benchmarks/tests/fibonacci_simple.py
/usr/bin/time -v python3 benchmarks/tests/fibonacci_simple.py
```

## 测试结果

### 性能指标

测试脚本会收集以下性能指标：

1. **执行时间**:
   - 用户时间 (User time)
   - 系统时间 (System time)
   - 总时间 (Real time)
   - CodeNothing内置时间 (--cn-time)

2. **内存使用**:
   - 最大内存使用量 (Maximum resident set size)
   - 页面错误次数
   - 上下文切换次数

3. **系统资源**:
   - CPU使用率
   - 文件系统I/O
   - 网络I/O

### 结果文件

- **原始数据**: `benchmarks/results/*.txt`
- **汇总报告**: `benchmarks/results/performance_report_<timestamp>.md`
- **详细分析**: `benchmarks/results/performance_summary_<timestamp>.md`

## 测试环境

### 推荐配置

- **操作系统**: Linux (Ubuntu 20.04+)
- **CPU**: 多核处理器
- **内存**: 4GB+
- **存储**: SSD

### 环境信息收集

测试脚本会自动收集以下环境信息：
- 操作系统版本
- CPU型号和频率
- 内存大小
- 编译器版本

## 扩展测试

### 添加新的测试

1. **创建测试文件**:
   在 `benchmarks/tests/` 目录下创建新的测试文件

2. **更新测试脚本**:
   在 `benchmarks/scripts/run_benchmarks.sh` 中添加新的测试调用

3. **更新报告生成**:
   在 `benchmarks/scripts/generate_report.sh` 中添加新的结果处理

### 添加新的语言

1. **实现相同算法**:
   用新语言实现相同的算法逻辑

2. **确保输出一致**:
   验证所有语言版本的输出结果相同

3. **更新测试脚本**:
   添加新语言的测试命令

## 注意事项

### 测试参数调整

- **斐波那契数列**: 递归版本的参数不要设置太大(建议≤25)，否则会导致指数级的计算时间
- **排序算法**: 数组大小根据系统性能调整，避免内存不足
- **测试次数**: 可以多次运行取平均值以获得更准确的结果

### 结果解读

- **时间测量**: 关注用户时间(User time)，它反映了程序实际的CPU使用时间
- **内存使用**: 最大内存使用量反映了程序的内存效率
- **系统开销**: 系统时间反映了系统调用的开销

### 性能影响因素

- 系统负载
- 编译器优化级别
- 运行时环境
- 硬件配置

## 故障排除

### 常见问题

1. **权限错误**: 确保脚本有执行权限
   ```bash
   chmod +x benchmarks/scripts/*.sh
   ```

2. **依赖缺失**: 检查所需的编程语言环境是否安装

3. **内存不足**: 减少测试数据规模

4. **类型错误**: 检查CodeNothing代码的类型声明

### 调试方法

- 使用 `--cn-parser` 选项查看详细的解析信息
- 检查原始测试结果文件中的错误信息
- 单独运行失败的测试以定位问题

---

*最后更新: 2025年 07月 31日*
