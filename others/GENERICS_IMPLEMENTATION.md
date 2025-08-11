# CodeNothing 泛型

## 概述

本文档描述了 CodeNothing 编程语言中泛型系统的实现。泛型系统允许编写可重用的代码，支持类型参数化的函数、类、枚举和接口。

## CodeNothing 现有的类型系统

CodeNothing 已经具备了强大的类型系统，包括：

### 现有的变量声明语法
- **显式类型声明**：`name: type = value`
- **自动类型推断**：`name: auto = value`

### 示例
```codenothing
// 显式类型声明
x: int = 42;
y: float = 3.14;
z: string = "Hello";

// 自动类型推断
a: auto = 100;      // 推断为 int
b: auto = 2.71;     // 推断为 float
c: auto = "World";  // 推断为 string
```

## 已实现的功能

### 1. AST 扩展

#### 泛型参数 (GenericParameter)
```rust
pub struct GenericParameter {
    pub name: String,                           // 类型参数名 (如 T, U, K)
    pub constraints: Vec<TypeConstraint>,       // 类型约束
    pub default_type: Option<Type>,            // 默认类型
}
```

#### 类型约束 (TypeConstraint)
```rust
pub enum TypeConstraint {
    Trait(String),      // trait 约束
    Sized,              // Sized 约束
    Copy,               // Copy 约束
    Send,               // Send 约束
    Sync,               // Sync 约束
}
```

#### 泛型类型 (Type 扩展)
```rust
pub enum Type {
    // ... 现有类型 ...
    Generic(String),                           // 泛型类型参数 T
    GenericClass(String, Vec<Type>),          // 泛型类 Container<T>
    GenericEnum(String, Vec<Type>),           // 泛型枚举 Option<T>
}
```

#### 泛型表达式 (Expression 扩展)
```rust
pub enum Expression {
    // ... 现有表达式 ...
    GenericFunctionCall(String, Vec<Type>, Vec<Expression>),     // 泛型函数调用
    GenericMethodCall(Box<Expression>, String, Vec<Type>, Vec<Expression>), // 泛型方法调用
    GenericObjectCreation(String, Vec<Type>, Vec<Expression>),   // 泛型对象创建
    TypeCast(Box<Expression>, Type),                             // 类型转换
    TypeOf(Box<Expression>),                                     // 类型查询
}
```

### 2. 语法支持

#### 泛型函数
```codenothing
fn max<T>(a: T, b: T) : T {
    if (a > b) {
        return a;
    } else {
        return b;
    };
};
```

#### 泛型类
```codenothing
class Container<T> {
    private T value;
    
    constructor<T>(T initial_value) {
        this.value = initial_value;
    };
    
    fn get<T>() : T {
        return this.value;
    };
    
    fn set<T>(T new_value) : void {
        this.value = new_value;
    };
};
```

#### 泛型枚举
```codenothing
enum Option<T> {
    Some(T value),
    None
};
```

#### 泛型接口
```codenothing
interface Comparable<T> {
    fn compare(T other) : int;
};
```

#### 带约束的泛型
```codenothing
fn sort<T: Comparable<T>>(array<T> items) : array<T> where T: Copy {
    // 排序实现
    return items;
};
```

### 3. 解析器实现

#### 泛型解析器 (generic_parser.rs)
- `parse_generic_parameters()` - 解析泛型参数列表 `<T, U, K>`
- `parse_generic_parameter()` - 解析单个泛型参数
- `parse_type_constraints()` - 解析类型约束
- `parse_where_clause()` - 解析 where 子句
- `parse_generic_type_arguments()` - 解析泛型类型实例化
- `parse_generic_function_call()` - 解析泛型函数调用
- `parse_generic_object_creation()` - 解析泛型对象创建

#### 集成到现有解析器
- 函数解析器支持泛型参数和 where 子句
- 类解析器支持泛型类和泛型方法
- 枚举解析器支持泛型枚举
- 接口解析器支持泛型接口
- 表达式解析器支持泛型函数调用和对象创建

### 4. 类型系统扩展

#### 类型推断
- 支持 `name: auto = value` 变量声明的类型推断
- 使用 `Type::Auto` 进行自动类型推断

#### 类型检查
- 泛型类型参数的验证
- 类型约束的检查
- where 子句的验证

### 5. 运行时支持

#### 表达式求值器扩展
- `GenericFunctionCall` - 泛型函数调用求值
- `GenericMethodCall` - 泛型方法调用求值
- `GenericObjectCreation` - 泛型对象创建求值
- `TypeCast` - 类型转换求值
- `TypeOf` - 类型查询求值

## 使用示例

### 基本泛型函数
```codenothing
fn identity<T>(value: T) : T {
    return value;
};

fn main() : void {
    int_result: auto = identity<int>(42);
    string_result: auto = identity<string>("Hello");
};
```

### 泛型类使用
```codenothing
fn main() : void {
    int_container: auto = new Container<int>(42);
    string_container: auto = new Container<string>("Hello");

    value: auto = int_container.get<int>();
    int_container.set<int>(100);
};
```

### 类型转换和查询
```codenothing
fn main() : void {
    x: auto = 42;
    y: auto = x as float;
    type_info: auto = typeof(x);
};
```

## 当前限制

1. **类型擦除**: 当前实现在运行时忽略类型参数，主要用于编译时检查
2. **约束检查**: 类型约束的运行时检查尚未完全实现
3. **类型推断**: 复杂的类型推断场景可能不完全支持
4. **性能优化**: 泛型代码的性能优化尚未实现

## 未来改进

1. **完整的类型检查**: 实现完整的泛型类型检查和约束验证
2. **类型推断增强**: 改进类型推断算法
3. **运行时类型信息**: 添加运行时类型信息支持
4. **性能优化**: 实现泛型代码的单态化优化
5. **更多约束类型**: 添加更多内置约束类型
6. **高阶类型**: 支持高阶类型和类型构造器

