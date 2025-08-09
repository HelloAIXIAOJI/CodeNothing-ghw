# CodeNothing Extended Math Library

CodeNothing扩展数学库提供了全面的数学计算功能，包括：
- 基础数学函数和扩展函数
- 三角函数和双曲函数
- 对数函数
- 统计函数
- 随机数生成
- 数值分析函数
- 丰富的数学常数

## 📦 安装和构建

### 构建库
```bash
# 在项目根目录运行
./build_libraries.sh

# 或者在Windows上运行
.\build_libraries.ps1
```

### 单独构建math库
```bash
cd library_math
cargo build --release
```

## 🚀 使用方法

### 1. 导入库
```codenothing
using lib <math>;
using lib <io>;
using ns std;
```

### 2. 基础数学函数

这些函数可以直接调用，无需命名空间前缀：

```codenothing
// 绝对值
result : string = abs("-5.5");        // "5.5"

// 最大值和最小值
max_val : string = max("10", "20");   // "20"
min_val : string = min("10", "20");   // "10"

// 幂运算
power : float = pow("2", "3");        // 8

// 平方根和立方根
sqrt_val : float = sqrt("16");        // 4
cbrt_val : float = cbrt("8");         // 2

// 取整函数
ceil_val : float = ceil("3.2");       // 4 (向上取整)
floor_val : float = floor("3.8");     // 3 (向下取整)
round_val : float = round("3.6");     // 4 (四舍五入)
trunc_val : float = trunc("3.9");     // 3 (截断)

// 符号函数
sign_val : int = sign("-5");          // -1
```

### 3. 三角函数

使用 `trig` 命名空间：

```codenothing
using ns trig;

// 基本三角函数（输入为弧度）
sin_val : string = sin("1.57");       // 约等于 "1"
cos_val : string = cos("0");          // "1"
tan_val : string = tan("0.785");      // 约等于 "1"

// 反三角函数
asin_val : string = asin("0.5");      // 约等于 "0.524"
acos_val : string = acos("0.5");      // 约等于 "1.047"
atan_val : string = atan("1");        // 约等于 "0.785"

// 角度弧度转换
radians : string = to_radians("90");  // 将90度转换为弧度
degrees : float = to_degrees("1.57");  // 将弧度转换为度数
```

### 4. 双曲函数

使用 `hyperbolic` 命名空间：

```codenothing
using ns hyperbolic;

// 双曲函数
sinh_val : float = sinh("1");         // 双曲正弦
cosh_val : float = cosh("0");         // 双曲余弦: 1
tanh_val : float = tanh("1");         // 双曲正切

// 反双曲函数
asinh_val : float = asinh("1");       // 反双曲正弦
acosh_val : float = acosh("2");       // 反双曲余弦
atanh_val : float = atanh("0.5");     // 反双曲正切
```

### 5. 统计函数

使用 `stats` 命名空间：

```codenothing
using ns stats;

// 统计函数（支持多个参数）
mean_val : float = mean("1", "2", "3", "4", "5");      // 平均值: 3
median_val : float = median("1", "3", "2", "5", "4");  // 中位数: 3
stddev_val : float = stddev("1", "2", "3", "4", "5");  // 标准差
variance_val : float = variance("1", "2", "3", "4", "5"); // 方差
```

### 6. 随机数生成

使用 `random` 命名空间：

```codenothing
using ns random;

// 设置随机种子
seed("12345");                        // 设置种子为12345

// 生成随机数
rand_val : float = random();          // 0-1之间的随机浮点数
rand_int : int = randint("1", "10");  // 1-10之间的随机整数
uniform_val : float = uniform("0", "100"); // 0-100之间的随机浮点数
```

### 7. 数值分析

使用 `numeric` 命名空间：

```codenothing
using ns numeric;

// 阶乘和组合
fact_val : int = factorial("5");      // 阶乘: 120
comb_val : int = combination("5", "2"); // 组合数C(5,2): 10
perm_val : int = permutation("5", "2"); // 排列数P(5,2): 20

// 数论函数
gcd_val : int = gcd("12", "8");       // 最大公约数: 4
lcm_val : int = lcm("12", "8");       // 最小公倍数: 24
```

### 4. 对数函数

使用 `log` 命名空间：

