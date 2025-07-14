# CodeNothing 动态库示例

这个目录包含了一个示例动态库，展示了如何为 CodeNothing 语言创建动态库。

## 库结构

动态库必须遵循以下规则：

1. 必须导出一个名为 `cn_init` 的函数，该函数返回一个包含库函数的 HashMap 指针。
2. 库函数必须接受 `Vec<String>` 类型的参数，并返回 `String` 类型的结果。

## 编译库

使用以下命令编译库：

```bash
cargo build --release
```

编译后的库文件将位于 `target/release` 目录中。

## 安装库

将编译后的库文件复制到 CodeNothing 解释器的 `library` 目录中：

```bash
# Windows
copy target\release\io.dll ..\library\

# Linux/macOS
cp target/release/libio.so ../library/
```

## 在 CodeNothing 中使用库

在 CodeNothing 代码中，使用 `using lib_once <库名>;` 语句导入库，然后使用 `lib_库名::函数名(参数)` 的形式调用库函数：

```
fn main() : int {
    // 导入库
    using lib_once <io>;
    
    // 调用库函数
    lib_io::println("Hello, world!");
    
    return 0;
};
```

## 示例库函数

本示例库提供了以下函数：

- `print(text)`: 打印文本到标准输出，不添加换行符
- `println(text)`: 打印文本到标准输出，并添加换行符
- `read_line()`: 从标准输入读取一行文本，并返回该文本 