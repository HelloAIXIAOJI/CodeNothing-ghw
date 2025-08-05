# CodeNothing --cn-time 功能实现

## 📋 功能概述

为CodeNothing添加了`--cn-time`命令行标识，用于显示程序运行时间。该功能提供精确的毫秒级时间测量，并根据时间长度自动选择合适的显示格式。

## 🚀 功能特性

### 时间测量范围
- **毫秒级精度**: 使用`std::time::Instant`提供高精度时间测量
- **全程覆盖**: 从文件预处理开始到程序执行完成的完整时间
- **错误情况支持**: 即使解析失败也会显示解析时间

### 智能格式化
根据执行时间长度自动选择最合适的显示格式：

#### 1. 毫秒格式 (< 1秒)
```
执行时间: 123.456 ms
```

#### 2. 秒格式 (1秒 - 1分钟)
```
执行时间: 1500.500 ms [1.5 s]
```

#### 3. 分钟格式 (≥ 1分钟)
```
执行时间: 125000.000 ms [2 min 5.0 s]
```

## 🔧 技术实现

### 1. 时间测量逻辑
```rust
// 开始计时（如果启用了时间显示）
let start_time = if show_time { Some(Instant::now()) } else { None };

// ... 程序执行 ...

// 显示执行时间（如果启用了时间显示）
if let Some(start) = start_time {
    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;
    println!("执行时间: {}", format_execution_time(duration_ms));
}
```

### 2. 格式化函数
```rust
fn format_execution_time(duration_ms: f64) -> String {
    if duration_ms < 1000.0 {
        // 毫秒格式
        format!("{:.3} ms", duration_ms)
    } else if duration_ms < 60000.0 {
        // 秒格式
        let seconds = duration_ms / 1000.0;
        format!("{:.3} ms [{:.1} s]", duration_ms, seconds)
    } else {
        // 分钟格式
        let total_seconds = duration_ms / 1000.0;
        let minutes = (total_seconds / 60.0).floor();
        let seconds = total_seconds % 60.0;
        format!("{:.3} ms [{:.0} min {:.1} s]", duration_ms, minutes, seconds)
    }
}
```

### 3. 参数解析
```rust
let show_time = args.iter().any(|arg| arg == "--cn-time");
```

## 📝 使用示例

### 基本用法
```bash
# 显示执行时间
cargo run -- examples/simple_test.cn --cn-time

# 输出示例:
# === 程序输出 ===
# Hello, World!
# 执行时间: 15.234 ms
```

### 组合使用
```bash
# 同时显示返回值和执行时间
cargo run -- examples/test.cn --cn-return --cn-time

# 输出示例:
# === 程序输出 ===
# 程序执行结果: 42
# 执行时间: 8.567 ms
```

### 解析错误情况
```bash
# 解析失败时也显示时间
cargo run -- examples/syntax_error.cn --cn-time

# 输出示例:
# 发现 1 个解析错误:
# 错误 1: 期望 ';', 但得到了 '.'
# 
# 可以使用 --cn-parser 选项查看更详细的解析信息。
# 由于存在解析错误，程序无法执行。
# 解析时间: 6.123 ms
```

## 🎯 应用场景

### 1. 性能分析
- 测量程序执行效率
- 比较不同实现的性能
- 识别性能瓶颈

### 2. 开发调试
- 验证优化效果
- 监控编译时间变化
- 分析复杂程序的执行时间

### 3. 基准测试
- 建立性能基线
- 回归测试中的性能验证
- 不同版本间的性能对比

## 📊 测试结果

### 格式化测试
```
时间格式化测试:
5.123 ms -> 5.123 ms
123.456 ms -> 123.456 ms
999.999 ms -> 999.999 ms
1000.0 ms -> 1000.000 ms [1.0 s]
1500.5 ms -> 1500.500 ms [1.5 s]
30000.0 ms -> 30000.000 ms [30.0 s]
59999.9 ms -> 59999.900 ms [60.0 s]
60000.0 ms -> 60000.000 ms [1 min 0.0 s]
90000.0 ms -> 90000.000 ms [1 min 30.0 s]
125000.0 ms -> 125000.000 ms [2 min 5.0 s]
3600000.0 ms -> 3600000.000 ms [60 min 0.0 s]
```

### 实际运行测试
```bash
# 简单程序
cargo run -- examples/simple_enhanced_pointer_test.cn --cn-time
# 输出: 执行时间: 32.782 ms

# 复杂程序
cargo run -- examples/pointer_test.cn --cn-time
# 输出: 执行时间: 31.362 ms

# 解析错误
cargo run -- examples/advanced_pointer_syntax_test.cn --cn-time
# 输出: 解析时间: 6.611 ms
```

## 🔄 兼容性

### 向后兼容
- ✅ 不影响现有功能
- ✅ 可选功能，默认不启用
- ✅ 与其他参数完全兼容

### 参数组合
- ✅ `--cn-time` + `--cn-return`: 显示返回值和时间
- ✅ `--cn-time` + `--cn-parser`: 显示解析信息和时间
- ✅ `--cn-time` + `--cn-debug`: 显示调试信息和时间
- ✅ 支持所有现有参数的任意组合

## 🎉 总结

`--cn-time`功能为CodeNothing提供了专业级的时间测量能力，具有以下优势：

1. **精确测量**: 毫秒级精度，覆盖完整执行周期
2. **智能显示**: 根据时间长度自动选择最佳格式
3. **全面支持**: 正常执行和错误情况都支持
4. **易于使用**: 简单的命令行参数，即开即用
5. **完全兼容**: 与现有功能无缝集成

这个功能对于性能分析、开发调试和基准测试都非常有用，是CodeNothing工具链的重要补充。
