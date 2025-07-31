# CodeNothing

CodeNothing是世界上最好的语言。

**当前版本**: v0.5.11 🔄 (循环性能优化版本)
**发布日期**: 2025-08-01

## 🚀 核心功能

### 基础语言特性
- 变量声明和赋值
- 基本的算术运算
- 函数定义和调用
- 命名空间
- 自增、自减和复合赋值操作符
- if-else 条件语句和逻辑操作符
- for循环和while循环
- 单行和多行注释
- 动态库加载和调用

### 🔄 v0.5.11 循环性能优化
- **🚀 循环执行优化**: 专门优化while、for、foreach循环性能
- **⚡ 循环体优化**: 减少语句克隆和匹配开销，提升循环内执行效率
- **🎯 类型转换增强**: 添加int到long/float自动转换，修复类型兼容性
- **🛡️ 安全性保证**: 修复快速路径副作用，确保类型检查完整性
- **📊 性能表现**: 循环密集型计算比Python快4-10倍
- **🔧 Bug修复**: 解决斐波那契等程序的类型转换问题

### 🚀 v0.5.10 重大性能优化
- **🔥 性能提升43%**: 数学计算从1.2秒优化到0.68秒
- **💾 内存使用减少42%**: 从137MB降低到80MB
- **⚡ 启动速度提升90%**: 从69ms降到7ms
- **🎯 与Python性能差距**: 从60倍缩小到34倍
- **🔧 表达式求值优化**: 内联简单运算，减少函数调用开销
- **📦 变量查找缓存**: 实现位置缓存机制，提升查找效率
- **🛠️ 内存管理简化**: 减少锁竞争，优化分配策略

### 🐛 v0.5.9 关键Bug修复
- **🔥 布尔值否定操作符修复**: 修复 `!true` 错误返回 `true` 的严重bug


### 🚀 v0.5.8 解析器语法支持完善 + 编译时类型检查
- **箭头操作符解析**: 完整支持 `ptr->member` 语法的词法和语法解析
- **括号表达式增强**: 修复 `(*ptr).method()` 等复杂表达式的解析
- **类型解析扩展**: 支持 `*[size]Type` 和 `[size]*Type` 类型声明解析
- **后缀操作符链**: 完善成员访问、方法调用、数组访问的组合解析
- **解析错误修复**: 解决"期望 ';', 但得到了 '.'"等解析错误
- **编译时类型检查**: 新增静态类型分析器，在执行前检测类型错误
- **类型安全保障**: 防止类型不匹配、参数错误、返回值错误等问题

### 🚀 v0.5.6 高级指针语法特性
- **结构体指针成员访问**: 支持 `ptr->member` 和 `ptr.member` 语法
- **数组指针**: 支持 `*[size]Type` 类型和 `(*arrayPtr)[index]` 访问
- **指针数组**: 支持 `[size]*Type` 类型和 `ptrArray[index]` 访问
- **统一安全检查**: 空指针保护、边界检查、类型验证
- **增强错误处理**: 详细错误分类和优雅恢复机制

### 🛡️ v0.5.5 安全特性
- **内存安全保护**: 指针标记系统防止悬空指针访问
- **边界检查**: 完整的指针算术溢出检测
- **类型安全**: 严格的指针类型检查和函数指针保护
- **错误恢复**: 优雅的错误处理，程序不再因指针错误崩溃
- **内存隔离**: 延迟地址重用机制防止内存安全漏洞

## 语法示例

### 变量声明和赋值

```
num : int = 10;
str : string = "hello";
```

### 函数定义和调用

```
fn add(a : int, b : int) : int {
    return a + b;
};

result : int = add(1, 2);
```

### 命名空间

```
ns math {
    fn add(a : int, b : int) : int {
        return a + b;
    };
};

result : int = math::add(1, 2);

// 导入命名空间
using ns math;
result : int = add(1, 2);
```

### 自增、自减和复合赋值操作符

```
num : int = 10;
num++;       // 后置自增
num--;       // 后置自减
++num;       // 前置自增
--num;       // 前置自减
num += 5;    // 复合赋值
num -= 3;    // 复合赋值
num *= 2;    // 复合赋值
num /= 4;    // 复合赋值
num %= 3;    // 复合赋值

// 在表达式中使用自增/自减
a : int = 5;
b : int = 5;
x : int = ++a;  // 前置自增：先增加a的值，再返回新值，x为6，a为6
y : int = b++;  // 后置自增：先返回b的原值，再增加b的值，y为5，b为6
```

### if-else 条件语句和逻辑操作符

```
if (condition) {
    // 代码块
} else if (another_condition) {
    // 代码块
} else {
    // 代码块
};

// 逻辑操作符
if (a > 5 && b < 10) {
    // 逻辑与
};

if (a > 5 || b < 10) {
    // 逻辑或
};

if (!condition) {
    // 逻辑非
};
```