```codenothing
using ns log;

// 自然对数
ln_val : float = ln("2.718");         // 约等于 1

// 常用对数（以10为底）
log10_val : float = log10("100");     // 2

// 以2为底的对数
log2_val : float = log2("8");         // 3

// 指定底数的对数
log_val : float = log("8", "2");      // log₂(8) = 3
```

### 8. 数学常数

使用 `constants` 命名空间：

```codenothing
using ns constants;

// 基础常数
pi_val : float = pi();                // 圆周率π
e_val : float = e();                  // 自然常数e
phi_val : float = phi();              // 黄金比例φ
sqrt2_val : float = sqrt2();          // 2的平方根

// 扩展常数
euler_gamma_val : float = euler_gamma(); // 欧拉常数γ
frac_1_pi_val : float = frac_1_pi();  // 1/π
frac_2_pi_val : float = frac_2_pi();  // 2/π
ln_2_val : float = ln_2();            // ln(2)
ln_10_val : float = ln_10();          // ln(10)
```

## 📚 完整函数列表

### 基础数学函数（根命名空间）
| 函数 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `abs(x)` | 1个数值字符串 | 绝对值 | 计算数值的绝对值 |
| `max(a, b)` | 2个数值字符串 | 较大值 | 返回两个数值中的较大者 |
| `min(a, b)` | 2个数值字符串 | 较小值 | 返回两个数值中的较小者 |
| `pow(base, exp)` | 2个数值字符串 | 幂运算结果 | 计算base的exp次方 |
| `sqrt(x)` | 1个数值字符串 | 平方根 | 计算数值的平方根 |
| `cbrt(x)` | 1个数值字符串 | 立方根 | 计算数值的立方根 |
| `ceil(x)` | 1个数值字符串 | 向上取整 | 向上取整到最近的整数 |
| `floor(x)` | 1个数值字符串 | 向下取整 | 向下取整到最近的整数 |
| `round(x)` | 1个数值字符串 | 四舍五入 | 四舍五入到最近的整数 |
| `trunc(x)` | 1个数值字符串 | 截断 | 截断小数部分 |
| `sign(x)` | 1个数值字符串 | 符号 | 返回数值的符号(-1, 0, 1) |

### 三角函数（trig命名空间）
| 函数 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `sin(x)` | 弧度值 | 正弦值 | 计算正弦值 |
| `cos(x)` | 弧度值 | 余弦值 | 计算余弦值 |
| `tan(x)` | 弧度值 | 正切值 | 计算正切值 |
| `asin(x)` | -1到1之间的值 | 反正弦值 | 计算反正弦值（弧度） |
| `acos(x)` | -1到1之间的值 | 反余弦值 | 计算反余弦值（弧度） |
| `atan(x)` | 任意值 | 反正切值 | 计算反正切值（弧度） |
| `to_radians(degrees)` | 角度值 | 弧度值 | 将角度转换为弧度 |
| `to_degrees(radians)` | 弧度值 | 角度值 | 将弧度转换为角度 |

### 双曲函数（hyperbolic命名空间）
| 函数 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `sinh(x)` | 数值 | 双曲正弦值 | 计算双曲正弦值 |
| `cosh(x)` | 数值 | 双曲余弦值 | 计算双曲余弦值 |
| `tanh(x)` | 数值 | 双曲正切值 | 计算双曲正切值 |
| `asinh(x)` | 数值 | 反双曲正弦值 | 计算反双曲正弦值 |
| `acosh(x)` | ≥1的数值 | 反双曲余弦值 | 计算反双曲余弦值 |
| `atanh(x)` | -1<x<1的数值 | 反双曲正切值 | 计算反双曲正切值 |

### 统计函数（stats命名空间）
| 函数 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `mean(...)` | 多个数值 | 平均值 | 计算数值的算术平均值 |
| `median(...)` | 多个数值 | 中位数 | 计算数值的中位数 |
| `stddev(...)` | 多个数值 | 标准差 | 计算样本标准差 |
| `variance(...)` | 多个数值 | 方差 | 计算样本方差 |

### 随机数生成（random命名空间）
| 函数 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `seed(s)` | 种子值 | 种子值 | 设置随机数种子 |
| `random()` | 无 | 0-1随机数 | 生成0到1之间的随机浮点数 |
| `randint(min, max)` | 最小值, 最大值 | 随机整数 | 生成指定范围的随机整数 |
| `uniform(min, max)` | 最小值, 最大值 | 随机浮点数 | 生成指定范围的随机浮点数 |

