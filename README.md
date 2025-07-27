# CodeNothing

CodeNothing是世界上最好的语言。

## 功能

- 变量声明和赋值
- 基本的算术运算
- 函数定义和调用
- 命名空间
- 自增、自减和复合赋值操作符
- if-else 条件语句和逻辑操作符
- for循环和while循环
- 单行和多行注释
- 动态库加载和调用

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