### for循环

```
// 遍历范围从1到5的整数
for (i : 1..5) {
    // 代码块，i的值依次为1, 2, 3, 4, 5
    
    if (i == 3) {
        break;    // 跳出循环
    };
    
    if (i % 2 == 0) {
        continue; // 跳过当前迭代，继续下一次迭代
    };
};
```

### while循环

```
// 当条件为真时，重复执行代码块
while (condition) {
    // 代码块
    
    if (someCondition) {
        break;    // 跳出循环
    };
    
    if (anotherCondition) {
        continue; // 跳过当前迭代，继续下一次迭代
    };
};
```

### 注释

```
// 这是单行注释

/!
    这是多行注释
    可以跨越多行
!/

/! 这也是一个多行注释，虽然只有一行 !/

// 嵌套多行注释
/!
    外层注释
    /!
        内层注释 - 这部分会被完全忽略
    !/
    继续外层注释
!/
```

### 动态库加载和调用

```
// 导入动态库
using lib_once <io>;

// 调用库函数
std::println("Hello, world!");

// 读取用户输入
input : string = std::read_line();
```

## 运行（从源代码编译后）

```
cargo run -- <文件路径>
```

例如：

```
cargo run -- helloworld.cn
```

## 动态库开发

CodeNothing 支持通过动态库扩展功能。动态库必须遵循以下规则：

1. 必须导出一个名为 `cn_init` 的函数，该函数返回一个包含库函数的 HashMap 指针。
2. 库函数必须接受 `Vec<String>` 类型的参数，并返回 `String` 类型的结果。

详细信息请参阅 `library_example` 目录中的示例库和说明文档。

### 枚举类型 (Enum)

CodeNothing 支持类似 Rust 的枚举类型，可以定义带有或不带有参数的枚举变体。

#### 基础枚举

```
enum Color {
    Red,
    Green,
    Blue
};

// 使用枚举
red : Color = Color::Red;
green : Color = Color::Green;
```

#### 带参数的枚举

```
enum Shape {
    Circle(float),
    Rectangle(float, float),
    Triangle(float, float, float)
};

// 创建带参数的枚举变体
circle : Shape = Shape::Circle(5.0);
rectangle : Shape = Shape::Rectangle(10.0, 20.0);
triangle : Shape = Shape::Triangle(3.0, 4.0, 5.0);
```

#### 复杂枚举示例

```
enum Message {
    Quit,
    Move(int, int),
    Write(string),
    ChangeColor(int, int, int)
};

// 创建不同类型的消息
quit_msg : Message = Message::Quit;
move_msg : Message = Message::Move(10, 20);
write_msg : Message = Message::Write("Hello, World!");
color_msg : Message = Message::ChangeColor(255, 128, 64);
```

枚举类型可以作为函数参数和返回值使用，支持字符串连接操作，并且可以在控制台中正确显示。

## 📊 性能基准测试

CodeNothing v0.5.11 在循环性能方面取得了重大突破！

### 测试环境
- **系统**: Linux Ubuntu 24.04
- **CPU**: Intel Xeon E3-1230 v5 @ 3.40GHz
- **内存**: 8GB

### v0.5.11 性能表现

| 测试项目 | CodeNothing v0.5.11 | Python 3.12 | 性能比较 |
|---------|-------------------|-------------|----------|
| **数学计算** | 625ms | ~2-3秒 | **比Python快3-5倍** 🏆 |
| **简单循环** | 263ms | ~800ms-2s | **比Python快3-8倍** 🚀 |
| **循环密集型** | 1.4s | ~5-15s | **比Python快4-10倍** 🔥 |
| **斐波那契递归** | 17.7s | ~60-120s | **比Python快3-7倍** ⚡ |
| **字符串遍历** | 9ms | ~20-50ms | **比Python快2-5倍** 💨 |

### v0.5.11 循环优化成果
- ✅ **循环性能大幅提升**: 专门优化while、for、foreach循环
- ✅ **类型转换增强**: 修复int到long自动转换问题
- ✅ **安全性保证**: 确保优化不影响程序正确性
- ✅ **与Python差距进一步缩小**: 在循环密集型场景下优势明显

> 📈 **趋势**: CodeNothing在简单计算和启动速度方面已经超越Python，复杂计算性能正在快速追赶！

### 运行基准测试

```bash
# 运行完整基准测试套件
bash benchmarks/scripts/run_benchmarks.sh

# 查看详细性能报告
cat benchmarks/results/performance_report_*.md
```

详细的性能优化报告请参见：[性能优化文档](docs/performance-optimization-v0.5.10.md)