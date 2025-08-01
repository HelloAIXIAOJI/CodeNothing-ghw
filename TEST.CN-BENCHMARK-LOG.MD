# CodeNothing 版本性能变化

本文件记录了 CodeNothing 版本性能变化。使用--cn-time记录

以下是测试文件/test.cn的内容：
```codenothing
using lib <io>;
using ns std;

fn main() : int {
    for (i : 0..1000000) {
        std::println(i);
    };
    return 0;
};
```

### v0.5.11
#### 第一次
```
执行时间: 17621.727 ms [17.6 s]

real    0m17.629s
user    0m10.634s
sys     0m6.648s
```
#### 第二次
```
执行时间: 17276.244 ms [17.3 s]

real    0m17.283s
user    0m10.437s
sys     0m6.136s
```
#### 第三次
```
执行时间: 17318.170 ms [17.3 s]

real    0m17.327s
user    0m10.563s
sys     0m6.245s
```

### v0.6.0

- 干掉了LLL： Library Loading Lock

#### 第一次
```
执行时间: 8661.609 ms [8.7 s]

real    0m8.676s
user    0m4.534s
sys     0m3.798s
```
#### 第二次
```
执行时间: 8675.918 ms [8.7 s]

real    0m8.685s
user    0m4.597s
sys     0m3.834s
```
#### 第三次
```
执行时间: 8841.831 ms [8.8 s]

real    0m8.851s
user    0m4.634s
sys     0m3.807s
```