### 数值分析（numeric命名空间）
| 函数 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `factorial(n)` | 非负整数 | 阶乘值 | 计算n的阶乘 |
| `combination(n, k)` | 两个非负整数 | 组合数 | 计算C(n,k)组合数 |
| `permutation(n, k)` | 两个非负整数 | 排列数 | 计算P(n,k)排列数 |
| `gcd(a, b)` | 两个正整数 | 最大公约数 | 计算最大公约数 |
| `lcm(a, b)` | 两个正整数 | 最小公倍数 | 计算最小公倍数 |

### 对数函数（log命名空间）
| 函数 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `ln(x)` | 正数 | 自然对数 | 计算以e为底的对数 |
| `log10(x)` | 正数 | 常用对数 | 计算以10为底的对数 |
| `log2(x)` | 正数 | 二进制对数 | 计算以2为底的对数 |
| `log(x, base)` | 正数, 底数 | 对数值 | 计算指定底数的对数 |

### 数学常数（constants命名空间）
| 函数 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `pi()` | 无 | π值 | 圆周率 3.141592653589793 |
| `e()` | 无 | e值 | 自然常数 2.718281828459045 |
| `phi()` | 无 | φ值 | 黄金比例 1.618033988749895 |
| `sqrt2()` | 无 | √2值 | 2的平方根 1.4142135623730951 |
| `euler_gamma()` | 无 | γ值 | 欧拉常数 0.5772156649015329 |
| `frac_1_pi()` | 无 | 1/π值 | 1/π的值 |
| `frac_2_pi()` | 无 | 2/π值 | 2/π的值 |
| `ln_2()` | 无 | ln(2)值 | 2的自然对数 |
| `ln_10()` | 无 | ln(10)值 | 10的自然对数 |

## ⚠️ 注意事项

### 输入输出格式
- 所有函数的参数和返回值都是字符串类型
- 输入的数值字符串会被解析为浮点数
- 无效输入会返回 "0"

### 错误处理
- 负数的平方根返回 "NaN"
- 超出定义域的反三角函数返回 "NaN"
- 非正数的对数返回 "NaN"
- 底数为1或非正数的对数返回 "NaN"

### 精度
- 使用Rust的f64类型进行计算
- 精度约为15-17位有效数字

## 🧪 测试

运行测试程序：
```bash
target/release/CodeNothing.exe math_library_test.cn
```

测试程序包含：
- 基础数学函数测试
- 三角函数测试
- 对数函数测试
- 数学常数测试
- 错误处理测试
- 综合计算示例

## 📝 示例程序

```codenothing
using lib <math>;
using lib <io>;
using ns std;
using ns trig;
using ns constants;
using ns stats;
using ns random;
using ns numeric;

fn main() : int {
    std::println("🧮 扩展Math库综合示例");

    // 1. 基础数学计算
    radius : int = 5;
    pi_val : float = pi();
    area : float = pi_val * pow("5", "2");

    std::println("圆的计算:");
    std::println("  半径: " + radius);
    std::println("  π: " + pi_val);
    std::println("  面积: π × r² ≈ " + area);

    // 2. 三角函数计算
    angle_deg : int = 45;
    angle_rad : float = to_radians("45");
    sin_45 : float = sin(angle_rad);
    cos_45 : float = cos(angle_rad);

    std::println("三角函数:");
    std::println("  sin(45°) = " + sin_45);
    std::println("  cos(45°) = " + cos_45);

    // 3. 统计分析
    data_mean : float = mean("85", "92", "78", "96", "88");
    data_median : float = median("85", "92", "78", "96", "88");

    std::println("统计分析:");
    std::println("  平均分: " + data_mean);
    std::println("  中位数: " + data_median);

    // 4. 随机数生成
    seed("2024");
    rand_score : int = randint("60", "100");

    std::println("随机生成:");
    std::println("  随机分数: " + rand_score);

    // 5. 数值分析
    fact_5 : int = factorial("5");
    comb_10_3 : int = combination("10", "3");

    std::println("数值分析:");
    std::println("  5! = " + fact_5);
    std::println("  C(10,3) = " + comb_10_3);

    return 0;
};
```

## 🔧 开发

### 添加新函数
1. 在相应的模块中定义函数
2. 在 `cn_init()` 函数中注册函数
3. 更新文档和测试

### 构建和测试
```bash
# 构建
cargo build --release

# 运行测试
target/release/CodeNothing.exe math_library_test.cn
```
