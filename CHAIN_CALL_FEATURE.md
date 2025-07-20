# CodeNothing 链式调用
## 语法设计

### 基本语法
```cn
object.method1().method2().method3()
```

### 支持的类型
- **字符串方法**: `trim()`, `to_upper()`, `to_lower()`, `length()`, `substring(start, end)`
- **数组方法**: `length()`, `push(item)`, `pop()`
- **映射方法**: `size()`, `get(key)`, `set(key, value)`

### 语法示例

#### 字符串链式调用
```cn
str : string = "  Hello World  ";
result : string = str.trim().to_upper().substring(0, 5);
// 结果: "HELLO"
```

#### 数组链式调用
```cn
arr : [int] = [1, 2, 3];
length : int = arr.length();
```

#### 映射链式调用
```cn
map : {string : string} = {"name" : "Alice"};
size : int = map.size();
```

## 支持的方法

### 字符串方法
- `trim()` - 去除首尾空格
- `to_upper()` - 转换为大写
- `to_lower()` - 转换为小写
- `length()` - 获取长度
- `substring(start, end)` - 截取子字符串

### 数组方法
- `length()` - 获取数组长度
- `push(item)` - 添加元素
- `pop()` - 移除并返回最后一个元素

### 映射方法
- `size()` - 获取映射大小
- `get(key)` - 获取值
- `set(key, value)` - 设置值

## 未来

1. **更多数据类型支持**: 支持更多数据类型的链式调用
2. **自定义方法**: 允许用户定义自定义方法
3. **错误处理**: 改进错误处理和异常机制
4. **性能优化**: 进一步优化链式调用的性能