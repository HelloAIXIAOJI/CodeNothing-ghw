# CodeNothing 构建指南

## 🚀 快速开始

### 基本构建
```bash
# 标准发布版本（推荐）
cargo build --release

# 开发版本（包含调试信息）
cargo build
```

### 运行程序
```bash
# 运行CodeNothing程序
./target/release/CodeNothing your_program.cn

# 查看帮助信息
./target/release/CodeNothing --help
```

## 🔧 构建选项

### Features 功能开关

#### rwlock-stats - 读写锁性能统计
```bash
# 启用读写锁性能监控（开发/调试用）
cargo build --release --features rwlock-stats

# 运行时查看统计信息
./target/release/CodeNothing program.cn --cn-rwlock
```

**注意**: `rwlock-stats` feature 会引入轻微的性能开销，仅建议在开发和性能分析时使用。

### 命令行参数

#### 基本选项
- `--cn-parser`: 显示详细的解析信息
- `--cn-lexer`: 显示词法分析信息  
- `--cn-debug`: 启用调试模式
- `--cn-return`: 显示程序执行结果
- `--cn-time`: 显示程序执行时间

#### v0.6.2 新增选项
- `--cn-rwlock`: 🚀 显示读写锁性能统计（需要 `rwlock-stats` feature）

#### 组合使用示例
```bash
# 完整性能分析
./target/release/CodeNothing program.cn --cn-time --cn-rwlock

# 调试模式
./target/release/CodeNothing program.cn --cn-debug --cn-parser
```

## 🛠️ 开发环境设置

### 推荐的开发工具链
```bash
# Rust 版本要求
rustc --version  # 建议 1.70+

# 有用的开发命令
cargo check      # 快速语法检查
cargo clippy     # 代码质量检查
cargo fmt        # 代码格式化
```

### 调试构建
```bash
# 调试版本（包含更多调试信息）
cargo build

# 运行调试版本
./target/debug/CodeNothing program.cn --cn-debug
```

## 📚 更多信息

- 查看 `CHANGELOG.md` 了解版本更新内容
- 查看 `TODOLOG.md` 了解开发路线图
- 性能问题请参考本文档的性能测试指南

---

**提示**: 如果在特定场景下性能没有预期提升，请检查是否使用了适合的测试程序。读写锁优化主要针对内存操作，而不是IO操作。
