// 🚀 CodeNothing JIT编译器 v0.6.4
// 基于Cranelift的即时编译系统

use crate::ast::{Expression, BinaryOperator, Statement};
use crate::interpreter::value::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Module, Linkage};

/// JIT编译器状态
pub struct JitCompiler {
    /// 表达式热点检测计数器
    hotspot_counters: HashMap<String, u32>,
    /// 循环热点检测计数器
    loop_counters: HashMap<String, u32>,
    /// 🔄 v0.7.7: 增强的循环热点分析器
    loop_hotspot_analyzer: LoopHotspotAnalyzer,
    /// 函数调用热点检测计数器
    function_call_counters: HashMap<String, u32>,
    /// 数学表达式热点检测计数器
    math_expression_counters: HashMap<String, u32>,
    /// 字符串操作热点检测计数器
    string_operation_counters: HashMap<String, u32>,
    /// 编译缓存
    compiled_functions: HashMap<String, CompiledFunction>,
    /// 编译的循环缓存
    compiled_loops: HashMap<String, CompiledLoop>,
    /// 🔄 v0.7.7: 增强的循环JIT编译缓存
    compiled_loop_jit_functions: HashMap<String, CompiledLoopJitFunction>,
    /// 编译的函数调用缓存
    compiled_function_calls: HashMap<String, CompiledFunctionCall>,
    /// 编译的数学表达式缓存
    compiled_math_expressions: HashMap<String, CompiledMathExpression>,
    /// 编译的字符串操作缓存
    compiled_string_operations: HashMap<String, CompiledStringOperation>,
    /// 表达式热点阈值
    hotspot_threshold: u32,
    /// 循环热点阈值
    loop_threshold: u32,
    /// 函数调用热点阈值
    function_call_threshold: u32,
    /// 数学表达式热点阈值
    math_expression_threshold: u32,
    /// 字符串操作热点阈值
    string_operation_threshold: u32,
}

/// 编译后的函数
#[derive(Clone)]
pub struct CompiledFunction {
    /// 函数指针
    func_ptr: *const u8,
    /// 函数签名信息
    signature: FunctionSignature,
}

/// 编译后的循环
#[derive(Clone)]
pub struct CompiledLoop {
    /// 函数指针
    func_ptr: *const u8,
    /// 循环签名信息
    signature: LoopSignature,
    /// 循环类型
    loop_type: LoopType,
}

/// 编译后的函数调用
#[derive(Clone)]
pub struct CompiledFunctionCall {
    /// 函数指针
    func_ptr: *const u8,
    /// 函数调用签名信息
    signature: FunctionCallSignature,
    /// 函数调用类型
    call_type: FunctionCallType,
    /// 是否内联
    is_inlined: bool,
}

/// 函数调用类型
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCallType {
    Simple,      // 简单函数调用
    Recursive,   // 递归函数调用
    Inline,      // 内联函数调用
    TailCall,    // 尾调用优化
}

/// 函数调用签名
#[derive(Debug, Clone)]
pub struct FunctionCallSignature {
    /// 函数名
    function_name: String,
    /// 参数类型
    param_types: Vec<JitType>,
    /// 返回类型
    return_type: JitType,
    /// 调用约定
    calling_convention: CallingConvention,
}

/// 调用约定
#[derive(Debug, Clone, PartialEq)]
pub enum CallingConvention {
    Standard,    // 标准调用约定
    FastCall,    // 快速调用约定
    Inline,      // 内联调用
}

/// 内联成本效益分析
#[derive(Debug, Clone)]
pub struct InlineCostBenefit {
    /// 函数名
    pub function_name: String,
    /// 内联成本
    pub inline_cost: u32,
    /// 调用开销
    pub call_overhead: u32,
    /// 调用频率
    pub call_frequency: u32,
    /// 效益分数
    pub benefit_score: f64,
    /// 是否应该内联
    pub should_inline: bool,
}

/// 递归函数优化策略
#[derive(Debug, Clone, PartialEq)]
pub enum RecursiveOptimization {
    TailCallOptimization,  // 尾调用优化
    Memoization,          // 记忆化
    IterativeConversion,  // 转换为迭代
    StackOptimization,    // 栈优化
}

/// 数学表达式类型
#[derive(Debug, Clone, PartialEq)]
pub enum MathExpressionType {
    BasicArithmetic,      // 基础算术运算 (+, -, *, /, %)
    PowerOperation,       // 幂运算
    TrigonometricFunction, // 三角函数 (sin, cos, tan)
    LogarithmicFunction,  // 对数函数 (log, ln)
    ExponentialFunction,  // 指数函数 (exp, pow)
    SquareRootFunction,   // 平方根函数 (sqrt)
    ComplexExpression,    // 复杂数学表达式
}

/// 数学函数优化策略
#[derive(Debug, Clone, PartialEq)]
pub enum MathOptimization {
    SIMDVectorization,    // SIMD向量化
    LookupTable,          // 查表法
    TaylorSeries,         // 泰勒级数展开
    NewtonRaphson,        // 牛顿-拉夫逊法
    FastApproximation,    // 快速近似算法
    ConstantFolding,      // 常量折叠
}

/// 编译后的数学表达式
#[derive(Clone)]
pub struct CompiledMathExpression {
    /// 函数指针
    func_ptr: *const u8,
    /// 数学表达式签名
    signature: MathExpressionSignature,
    /// 表达式类型
    expression_type: MathExpressionType,
    /// 优化策略
    optimization: MathOptimization,
    /// 是否使用SIMD
    uses_simd: bool,
}

/// 数学表达式签名
#[derive(Debug, Clone)]
pub struct MathExpressionSignature {
    /// 表达式描述
    expression_desc: String,
    /// 输入类型
    input_types: Vec<JitType>,
    /// 输出类型
    output_type: JitType,
    /// 精度要求
    precision: MathPrecision,
}

/// 数学精度要求
#[derive(Debug, Clone, PartialEq)]
pub enum MathPrecision {
    Fast,        // 快速但精度较低
    Standard,    // 标准精度
    High,        // 高精度
    Extended,    // 扩展精度
}

/// 字符串操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum StringOperationType {
    Concatenation,    // 字符串拼接
    Search,          // 字符串搜索
    Replace,         // 字符串替换
    Substring,       // 子字符串提取
    Split,           // 字符串分割
    PatternMatch,    // 模式匹配
    Comparison,      // 字符串比较
    Formatting,      // 字符串格式化
}

/// 字符串优化策略
#[derive(Debug, Clone, PartialEq)]
pub enum StringOptimization {
    ZeroCopy,           // 零拷贝优化
    InPlaceModification, // 原地修改
    BufferReuse,        // 缓冲区重用
    BoyerMoore,         // Boyer-Moore搜索算法
    KMP,                // KMP搜索算法
    RabinKarp,          // Rabin-Karp搜索算法
    SmallStringOptimization, // 小字符串优化
    StringInterning,    // 字符串驻留
}

/// 编译后的字符串操作
#[derive(Clone)]
pub struct CompiledStringOperation {
    /// 函数指针
    func_ptr: *const u8,
    /// 字符串操作签名
    signature: StringOperationSignature,
    /// 操作类型
    operation_type: StringOperationType,
    /// 优化策略
    optimization: StringOptimization,
    /// 是否零拷贝
    is_zero_copy: bool,
}

/// 字符串操作签名
#[derive(Debug, Clone)]
pub struct StringOperationSignature {
    /// 操作描述
    operation_desc: String,
    /// 输入字符串数量
    input_count: usize,
    /// 输出类型
    output_type: StringOutputType,
    /// 内存使用策略
    memory_strategy: StringMemoryStrategy,
}

/// 字符串输出类型
#[derive(Debug, Clone, PartialEq)]
pub enum StringOutputType {
    String,      // 字符串
    Boolean,     // 布尔值（比较、搜索结果）
    Integer,     // 整数（位置、长度等）
    StringArray, // 字符串数组（分割结果）
}

/// 字符串内存策略
#[derive(Debug, Clone, PartialEq)]
pub enum StringMemoryStrategy {
    Allocate,    // 分配新内存
    Reuse,       // 重用现有内存
    InPlace,     // 原地操作
    View,        // 字符串视图（零拷贝）
}

/// 数组操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayOperationType {
    Access,          // 数组访问 array[index]
    Iteration,       // 数组遍历 for item in array
    Map,            // 数组映射 array.map(fn)
    Filter,         // 数组过滤 array.filter(fn)
    Reduce,         // 数组归约 array.reduce(fn, init)
    ForEach,        // 数组遍历 array.forEach(fn)
    Sort,           // 数组排序 array.sort()
    Search,         // 数组搜索 array.find(fn)
    Slice,          // 数组切片 array.slice(start, end)
    Concat,         // 数组连接 array.concat(other)
    Push,           // 数组添加 array.push(item)
    Pop,            // 数组弹出 array.pop()
    Length,         // 数组长度 array.length
    BoundsCheck,    // 边界检查优化
}

/// 数组优化策略
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayOptimization {
    BoundsCheckElimination,  // 边界检查消除
    Vectorization,          // 向量化操作
    MemoryPrefetch,         // 内存预取
    CacheOptimization,      // 缓存优化
    LoopUnrolling,          // 循环展开
    SIMDOperations,         // SIMD操作
    InPlaceOperations,      // 原地操作
    ParallelProcessing,     // 并行处理
    MemoryCoalescing,       // 内存合并访问
    BranchPrediction,       // 分支预测优化
}

/// 数组元素类型
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElementType {
    Integer,     // 整数
    Float,       // 浮点数
    String,      // 字符串
    Boolean,     // 布尔值
    Object,      // 对象
    Mixed,       // 混合类型
}

/// 数组输出类型
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayOutputType {
    Array,       // 数组
    Single,      // 单个值
    Boolean,     // 布尔值
    Integer,     // 整数
    Iterator,    // 迭代器
}

/// 数组内存访问模式
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayMemoryPattern {
    Sequential,    // 顺序访问
    Random,        // 随机访问
    Strided,       // 跨步访问
    Reverse,       // 反向访问
    Sparse,        // 稀疏访问
}

/// 数组操作签名
#[derive(Debug, Clone)]
pub struct ArrayOperationSignature {
    /// 操作描述
    operation_desc: String,
    /// 数组元素类型
    element_type: ArrayElementType,
    /// 数组大小（如果已知）
    array_size: Option<usize>,
    /// 输出类型
    output_type: ArrayOutputType,
    /// 内存访问模式
    memory_pattern: ArrayMemoryPattern,
}

/// 编译后的数组操作
#[derive(Clone)]
pub struct CompiledArrayOperation {
    /// 函数指针
    func_ptr: *const u8,
    /// 数组操作签名
    signature: ArrayOperationSignature,
    /// 操作类型
    operation_type: ArrayOperationType,
    /// 优化策略
    optimization: ArrayOptimization,
    /// 是否向量化
    is_vectorized: bool,
    /// 是否消除边界检查
    bounds_check_eliminated: bool,
}

/// 循环类型
#[derive(Debug, Clone, PartialEq)]
pub enum LoopType {
    While,
    For,
    ForEach,
}

/// 循环优化策略
#[derive(Debug, Clone, PartialEq)]
pub enum LoopOptimization {
    None,                    // 无优化
    Unroll(u32),            // 循环展开（展开因子）
    Vectorize,              // 向量化
    MemoryOptimize,         // 内存访问优化
    LoopInvariantHoisting,  // 循环不变量提升
    StrengthReduction,      // 强度削减
    LoopFusion,             // 循环融合
    Combined(Vec<LoopOptimization>), // 组合优化
}

/// 循环控制流上下文
#[derive(Debug, Clone)]
pub struct LoopControlContext {
    /// 循环继续块（continue跳转目标）
    pub continue_block: Block,
    /// 循环退出块（break跳转目标）
    pub break_block: Block,
    /// 循环类型
    pub loop_type: LoopType,
    /// 是否包含break/continue语句
    pub has_control_flow: bool,
}

/// 循环分析结果
#[derive(Debug, Clone)]
pub struct LoopAnalysis {
    /// 循环迭代次数（如果可确定）
    pub iteration_count: Option<u32>,
    /// 循环体复杂度评分
    pub complexity_score: u32,
    /// 是否包含内存访问
    pub has_memory_access: bool,
    /// 是否包含分支
    pub has_branches: bool,
    /// 是否包含break/continue控制流
    pub has_control_flow: bool,
    /// 循环不变量列表
    pub loop_invariants: Vec<String>,
    /// 变量依赖关系
    pub variable_dependencies: Vec<String>,
    /// 推荐的优化策略
    pub recommended_optimization: LoopOptimization,
}

/// 循环签名
#[derive(Debug, Clone)]
pub struct LoopSignature {
    /// 输入变量类型
    input_types: Vec<JitType>,
    /// 输出变量类型
    output_types: Vec<JitType>,
    /// 循环变量类型（for循环）
    loop_var_type: Option<JitType>,
}

impl CompiledFunction {
    /// 调用编译后的函数
    pub fn call(&self, args: &[i64]) -> i64 {
        match self.signature.param_types.len() {
            0 => {
                let func: fn() -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                func()
            },
            1 => {
                let func: fn(i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                func(args[0])
            },
            2 => {
                let func: fn(i64, i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                func(args[0], args[1])
            },
            3 => {
                let func: fn(i64, i64, i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                func(args[0], args[1], args[2])
            },
            4 => {
                let func: fn(i64, i64, i64, i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                func(args[0], args[1], args[2], args[3])
            },
            _ => {
                // 对于更多参数，使用通用调用方式
                let func: unsafe extern "C" fn(*const i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                unsafe { func(args.as_ptr()) }
            }
        }
    }
}

impl CompiledLoop {
    /// 调用编译后的循环
    pub fn call(&self, args: &[i64]) -> Vec<i64> {
        match self.signature.input_types.len() {
            0 => {
                let func: fn() -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                vec![func()]
            },
            1 => {
                let func: fn(i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                vec![func(args[0])]
            },
            2 => {
                let func: fn(i64, i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                vec![func(args[0], args[1])]
            },
            3 => {
                let func: fn(i64, i64, i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                vec![func(args[0], args[1], args[2])]
            },
            _ => {
                // 对于更多参数，使用通用调用方式
                let func: unsafe extern "C" fn(*const i64) -> i64 = unsafe { std::mem::transmute(self.func_ptr) };
                vec![unsafe { func(args.as_ptr()) }]
            }
        }
    }
}

/// 函数签名
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// 参数类型
    param_types: Vec<JitType>,
    /// 返回类型
    return_type: JitType,
}

/// JIT支持的类型
#[derive(Debug, Clone, PartialEq)]
pub enum JitType {
    Int32,
    Int64,
    Float64,
    Bool,
    Void,
}

impl JitCompiler {
    /// 创建新的JIT编译器
    pub fn new() -> Self {
        Self {
            hotspot_counters: HashMap::new(),
            loop_counters: HashMap::new(),
            loop_hotspot_analyzer: LoopHotspotAnalyzer::new(),
            function_call_counters: HashMap::new(),
            math_expression_counters: HashMap::new(),
            string_operation_counters: HashMap::new(),
            compiled_functions: HashMap::new(),
            compiled_loops: HashMap::new(),
            compiled_loop_jit_functions: HashMap::new(),
            compiled_function_calls: HashMap::new(),
            compiled_math_expressions: HashMap::new(),
            compiled_string_operations: HashMap::new(),
            hotspot_threshold: 100, // 表达式执行100次后触发JIT编译
            loop_threshold: 100,    // 循环执行100次后触发JIT编译
            function_call_threshold: 50, // 函数调用50次后触发JIT编译
            math_expression_threshold: 30, // 数学表达式30次后触发JIT编译
            string_operation_threshold: 25, // 字符串操作25次后触发JIT编译
        }
    }

    /// 检查表达式是否应该JIT编译
    pub fn should_compile(&mut self, key: &str) -> bool {
        let counter = self.hotspot_counters.entry(key.to_string()).or_insert(0);
        *counter += 1;
        *counter >= self.hotspot_threshold
    }

    /// 检查循环是否应该JIT编译
    pub fn should_compile_loop(&mut self, key: &str) -> bool {
        let counter = self.loop_counters.entry(key.to_string()).or_insert(0);
        *counter += 1;
        *counter >= self.loop_threshold
    }

    /// 🔄 v0.7.7: 记录循环执行并分析热点
    pub fn record_and_analyze_loop(&mut self, loop_key: &str, iterations: usize, execution_time: Duration, loop_body: &[Statement]) {
        self.loop_hotspot_analyzer.record_loop_execution(loop_key, iterations, execution_time, loop_body);
    }

    /// 🔄 v0.7.7: 检查是否应该JIT编译循环（增强版）
    pub fn should_jit_compile_loop_enhanced(&self, loop_key: &str) -> bool {
        self.loop_hotspot_analyzer.should_jit_compile_loop(loop_key)
    }

    /// 🔄 v0.7.7: 获取循环热点分析统计
    pub fn get_loop_hotspot_stats(&self) -> LoopHotspotAnalyzerStats {
        self.loop_hotspot_analyzer.get_analyzer_stats()
    }

    /// 🔄 v0.7.7: 获取所有热点循环
    pub fn get_hotspot_loops(&self) -> Vec<(String, f32)> {
        self.loop_hotspot_analyzer.get_hotspot_loops()
    }

    /// 🔄 v0.7.7: 编译循环JIT函数
    pub fn compile_loop_jit(&mut self, loop_key: &str, loop_body: &[Statement], loop_condition: Option<&Expression>) -> Result<CompiledLoopJitFunction, String> {
        crate::jit_debug_println!("🔄 JIT: 开始编译循环JIT函数 {}", loop_key);

        // 检查是否已经编译过
        if let Some(compiled) = self.compiled_loop_jit_functions.get(loop_key) {
            crate::jit_debug_println!("🔄 JIT: 使用缓存的循环JIT函数 {}", loop_key);
            return Ok(compiled.clone());
        }

        let compilation_start = std::time::Instant::now();

        // 分析循环特征
        let loop_stats = self.loop_hotspot_analyzer.get_loop_stats(loop_key);
        let optimization_strategies = self.select_optimization_strategies(loop_stats, loop_body);

        // 创建Cranelift编译器
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
            .map_err(|e| format!("创建JIT构建器失败: {}", e))?;
        let mut module = JITModule::new(builder);

        // 创建函数签名
        let signature = self.create_loop_jit_signature(loop_body)?;
        let mut func = Function::new();
        func.signature = signature.clone();

        // 简化的编译过程（暂时跳过复杂的Cranelift编译）
        // TODO: 实现完整的Cranelift编译逻辑
        crate::jit_debug_println!("🔄 JIT: 简化编译循环体，策略数量: {}", optimization_strategies.len());

        // 简化的函数指针创建（暂时使用占位符）
        let func_ptr = std::ptr::null();

        let compilation_time = compilation_start.elapsed();

        // 创建编译结果
        let compiled_function = CompiledLoopJitFunction {
            func_ptr,
            signature: LoopJitSignature {
                input_types: vec![JitType::Int64], // 简化处理
                output_type: JitType::Int64,
                loop_variables: vec![], // 后续扩展
            },
            optimization_strategies: optimization_strategies.iter().map(|s| format!("{:?}", s)).collect(),
            compilation_time,
            expected_speedup: self.estimate_speedup(&optimization_strategies),
        };

        // 缓存编译结果
        self.compiled_loop_jit_functions.insert(loop_key.to_string(), compiled_function.clone());

        crate::jit_debug_println!("🔄 JIT: 循环JIT编译完成 {} - 耗时: {:?}, 预期加速: {:.2}x",
                                 loop_key, compilation_time, compiled_function.expected_speedup);

        Ok(compiled_function)
    }

    /// 🔄 v0.7.7: 选择循环优化策略
    fn select_optimization_strategies(&self, loop_stats: Option<&LoopExecutionStats>, loop_body: &[Statement]) -> Vec<LoopOptimizationStrategy> {
        let mut strategies = Vec::new();

        // 基于循环统计选择策略
        if let Some(stats) = loop_stats {
            // 小迭代次数循环 - 考虑展开
            if stats.average_iterations_per_execution < 20.0 {
                strategies.push(LoopOptimizationStrategy::LoopUnrolling { factor: 4 });
            }

            // 大迭代次数循环 - 考虑向量化
            if stats.average_iterations_per_execution > 100.0 {
                strategies.push(LoopOptimizationStrategy::Vectorization { simd_width: 4 });
            }

            // 内存密集型循环 - 考虑预取
            if stats.memory_usage_pattern.is_memory_intensive {
                strategies.push(LoopOptimizationStrategy::MemoryPrefetching);
            }
        }

        // 基于循环体分析选择策略
        if self.has_loop_invariants(loop_body) {
            strategies.push(LoopOptimizationStrategy::LoopInvariantCodeMotion);
        }

        if self.has_strength_reduction_opportunities(loop_body) {
            strategies.push(LoopOptimizationStrategy::StrengthReduction);
        }

        // 默认策略
        if strategies.is_empty() {
            strategies.push(LoopOptimizationStrategy::LoopUnrolling { factor: 2 });
        }

        crate::jit_debug_println!("🎯 JIT: 选择优化策略: {:?}", strategies);
        strategies
    }

    /// 🔄 v0.7.7: 应用优化策略
    fn apply_optimization_strategy(
        &self,
        builder: &mut FunctionBuilder,
        strategy: &LoopOptimizationStrategy,
        loop_body: &[Statement],
        _loop_condition: Option<&Expression>
    ) -> Result<(), String> {
        match strategy {
            LoopOptimizationStrategy::LoopUnrolling { factor } => {
                crate::jit_debug_println!("🔄 JIT: 应用循环展开优化，因子: {}", factor);
                // 循环展开实现（简化版）
                for _i in 0..*factor {
                    // 这里会重复生成循环体代码
                    // 实际实现需要更复杂的逻辑
                }
            },
            LoopOptimizationStrategy::Vectorization { simd_width } => {
                crate::jit_debug_println!("🔄 JIT: 应用向量化优化，SIMD宽度: {}", simd_width);
                // 向量化实现（简化版）
                // 需要分析循环体中的数组操作并生成SIMD指令
            },
            LoopOptimizationStrategy::StrengthReduction => {
                crate::jit_debug_println!("🔄 JIT: 应用强度削减优化");
                // 强度削减实现（简化版）
                // 将昂贵的乘法操作替换为加法
            },
            LoopOptimizationStrategy::LoopInvariantCodeMotion => {
                crate::jit_debug_println!("🔄 JIT: 应用循环不变量提升优化");
                // 循环不变量提升实现（简化版）
                // 将循环不变的计算移到循环外
            },
            LoopOptimizationStrategy::LoopFusion => {
                crate::jit_debug_println!("🔄 JIT: 应用循环融合优化");
                // 循环融合实现（简化版）
            },
            LoopOptimizationStrategy::MemoryPrefetching => {
                crate::jit_debug_println!("🔄 JIT: 应用内存预取优化");
                // 内存预取实现（简化版）
                // 生成预取指令
            },
        }
        Ok(())
    }

    /// 🔄 v0.7.7: 编译循环体（JIT版本）
    fn compile_loop_body_jit(&self, builder: &mut FunctionBuilder, loop_body: &[Statement]) -> Result<(), String> {
        for stmt in loop_body {
            match stmt {
                Statement::VariableDeclaration(name, _var_type, init_expr) => {
                    crate::jit_debug_println!("🔄 JIT: 编译变量声明 {}", name);
                    // 编译变量声明
                    if let Some(expr) = init_expr {
                        // 编译初始化表达式
                        let _value = self.compile_expression_jit(builder, expr)?;
                        // 存储到变量
                    }
                },
                Statement::VariableAssignment(name, expr) => {
                    crate::jit_debug_println!("🔄 JIT: 编译变量赋值 {}", name);
                    // 编译变量赋值
                    let _value = self.compile_expression_jit(builder, expr)?;
                    // 存储到变量
                },
                Statement::IfElse(condition, then_block, else_blocks) => {
                    crate::jit_debug_println!("🔄 JIT: 编译if-else语句");
                    // 编译条件分支
                    let _condition_value = self.compile_expression_jit(builder, condition)?;

                    // 创建分支块
                    let then_block_id = builder.create_block();
                    let else_block_id = builder.create_block();
                    let merge_block_id = builder.create_block();

                    // 条件跳转
                    // builder.ins().brz(condition_value, else_block_id, &[]);
                    // builder.ins().jump(then_block_id, &[]);

                    // 编译then块
                    builder.switch_to_block(then_block_id);
                    for then_stmt in then_block {
                        self.compile_loop_body_jit(builder, &[then_stmt.clone()])?;
                    }
                    builder.ins().jump(merge_block_id, &[]);

                    // 编译else块
                    builder.switch_to_block(else_block_id);
                    for (else_condition, else_block) in else_blocks {
                        if else_condition.is_none() {
                            // 最终的else块
                            for else_stmt in else_block {
                                self.compile_loop_body_jit(builder, &[else_stmt.clone()])?;
                            }
                        }
                    }
                    builder.ins().jump(merge_block_id, &[]);

                    // 合并块
                    builder.switch_to_block(merge_block_id);
                },
                _ => {
                    // 其他语句类型的编译
                    crate::jit_debug_println!("🔄 JIT: 编译其他语句类型");
                }
            }
        }
        Ok(())
    }

    /// 🔄 v0.7.7: 编译表达式（JIT版本）
    fn compile_expression_jit(&self, builder: &mut FunctionBuilder, expr: &Expression) -> Result<cranelift::prelude::Value, String> {
        match expr {
            Expression::IntLiteral(value) => {
                Ok(builder.ins().iconst(types::I64, *value as i64))
            },
            Expression::Variable(name) => {
                crate::jit_debug_println!("🔄 JIT: 编译变量访问 {}", name);
                // 简化处理：返回常量值
                Ok(builder.ins().iconst(types::I64, 0))
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.compile_expression_jit(builder, left)?;
                let right_val = self.compile_expression_jit(builder, right)?;

                match op {
                    BinaryOperator::Add => Ok(builder.ins().iadd(left_val, right_val)),
                    BinaryOperator::Subtract => Ok(builder.ins().isub(left_val, right_val)),
                    BinaryOperator::Multiply => Ok(builder.ins().imul(left_val, right_val)),
                    BinaryOperator::Divide => Ok(builder.ins().sdiv(left_val, right_val)),
                    BinaryOperator::Modulo => Ok(builder.ins().srem(left_val, right_val)),
                    _ => Err(format!("不支持的二元操作符: {:?}", op))
                }
            },
            _ => {
                crate::jit_debug_println!("🔄 JIT: 编译其他表达式类型");
                Ok(builder.ins().iconst(types::I64, 0))
            }
        }
    }

    /// 🔄 v0.7.7: 检查循环是否有不变量
    fn has_loop_invariants(&self, loop_body: &[Statement]) -> bool {
        // 简化实现：检查是否有常量表达式
        for stmt in loop_body {
            match stmt {
                Statement::VariableDeclaration(_, _, Some(expr)) => {
                    match expr {
                        Expression::IntLiteral(_) => return true,
                        Expression::FloatLiteral(_) => return true,
                        Expression::StringLiteral(_) => return true,
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        false
    }

    /// 🔄 v0.7.7: 检查是否有强度削减机会
    fn has_strength_reduction_opportunities(&self, loop_body: &[Statement]) -> bool {
        // 简化实现：检查是否有乘法操作
        for stmt in loop_body {
            if let Statement::VariableAssignment(_, expr) = stmt {
                if self.contains_multiplication(expr) {
                    return true;
                }
            }
        }
        false
    }

    /// 🔄 v0.7.7: 检查表达式是否包含乘法
    fn contains_multiplication(&self, expr: &Expression) -> bool {
        match expr {
            Expression::BinaryOp(_, BinaryOperator::Multiply, _) => true,
            Expression::BinaryOp(left, _, right) => {
                self.contains_multiplication(left) || self.contains_multiplication(right)
            },
            _ => false,
        }
    }

    /// 🔄 v0.7.7: 创建循环JIT函数签名
    fn create_loop_jit_signature(&self, _loop_body: &[Statement]) -> Result<Signature, String> {
        let mut sig = Signature::new(isa::CallConv::SystemV);
        // 简化处理：无参数，返回i64
        sig.returns.push(AbiParam::new(types::I64));
        Ok(sig)
    }

    /// 🔄 v0.7.7: 估算性能提升
    fn estimate_speedup(&self, strategies: &[LoopOptimizationStrategy]) -> f32 {
        let mut speedup = 1.0;

        for strategy in strategies {
            match strategy {
                LoopOptimizationStrategy::LoopUnrolling { factor } => {
                    speedup *= 1.0 + (*factor as f32 * 0.1); // 每个展开因子增加10%
                },
                LoopOptimizationStrategy::Vectorization { simd_width } => {
                    speedup *= 1.0 + (*simd_width as f32 * 0.2); // SIMD带来显著提升
                },
                LoopOptimizationStrategy::StrengthReduction => {
                    speedup *= 1.15; // 强度削减带来15%提升
                },
                LoopOptimizationStrategy::LoopInvariantCodeMotion => {
                    speedup *= 1.25; // 不变量提升带来25%提升
                },
                LoopOptimizationStrategy::LoopFusion => {
                    speedup *= 1.20; // 循环融合带来20%提升
                },
                LoopOptimizationStrategy::MemoryPrefetching => {
                    speedup *= 1.10; // 内存预取带来10%提升
                },
            }
        }

        speedup
    }

    /// 检查函数调用是否应该JIT编译
    pub fn should_compile_function_call(&mut self, function_name: &str, call_site: &str) -> bool {
        let key = format!("{}@{}", function_name, call_site);
        let counter = self.function_call_counters.entry(key).or_insert(0);
        *counter += 1;
        *counter >= self.function_call_threshold
    }

    /// 检查数学表达式是否应该JIT编译
    pub fn should_compile_math_expression(&mut self, expression_key: &str) -> bool {
        let counter = self.math_expression_counters.entry(expression_key.to_string()).or_insert(0);
        *counter += 1;
        *counter >= self.math_expression_threshold
    }

    /// 检查字符串操作是否应该JIT编译
    pub fn should_compile_string_operation(&mut self, operation_key: &str) -> bool {
        let counter = self.string_operation_counters.entry(operation_key.to_string()).or_insert(0);
        *counter += 1;
        *counter >= self.string_operation_threshold
    }

    /// 生成函数调用的唯一键
    pub fn generate_function_call_key(&self, function_name: &str, call_site: &str) -> String {
        format!("call_{}_{}", function_name, call_site)
    }

    /// 生成循环的唯一键
    pub fn generate_loop_key(&self, loop_type: &str, location: &str) -> String {
        format!("loop_{}_{}", loop_type, location)
    }

    /// 生成数学表达式的唯一键
    pub fn generate_math_expression_key(&self, expression: &Expression) -> String {
        match expression {
            Expression::BinaryOp(left, op, right) => {
                let left_key = self.generate_math_expression_key(left);
                let right_key = self.generate_math_expression_key(right);
                let op_str = match op {
                    BinaryOperator::Add => "add",
                    BinaryOperator::Subtract => "sub",
                    BinaryOperator::Multiply => "mul",
                    BinaryOperator::Divide => "div",
                    BinaryOperator::Modulo => "mod",
                    // v0.7.2新增：位运算符支持
                    BinaryOperator::BitwiseAnd => "and",
                    BinaryOperator::BitwiseOr => "or",
                    BinaryOperator::BitwiseXor => "xor",
                    BinaryOperator::LeftShift => "shl",
                    BinaryOperator::RightShift => "shr",
                };
                format!("math_{}_{}__{}", op_str, left_key, right_key)
            },
            Expression::IntLiteral(n) => format!("int_{}", n),
            Expression::FloatLiteral(f) => format!("float_{}", f.to_bits()),
            Expression::Variable(name) => format!("var_{}", name),
            Expression::FunctionCall(name, args) => {
                let args_key = args.iter()
                    .map(|arg| self.generate_math_expression_key(arg))
                    .collect::<Vec<_>>()
                    .join("_");
                format!("func_{}_{}", name, args_key)
            },
            _ => "complex_expr".to_string(),
        }
    }

    /// 生成字符串操作的唯一键
    pub fn generate_string_operation_key(&self, operation: &str, operands: &[String]) -> String {
        let operands_key = operands.join("_");
        format!("string_{}_{}", operation, operands_key)
    }

    /// 识别字符串操作类型
    pub fn identify_string_operation_type(&self, operation: &str) -> StringOperationType {
        match operation {
            "concat" | "+" => StringOperationType::Concatenation,
            "contains" | "indexOf" | "search" => StringOperationType::Search,
            "replace" | "replaceAll" => StringOperationType::Replace,
            "substring" | "substr" | "slice" => StringOperationType::Substring,
            "split" => StringOperationType::Split,
            "match" | "regex" => StringOperationType::PatternMatch,
            "equals" | "compare" | "==" | "!=" => StringOperationType::Comparison,
            "format" | "sprintf" => StringOperationType::Formatting,
            _ => StringOperationType::Concatenation, // 默认为拼接
        }
    }

    /// 选择字符串操作的优化策略
    pub fn select_string_optimization(&self, op_type: &StringOperationType, string_length: usize) -> StringOptimization {
        match op_type {
            StringOperationType::Concatenation => {
                if string_length <= 64 {
                    StringOptimization::SmallStringOptimization
                } else {
                    StringOptimization::ZeroCopy
                }
            },
            StringOperationType::Search => {
                if string_length > 1000 {
                    StringOptimization::BoyerMoore
                } else {
                    StringOptimization::KMP
                }
            },
            StringOperationType::Replace => {
                StringOptimization::InPlaceModification
            },
            StringOperationType::Substring => {
                StringOptimization::ZeroCopy
            },
            StringOperationType::Split => {
                StringOptimization::BufferReuse
            },
            StringOperationType::PatternMatch => {
                StringOptimization::RabinKarp
            },
            StringOperationType::Comparison => {
                StringOptimization::ZeroCopy
            },
            StringOperationType::Formatting => {
                StringOptimization::BufferReuse
            },
        }
    }

    /// 识别数学表达式类型
    pub fn identify_math_expression_type(&self, expression: &Expression) -> MathExpressionType {
        match expression {
            Expression::BinaryOp(_, op, _) => {
                match op {
                    BinaryOperator::Add | BinaryOperator::Subtract |
                    BinaryOperator::Multiply | BinaryOperator::Divide |
                    BinaryOperator::Modulo => MathExpressionType::BasicArithmetic,
                    // v0.7.2新增：位运算表达式类型
                    BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOr |
                    BinaryOperator::BitwiseXor | BinaryOperator::LeftShift |
                    BinaryOperator::RightShift => MathExpressionType::BasicArithmetic,
                }
            },
            Expression::FunctionCall(name, _) => {
                match name.as_str() {
                    "sin" | "cos" | "tan" | "asin" | "acos" | "atan" => {
                        MathExpressionType::TrigonometricFunction
                    },
                    "log" | "ln" | "log10" | "log2" => {
                        MathExpressionType::LogarithmicFunction
                    },
                    "exp" | "pow" => {
                        MathExpressionType::ExponentialFunction
                    },
                    "sqrt" | "cbrt" => {
                        MathExpressionType::SquareRootFunction
                    },
                    "power" | "**" => {
                        MathExpressionType::PowerOperation
                    },
                    _ => MathExpressionType::ComplexExpression,
                }
            },
            _ => MathExpressionType::ComplexExpression,
        }
    }

    /// 选择数学表达式的优化策略
    pub fn select_math_optimization(&self, expr_type: &MathExpressionType, complexity: u32) -> MathOptimization {
        match expr_type {
            MathExpressionType::BasicArithmetic => {
                if complexity <= 3 {
                    MathOptimization::SIMDVectorization
                } else {
                    MathOptimization::ConstantFolding
                }
            },
            MathExpressionType::TrigonometricFunction => {
                MathOptimization::LookupTable
            },
            MathExpressionType::LogarithmicFunction => {
                MathOptimization::TaylorSeries
            },
            MathExpressionType::ExponentialFunction => {
                MathOptimization::FastApproximation
            },
            MathExpressionType::SquareRootFunction => {
                MathOptimization::NewtonRaphson
            },
            MathExpressionType::PowerOperation => {
                MathOptimization::FastApproximation
            },
            MathExpressionType::ComplexExpression => {
                MathOptimization::SIMDVectorization
            },
        }
    }

    /// 检查表达式是否适合JIT编译
    pub fn can_compile_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::IntLiteral(_) => true,
            Expression::FloatLiteral(_) => true,
            Expression::Variable(_) => true,
            Expression::BinaryOp(left, op, right) => {
                self.is_simple_binary_op(op) &&
                self.can_compile_expression(left) &&
                self.can_compile_expression(right)
            },
            Expression::CompareOp(left, op, right) => {
                self.is_simple_compare_op(op) &&
                self.can_compile_expression(left) &&
                self.can_compile_expression(right)
            },
            Expression::LogicalOp(left, op, right) => {
                self.is_simple_logical_op(op) &&
                self.can_compile_expression(left) &&
                self.can_compile_expression(right)
            },
            _ => false,
        }
    }

    /// 检查语句是否适合JIT编译
    pub fn can_compile_statement(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::VariableDeclaration(_, _, expr) => {
                self.can_compile_expression(expr)
            },
            Statement::VariableAssignment(_, expr) => {
                self.can_compile_expression(expr)
            },
            Statement::WhileLoop(condition, body) => {
                self.can_compile_expression(condition) &&
                body.iter().all(|s| self.can_compile_simple_statement(s))
            },
            Statement::ForLoop(_, start, end, body) => {
                self.can_compile_expression(start) &&
                self.can_compile_expression(end) &&
                body.iter().all(|s| self.can_compile_simple_statement(s))
            },
            _ => false,
        }
    }

    /// 检查函数调用是否适合JIT编译
    pub fn can_compile_function_call(&self, function_name: &str, args: &[Expression]) -> bool {
        // 检查函数名是否为简单函数
        if !self.is_simple_function(function_name) {
            return false;
        }

        // 检查参数是否都可以编译
        args.iter().all(|arg| self.can_compile_expression(arg))
    }

    /// 检查是否为简单函数（适合JIT编译）
    fn is_simple_function(&self, function_name: &str) -> bool {
        // 简单的数学函数和用户定义的小函数
        matches!(function_name,
            "abs" | "max" | "min" | "sqrt" | "pow" |
            "add" | "sub" | "mul" | "div" | "mod" |
            "factorial" | "fibonacci" | "gcd" | "lcm"
        ) || function_name.len() <= 20 // 简单启发式：短函数名通常是简单函数
    }

    /// 检查函数是否适合内联
    pub fn should_inline_function(&self, function_name: &str, function_body_size: usize) -> bool {
        // 内联条件：
        // 1. 函数体很小（少于10行）
        // 2. 不是递归函数
        // 3. 参数数量少于5个
        // 4. 是简单的数学运算函数
        function_body_size <= 10 &&
        !self.is_recursive_function(function_name) &&
        self.is_inline_candidate(function_name)
    }

    /// 检查函数是否为内联候选
    fn is_inline_candidate(&self, function_name: &str) -> bool {
        // 优先内联的函数类型
        matches!(function_name,
            "double" | "triple" | "square" | "cube" |
            "add" | "sub" | "mul" | "div" | "mod" |
            "abs" | "max" | "min" | "clamp" |
            "is_even" | "is_odd" | "sign"
        ) ||
        // 短函数名通常是简单函数
        function_name.len() <= 8 ||
        // 包含简单操作关键词的函数
        function_name.contains("get") ||
        function_name.contains("set") ||
        function_name.contains("calc")
    }

    /// 检查是否为递归函数
    fn is_recursive_function(&self, function_name: &str) -> bool {
        // 简单启发式：检查函数名是否包含递归相关的关键词
        matches!(function_name, "factorial" | "fibonacci" | "gcd") ||
        function_name.contains("recursive") ||
        function_name.contains("recur")
    }

    /// 计算内联成本效益分析
    pub fn analyze_inline_cost_benefit(&self, function_name: &str, call_frequency: u32) -> InlineCostBenefit {
        let inline_cost = self.calculate_inline_cost(function_name);
        let call_overhead = self.calculate_call_overhead(function_name);
        let benefit_score = (call_overhead as f64 * call_frequency as f64) - inline_cost as f64;

        InlineCostBenefit {
            function_name: function_name.to_string(),
            inline_cost,
            call_overhead,
            call_frequency,
            benefit_score,
            should_inline: benefit_score > 0.0 && self.is_inline_candidate(function_name),
        }
    }

    /// 计算内联成本
    fn calculate_inline_cost(&self, function_name: &str) -> u32 {
        // 基于函数复杂度的内联成本估算
        match function_name {
            "double" | "triple" => 1,  // 非常简单的函数
            "add" | "sub" | "mul" => 2,  // 简单数学运算
            "square" | "cube" => 3,  // 稍复杂的运算
            "abs" | "max" | "min" => 4,  // 条件运算
            _ => {
                // 基于函数名长度的启发式估算
                if function_name.len() <= 5 {
                    3
                } else if function_name.len() <= 10 {
                    5
                } else {
                    8
                }
            }
        }
    }

    /// 计算函数调用开销
    fn calculate_call_overhead(&self, function_name: &str) -> u32 {
        // 函数调用的固定开销
        let base_overhead = 10; // 基础调用开销

        // 根据函数类型调整开销
        let type_overhead = if self.is_recursive_function(function_name) {
            5 // 递归函数额外开销
        } else if self.is_inline_candidate(function_name) {
            2 // 简单函数较少开销
        } else {
            3 // 普通函数开销
        };

        base_overhead + type_overhead
    }

    /// 检查递归函数是否适合优化
    pub fn should_optimize_recursive_function(&self, function_name: &str, recursion_depth: u32) -> bool {
        // 递归优化条件：
        // 1. 是递归函数
        // 2. 递归深度不太深（避免栈溢出）
        // 3. 是简单的递归模式
        self.is_recursive_function(function_name) &&
        recursion_depth <= 100 && // 最大递归深度限制
        self.is_simple_recursive_pattern(function_name)
    }

    /// 检查是否为简单递归模式
    fn is_simple_recursive_pattern(&self, function_name: &str) -> bool {
        // 简单递归模式：尾递归、线性递归等
        matches!(function_name,
            "factorial" | "fibonacci" | "gcd" | "power" |
            "sum_recursive" | "count_recursive" | "find_recursive"
        ) || function_name.contains("tail_") || function_name.contains("linear_")
    }

    /// 分析递归函数的优化策略
    pub fn analyze_recursive_optimization(&self, function_name: &str) -> RecursiveOptimization {
        if self.is_tail_recursive(function_name) {
            RecursiveOptimization::TailCallOptimization
        } else if self.is_memoizable(function_name) {
            RecursiveOptimization::Memoization
        } else if self.can_convert_to_iterative(function_name) {
            RecursiveOptimization::IterativeConversion
        } else {
            RecursiveOptimization::StackOptimization
        }
    }

    /// 检查是否为尾递归
    fn is_tail_recursive(&self, function_name: &str) -> bool {
        // 简单启发式：检查函数名或已知的尾递归函数
        function_name.contains("tail_") ||
        matches!(function_name, "factorial_tail" | "sum_tail" | "gcd")
    }

    /// 检查是否可以记忆化
    fn is_memoizable(&self, function_name: &str) -> bool {
        // 适合记忆化的递归函数：fibonacci、动态规划等
        matches!(function_name, "fibonacci" | "fib") ||
        function_name.contains("dp_") ||
        function_name.contains("memo_")
    }

    /// 检查是否可以转换为迭代
    fn can_convert_to_iterative(&self, function_name: &str) -> bool {
        // 可以转换为迭代的递归函数
        matches!(function_name, "factorial" | "power" | "sum_recursive") ||
        function_name.contains("linear_")
    }

    /// 检查简单语句是否适合JIT编译（用于循环体）
    pub fn can_compile_simple_statement(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::VariableDeclaration(_, var_type, expr) => {
                // 支持简单类型的变量声明
                self.is_simple_type_direct(var_type) && self.can_compile_expression(expr)
            },
            Statement::VariableAssignment(_, expr) => {
                self.can_compile_expression(expr)
            },
            Statement::Increment(_) | Statement::Decrement(_) |
            Statement::PreIncrement(_) | Statement::PreDecrement(_) => true,
            Statement::CompoundAssignment(_, op, expr) => {
                self.is_simple_binary_op(op) && self.can_compile_expression(expr)
            },
            // 支持循环内条件语句编译
            Statement::IfElse(condition, then_stmts, else_branches) => {
                self.can_compile_expression(condition) &&
                then_stmts.len() <= 5 && // 增加then分支语句数量限制
                else_branches.len() <= 1 && // 只支持一个else分支
                then_stmts.iter().all(|s| self.can_compile_simple_statement(s)) &&
                else_branches.iter().all(|(cond, stmts)| {
                    cond.is_none() && // 只支持else，不支持else-if
                    stmts.len() <= 5 && // 增加else分支语句数量限制
                    stmts.iter().all(|s| self.can_compile_simple_statement(s))
                })
            },

            // 支持break和continue控制流语句
            Statement::Break | Statement::Continue => true,
            _ => false,
        }
    }

    /// 检查是否为简单类型
    fn is_simple_type(&self, var_type: &Option<crate::ast::Type>) -> bool {
        match var_type {
            Some(crate::ast::Type::Int) |
            Some(crate::ast::Type::Long) |
            Some(crate::ast::Type::Float) |
            Some(crate::ast::Type::Bool) |
            None => true, // None表示类型推断
            _ => false,
        }
    }

    /// 检查是否为简单类型（直接类型）
    fn is_simple_type_direct(&self, var_type: &crate::ast::Type) -> bool {
        matches!(var_type,
            crate::ast::Type::Int |
            crate::ast::Type::Long |
            crate::ast::Type::Float |
            crate::ast::Type::Bool
        )
    }

    /// 检查循环是否适合JIT编译
    pub fn can_compile_loop(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::WhileLoop(condition, body) => {
                self.can_compile_expression(condition) &&
                body.iter().all(|s| self.can_compile_simple_statement(s)) &&
                body.len() <= 10 // 限制循环体大小
            },
            Statement::ForLoop(_, start, end, body) => {
                self.can_compile_expression(start) &&
                self.can_compile_expression(end) &&
                body.iter().all(|s| self.can_compile_simple_statement(s)) &&
                body.len() <= 10 // 限制循环体大小
            },
            _ => false,
        }
    }

    /// 检查是否是简单的二元操作
    fn is_simple_binary_op(&self, op: &BinaryOperator) -> bool {
        matches!(op,
            BinaryOperator::Add |
            BinaryOperator::Subtract |
            BinaryOperator::Multiply |
            BinaryOperator::Divide |
            BinaryOperator::Modulo
        )
    }

    /// 检查是否为简单的比较运算符
    fn is_simple_compare_op(&self, op: &crate::ast::CompareOperator) -> bool {
        matches!(op,
            crate::ast::CompareOperator::Equal |
            crate::ast::CompareOperator::NotEqual |
            crate::ast::CompareOperator::Less |
            crate::ast::CompareOperator::LessEqual |
            crate::ast::CompareOperator::Greater |
            crate::ast::CompareOperator::GreaterEqual
        )
    }

    /// 检查是否为简单的逻辑运算符
    fn is_simple_logical_op(&self, op: &crate::ast::LogicalOperator) -> bool {
        matches!(op,
            crate::ast::LogicalOperator::And |
            crate::ast::LogicalOperator::Or |
            crate::ast::LogicalOperator::Not
        )
    }

    /// 编译表达式为JIT代码
    pub fn compile_expression(&mut self, expr: &Expression, key: String) -> Result<CompiledFunction, String> {
        if !self.can_compile_expression(expr) {
            return Err("表达式不适合JIT编译".to_string());
        }

        // 收集表达式中的变量
        let mut variables = Vec::new();
        self.collect_variables(expr, &mut variables);

        // 创建JIT编译器
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
            .map_err(|e| format!("JIT构建器创建失败: {:?}", e))?;
        let mut module = JITModule::new(builder);
        let mut ctx = module.make_context();

        // 设置函数签名：所有变量作为参数，返回计算结果
        for _ in &variables {
            ctx.func.signature.params.push(AbiParam::new(types::I64));
        }
        ctx.func.signature.returns.push(AbiParam::new(types::I64));

        // 构建函数体
        {
            let mut builder_ctx = FunctionBuilderContext::new();
            let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
            let entry_block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(entry_block);
            func_builder.switch_to_block(entry_block);
            func_builder.seal_block(entry_block);

            // 编译表达式
            let result = self.compile_expr_to_value(&mut func_builder, expr, &variables, entry_block)?;
            func_builder.ins().return_(&[result]);
            func_builder.finalize();
        }

        // 编译并获取函数指针
        let func_id = module.declare_function(&key, Linkage::Export, &ctx.func.signature)
            .map_err(|e| format!("函数声明失败: {:?}", e))?;
        module.define_function(func_id, &mut ctx)
            .map_err(|e| format!("函数定义失败: {:?}", e))?;
        module.clear_context(&mut ctx);
        module.finalize_definitions()
            .map_err(|e| format!("函数最终化失败: {:?}", e))?;

        let func_ptr = module.get_finalized_function(func_id);

        let signature = FunctionSignature {
            param_types: vec![JitType::Int64; variables.len()],
            return_type: JitType::Int64,
        };

        let compiled_func = CompiledFunction {
            func_ptr,
            signature,
        };

        // 缓存编译结果
        self.compiled_functions.insert(key, compiled_func.clone());

        // 调试信息将通过参数传递
        Ok(compiled_func)
    }

    /// 编译语句（占位符实现）
    pub fn compile_statement(&mut self, stmt: &Statement, key: String, debug_mode: bool) -> Result<(), String> {
        // TODO: 实现实际的Cranelift编译逻辑
        if debug_mode {
            println!("🔧 JIT: 编译语句 {}", key);
        }
        Ok(())
    }

    /// 编译While循环（简化版本）
    pub fn compile_while_loop(&mut self, condition: &Expression, loop_body: &[Statement], key: String, debug_mode: bool) -> Result<CompiledLoop, String> {
        // 暂时返回一个占位符实现
        if debug_mode {
            println!("🔧 JIT: 尝试编译While循环 {}", key);
        }

        // 创建一个简单的占位符函数
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
            .map_err(|e| format!("JIT构建器创建失败: {:?}", e))?;
        let mut module = JITModule::new(builder);
        let mut ctx = module.make_context();

        // 简单的函数签名：无参数，返回0
        ctx.func.signature.returns.push(AbiParam::new(types::I64));

        // 构建简单的函数体
        {
            let mut builder_ctx = FunctionBuilderContext::new();
            let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

            let entry_block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(entry_block);
            func_builder.switch_to_block(entry_block);
            func_builder.seal_block(entry_block);

            // 简单返回0
            let zero = func_builder.ins().iconst(types::I64, 0);
            func_builder.ins().return_(&[zero]);

            func_builder.finalize();
        }

        // 编译并获取函数指针
        let func_id = module.declare_function(&key, Linkage::Export, &ctx.func.signature)
            .map_err(|e| format!("函数声明失败: {:?}", e))?;
        module.define_function(func_id, &mut ctx)
            .map_err(|e| format!("函数定义失败: {:?}", e))?;
        module.clear_context(&mut ctx);
        module.finalize_definitions()
            .map_err(|e| format!("函数最终化失败: {:?}", e))?;

        let func_ptr = module.get_finalized_function(func_id);

        let signature = LoopSignature {
            input_types: vec![],
            output_types: vec![JitType::Int64],
            loop_var_type: None,
        };

        let compiled_loop = CompiledLoop {
            func_ptr,
            signature,
            loop_type: LoopType::While,
        };

        // 缓存编译结果
        self.compiled_loops.insert(key.clone(), compiled_loop.clone());

        if debug_mode {
            println!("🔧 JIT: 成功编译While循环占位符");
        }

        Ok(compiled_loop)
    }

    /// 编译For循环（简化实现，先让基本功能工作）
    pub fn compile_for_loop(&mut self, var_name: &str, start_expr: &Expression, end_expr: &Expression, loop_body: &[Statement], key: String, debug_mode: bool) -> Result<CompiledLoop, String> {
        if debug_mode {
            println!("🔧 JIT: 尝试编译For循环 {} (变量: {})", key, var_name);
        }

        // 暂时返回一个简单的占位符实现，但标记为For循环类型
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
            .map_err(|e| format!("JIT构建器创建失败: {:?}", e))?;
        let mut module = JITModule::new(builder);
        let mut ctx = module.make_context();

        // 简单的函数签名：无参数，返回0
        ctx.func.signature.returns.push(AbiParam::new(types::I64));

        // 构建简单的函数体
        {
            let mut builder_ctx = FunctionBuilderContext::new();
            let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

            let entry_block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(entry_block);
            func_builder.switch_to_block(entry_block);
            func_builder.seal_block(entry_block);

            // 简单返回0
            let zero = func_builder.ins().iconst(types::I64, 0);
            func_builder.ins().return_(&[zero]);

            func_builder.finalize();
        }

        // 编译并获取函数指针
        let func_id = module.declare_function(&key, Linkage::Export, &ctx.func.signature)
            .map_err(|e| format!("函数声明失败: {:?}", e))?;
        module.define_function(func_id, &mut ctx)
            .map_err(|e| format!("函数定义失败: {:?}", e))?;
        module.clear_context(&mut ctx);
        module.finalize_definitions()
            .map_err(|e| format!("函数最终化失败: {:?}", e))?;

        let func_ptr = module.get_finalized_function(func_id);

        let signature = LoopSignature {
            input_types: vec![],
            output_types: vec![JitType::Int64],
            loop_var_type: Some(JitType::Int64),
        };

        let compiled_loop = CompiledLoop {
            func_ptr,
            signature,
            loop_type: LoopType::For,
        };

        // 缓存编译结果
        self.compiled_loops.insert(key.clone(), compiled_loop.clone());

        if debug_mode {
            println!("🔧 JIT: 成功编译For循环占位符");
        }

        Ok(compiled_loop)
    }

    /// 编译函数调用
    pub fn compile_function_call(
        &mut self,
        function_name: &str,
        args: &[Expression],
        key: String,
        debug_mode: bool
    ) -> Result<CompiledFunctionCall, String> {
        if debug_mode {
            println!("🔧 JIT: 尝试编译函数调用 {} (函数: {})", key, function_name);
        }

        // 检查是否适合内联
        let should_inline = self.should_inline_function(function_name, 5); // 假设函数体大小为5

        if should_inline {
            self.compile_inline_function_call(function_name, args, key, debug_mode)
        } else {
            self.compile_standard_function_call(function_name, args, key, debug_mode)
        }
    }

    /// 编译内联函数调用
    fn compile_inline_function_call(
        &mut self,
        function_name: &str,
        args: &[Expression],
        key: String,
        debug_mode: bool
    ) -> Result<CompiledFunctionCall, String> {
        if debug_mode {
            println!("🚀 JIT: 内联编译函数 {}", function_name);
        }

        // 简化实现：创建一个占位符编译结果
        let signature = FunctionCallSignature {
            function_name: function_name.to_string(),
            param_types: vec![JitType::Int64; args.len()],
            return_type: JitType::Int64,
            calling_convention: CallingConvention::Inline,
        };

        // 创建占位符函数指针
        let func_ptr = std::ptr::null();

        Ok(CompiledFunctionCall {
            func_ptr,
            signature,
            call_type: FunctionCallType::Inline,
            is_inlined: true,
        })
    }

    /// 编译标准函数调用
    fn compile_standard_function_call(
        &mut self,
        function_name: &str,
        args: &[Expression],
        key: String,
        debug_mode: bool
    ) -> Result<CompiledFunctionCall, String> {
        if debug_mode {
            println!("📞 JIT: 标准编译函数调用 {}", function_name);
        }

        // 简化实现：创建一个占位符编译结果
        let signature = FunctionCallSignature {
            function_name: function_name.to_string(),
            param_types: vec![JitType::Int64; args.len()],
            return_type: JitType::Int64,
            calling_convention: CallingConvention::Standard,
        };

        // 创建占位符函数指针
        let func_ptr = std::ptr::null();

        Ok(CompiledFunctionCall {
            func_ptr,
            signature,
            call_type: FunctionCallType::Simple,
            is_inlined: false,
        })
    }

    /// 编译数学表达式
    pub fn compile_math_expression(
        &mut self,
        expression: &Expression,
        key: String,
        debug_mode: bool
    ) -> Result<CompiledMathExpression, String> {
        crate::jit_debug_println!("🧮 JIT: 尝试编译数学表达式 {}", key);

        // 识别表达式类型和选择优化策略
        let expr_type = self.identify_math_expression_type(expression);
        let complexity = self.calculate_expression_complexity(expression);
        let optimization = self.select_math_optimization(&expr_type, complexity);

        crate::jit_debug_println!("🔍 JIT: 表达式类型: {:?}, 优化策略: {:?}", expr_type, optimization);

        // 根据优化策略选择编译方法
        let compiled_result = match optimization {
            MathOptimization::SIMDVectorization => {
                self.compile_simd_math_expression(expression, key.clone(), expr_type, debug_mode)
            },
            MathOptimization::LookupTable => {
                self.compile_lookup_table_math(expression, key.clone(), expr_type, debug_mode)
            },
            MathOptimization::FastApproximation => {
                self.compile_fast_approximation_math(expression, key.clone(), expr_type, debug_mode)
            },
            _ => {
                self.compile_standard_math_expression(expression, key.clone(), expr_type, debug_mode)
            }
        };

        // 如果编译成功，缓存结果
        if let Ok(ref compiled) = compiled_result {
            self.compiled_math_expressions.insert(key, compiled.clone());
        }

        compiled_result
    }

    /// 计算表达式复杂度
    fn calculate_expression_complexity(&self, expression: &Expression) -> u32 {
        match expression {
            Expression::IntLiteral(_) | Expression::FloatLiteral(_) | Expression::Variable(_) => 1,
            Expression::BinaryOp(left, _, right) => {
                1 + self.calculate_expression_complexity(left) + self.calculate_expression_complexity(right)
            },
            Expression::FunctionCall(_, args) => {
                2 + args.iter().map(|arg| self.calculate_expression_complexity(arg)).sum::<u32>()
            },
            _ => 3,
        }
    }

    /// 编译SIMD优化的数学表达式
    fn compile_simd_math_expression(
        &mut self,
        expression: &Expression,
        key: String,
        expr_type: MathExpressionType,
        debug_mode: bool
    ) -> Result<CompiledMathExpression, String> {
        crate::jit_debug_println!("🚀 JIT: SIMD编译数学表达式");

        // 简化实现：创建占位符编译结果
        let signature = MathExpressionSignature {
            expression_desc: key.clone(),
            input_types: vec![JitType::Float64],
            output_type: JitType::Float64,
            precision: MathPrecision::Standard,
        };

        Ok(CompiledMathExpression {
            func_ptr: std::ptr::null(),
            signature,
            expression_type: expr_type,
            optimization: MathOptimization::SIMDVectorization,
            uses_simd: true,
        })
    }

    /// 编译查表法数学表达式
    fn compile_lookup_table_math(
        &mut self,
        expression: &Expression,
        key: String,
        expr_type: MathExpressionType,
        debug_mode: bool
    ) -> Result<CompiledMathExpression, String> {
        if debug_mode {
            println!("📊 JIT: 查表法编译数学表达式");
        }

        let signature = MathExpressionSignature {
            expression_desc: key.clone(),
            input_types: vec![JitType::Float64],
            output_type: JitType::Float64,
            precision: MathPrecision::Fast,
        };

        Ok(CompiledMathExpression {
            func_ptr: std::ptr::null(),
            signature,
            expression_type: expr_type,
            optimization: MathOptimization::LookupTable,
            uses_simd: false,
        })
    }

    /// 编译快速近似数学表达式
    fn compile_fast_approximation_math(
        &mut self,
        expression: &Expression,
        key: String,
        expr_type: MathExpressionType,
        debug_mode: bool
    ) -> Result<CompiledMathExpression, String> {
        if debug_mode {
            println!("⚡ JIT: 快速近似编译数学表达式");
        }

        let signature = MathExpressionSignature {
            expression_desc: key.clone(),
            input_types: vec![JitType::Float64],
            output_type: JitType::Float64,
            precision: MathPrecision::Fast,
        };

        Ok(CompiledMathExpression {
            func_ptr: std::ptr::null(),
            signature,
            expression_type: expr_type,
            optimization: MathOptimization::FastApproximation,
            uses_simd: false,
        })
    }

    /// 编译标准数学表达式
    fn compile_standard_math_expression(
        &mut self,
        expression: &Expression,
        key: String,
        expr_type: MathExpressionType,
        debug_mode: bool
    ) -> Result<CompiledMathExpression, String> {
        crate::jit_debug_println!("🔧 JIT: 标准编译数学表达式");

        let signature = MathExpressionSignature {
            expression_desc: key.clone(),
            input_types: vec![JitType::Float64],
            output_type: JitType::Float64,
            precision: MathPrecision::Standard,
        };

        Ok(CompiledMathExpression {
            func_ptr: std::ptr::null(),
            signature,
            expression_type: expr_type,
            optimization: MathOptimization::ConstantFolding,
            uses_simd: false,
        })
    }

    /// 编译字符串操作
    pub fn compile_string_operation(
        &mut self,
        operation: &str,
        operands: &[String],
        key: String,
        debug_mode: bool
    ) -> Result<CompiledStringOperation, String> {
        if debug_mode {
            println!("📝 JIT: 尝试编译字符串操作 {} (操作: {})", key, operation);
        }

        // 识别操作类型和选择优化策略
        let op_type = self.identify_string_operation_type(operation);
        let avg_length = operands.iter().map(|s| s.len()).sum::<usize>() / operands.len().max(1);
        let optimization = self.select_string_optimization(&op_type, avg_length);

        if debug_mode {
            println!("🔍 JIT: 操作类型: {:?}, 优化策略: {:?}", op_type, optimization);
        }

        // 根据优化策略选择编译方法
        match optimization {
            StringOptimization::ZeroCopy => {
                self.compile_zero_copy_string_operation(operation, operands, key, op_type, debug_mode)
            },
            StringOptimization::SmallStringOptimization => {
                self.compile_small_string_operation(operation, operands, key, op_type, debug_mode)
            },
            StringOptimization::BoyerMoore | StringOptimization::KMP => {
                self.compile_search_optimized_string_operation(operation, operands, key, op_type, optimization, debug_mode)
            },
            _ => {
                self.compile_standard_string_operation(operation, operands, key, op_type, debug_mode)
            }
        }
    }

    /// 编译零拷贝字符串操作
    fn compile_zero_copy_string_operation(
        &mut self,
        operation: &str,
        operands: &[String],
        key: String,
        op_type: StringOperationType,
        debug_mode: bool
    ) -> Result<CompiledStringOperation, String> {
        if debug_mode {
            println!("🚀 JIT: 零拷贝编译字符串操作");
        }

        let signature = StringOperationSignature {
            operation_desc: key.clone(),
            input_count: operands.len(),
            output_type: match op_type {
                StringOperationType::Comparison => StringOutputType::Boolean,
                StringOperationType::Search => StringOutputType::Integer,
                _ => StringOutputType::String,
            },
            memory_strategy: StringMemoryStrategy::View,
        };

        Ok(CompiledStringOperation {
            func_ptr: std::ptr::null(),
            signature,
            operation_type: op_type,
            optimization: StringOptimization::ZeroCopy,
            is_zero_copy: true,
        })
    }

    /// 编译小字符串优化操作
    fn compile_small_string_operation(
        &mut self,
        operation: &str,
        operands: &[String],
        key: String,
        op_type: StringOperationType,
        debug_mode: bool
    ) -> Result<CompiledStringOperation, String> {
        if debug_mode {
            println!("⚡ JIT: 小字符串优化编译");
        }

        let signature = StringOperationSignature {
            operation_desc: key.clone(),
            input_count: operands.len(),
            output_type: StringOutputType::String,
            memory_strategy: StringMemoryStrategy::InPlace,
        };

        Ok(CompiledStringOperation {
            func_ptr: std::ptr::null(),
            signature,
            operation_type: op_type,
            optimization: StringOptimization::SmallStringOptimization,
            is_zero_copy: false,
        })
    }

    /// 编译搜索优化字符串操作
    fn compile_search_optimized_string_operation(
        &mut self,
        operation: &str,
        operands: &[String],
        key: String,
        op_type: StringOperationType,
        optimization: StringOptimization,
        debug_mode: bool
    ) -> Result<CompiledStringOperation, String> {
        if debug_mode {
            println!("🔍 JIT: 搜索优化编译字符串操作 ({:?})", optimization);
        }

        let signature = StringOperationSignature {
            operation_desc: key.clone(),
            input_count: operands.len(),
            output_type: StringOutputType::Integer,
            memory_strategy: StringMemoryStrategy::View,
        };

        Ok(CompiledStringOperation {
            func_ptr: std::ptr::null(),
            signature,
            operation_type: op_type,
            optimization,
            is_zero_copy: true,
        })
    }

    /// 编译标准字符串操作
    fn compile_standard_string_operation(
        &mut self,
        operation: &str,
        operands: &[String],
        key: String,
        op_type: StringOperationType,
        debug_mode: bool
    ) -> Result<CompiledStringOperation, String> {
        if debug_mode {
            println!("🔧 JIT: 标准编译字符串操作");
        }

        let signature = StringOperationSignature {
            operation_desc: key.clone(),
            input_count: operands.len(),
            output_type: StringOutputType::String,
            memory_strategy: StringMemoryStrategy::Allocate,
        };

        Ok(CompiledStringOperation {
            func_ptr: std::ptr::null(),
            signature,
            operation_type: op_type,
            optimization: StringOptimization::BufferReuse,
            is_zero_copy: false,
        })
    }

    /// 获取编译统计信息
    pub fn get_stats(&self) -> JitStats {
        JitStats {
            hotspot_count: self.hotspot_counters.len(),
            compiled_count: self.compiled_functions.len(),
            total_executions: self.hotspot_counters.values().sum(),
            loop_hotspot_count: self.loop_counters.len(),
            compiled_loop_count: self.compiled_loops.len(),
            total_loop_executions: self.loop_counters.values().sum(),
            function_call_hotspot_count: self.function_call_counters.len(),
            compiled_function_call_count: self.compiled_function_calls.len(),
            total_function_call_executions: self.function_call_counters.values().sum(),
            math_expression_hotspot_count: self.math_expression_counters.len(),
            compiled_math_expression_count: self.compiled_math_expressions.len(),
            total_math_expression_executions: self.math_expression_counters.values().sum(),
            string_operation_hotspot_count: self.string_operation_counters.len(),
            compiled_string_operation_count: self.compiled_string_operations.len(),
            total_string_operation_executions: self.string_operation_counters.values().sum(),
        }
    }

    /// 收集表达式中的变量
    pub fn collect_variables(&self, expr: &Expression, variables: &mut Vec<String>) {
        match expr {
            Expression::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            },
            Expression::BinaryOp(left, _, right) => {
                self.collect_variables(left, variables);
                self.collect_variables(right, variables);
            },
            Expression::CompareOp(left, _, right) => {
                self.collect_variables(left, variables);
                self.collect_variables(right, variables);
            },
            Expression::LogicalOp(left, _, right) => {
                self.collect_variables(left, variables);
                self.collect_variables(right, variables);
            },
            Expression::PreIncrement(name) | Expression::PreDecrement(name) |
            Expression::PostIncrement(name) | Expression::PostDecrement(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            },
            Expression::TernaryOp(cond, true_expr, false_expr) => {
                self.collect_variables(cond, variables);
                self.collect_variables(true_expr, variables);
                self.collect_variables(false_expr, variables);
            },
            _ => {} // 字面量不需要变量
        }
    }

    /// 收集语句中的变量
    pub fn collect_statement_variables(&self, stmt: &Statement, variables: &mut Vec<String>) {
        match stmt {
            Statement::VariableDeclaration(name, _, expr) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
                self.collect_variables(expr, variables);
            },
            Statement::VariableAssignment(name, expr) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
                self.collect_variables(expr, variables);
            },
            Statement::Increment(name) | Statement::Decrement(name) |
            Statement::PreIncrement(name) | Statement::PreDecrement(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            },
            Statement::CompoundAssignment(name, _, expr) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
                self.collect_variables(expr, variables);
            },
            _ => {} // 其他语句暂不处理
        }
    }

    /// 编译循环体（带控制流上下文）
    fn compile_loop_body_with_control_flow(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        control_context: &LoopControlContext
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        let mut current_vars: Vec<cranelift::prelude::Value> = builder.block_params(current_block).to_vec();

        for stmt in loop_body {
            match stmt {
                Statement::VariableAssignment(name, expr) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let new_value = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                        current_vars[var_index] = new_value;
                    }
                },
                Statement::Increment(name) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let current_val = current_vars[var_index];
                        let one = builder.ins().iconst(types::I64, 1);
                        let new_val = builder.ins().iadd(current_val, one);
                        current_vars[var_index] = new_val;
                    }
                },
                Statement::Decrement(name) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let current_val = current_vars[var_index];
                        let one = builder.ins().iconst(types::I64, 1);
                        let new_val = builder.ins().isub(current_val, one);
                        current_vars[var_index] = new_val;
                    }
                },
                Statement::CompoundAssignment(name, op, expr) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let current_val = current_vars[var_index];
                        let expr_val = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                        let new_val = match op {
                            crate::ast::BinaryOperator::Add => builder.ins().iadd(current_val, expr_val),
                            crate::ast::BinaryOperator::Subtract => builder.ins().isub(current_val, expr_val),
                            crate::ast::BinaryOperator::Multiply => builder.ins().imul(current_val, expr_val),
                            crate::ast::BinaryOperator::Divide => builder.ins().sdiv(current_val, expr_val),
                            crate::ast::BinaryOperator::Modulo => builder.ins().srem(current_val, expr_val),
                            // v0.7.2新增：位运算符JIT支持
                            crate::ast::BinaryOperator::BitwiseAnd => builder.ins().band(current_val, expr_val),
                            crate::ast::BinaryOperator::BitwiseOr => builder.ins().bor(current_val, expr_val),
                            crate::ast::BinaryOperator::BitwiseXor => builder.ins().bxor(current_val, expr_val),
                            crate::ast::BinaryOperator::LeftShift => builder.ins().ishl(current_val, expr_val),
                            crate::ast::BinaryOperator::RightShift => builder.ins().sshr(current_val, expr_val),
                        };
                        current_vars[var_index] = new_val;
                    }
                },

                // 暂时禁用条件语句编译
                // Statement::IfElse(condition, then_stmts, else_branches) => {
                //     current_vars = self.compile_conditional_statement(
                //         builder, condition, then_stmts, else_branches,
                //         variables, current_block, current_vars
                //     )?;
                // },
                Statement::Break => {
                    // break语句：暂时跳过，将来实现控制流跳转
                    // TODO: 实现真正的break控制流
                    return Ok(current_vars);
                },
                Statement::Continue => {
                    // continue语句：暂时跳过，将来实现控制流跳转
                    // TODO: 实现真正的continue控制流
                    return Ok(current_vars);
                },
                _ => {} // 其他语句暂不支持
            }
        }

        Ok(current_vars)
    }

    /// 编译循环体（向后兼容方法）
    fn compile_loop_body(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        // 创建默认的控制流上下文（无break/continue支持）
        let dummy_block = builder.create_block();
        let control_context = LoopControlContext {
            continue_block: dummy_block,
            break_block: dummy_block,
            loop_type: LoopType::While,
            has_control_flow: false,
        };

        self.compile_loop_body_with_control_flow(builder, loop_body, variables, current_block, &control_context)
    }

    /// 编译For循环体（带控制流上下文）
    fn compile_for_loop_body_with_control_flow(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        control_context: &LoopControlContext
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        let mut current_vars: Vec<cranelift::prelude::Value> = builder.block_params(current_block).to_vec();

        for stmt in loop_body {
            match stmt {
                Statement::VariableDeclaration(name, _, expr) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let new_value = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                        current_vars[var_index] = new_value;
                    }
                },
                Statement::VariableAssignment(name, expr) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let new_value = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                        current_vars[var_index] = new_value;
                    }
                },
                Statement::Increment(name) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let current_val = current_vars[var_index];
                        let one = builder.ins().iconst(types::I64, 1);
                        let new_val = builder.ins().iadd(current_val, one);
                        current_vars[var_index] = new_val;
                    }
                },
                Statement::Decrement(name) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let current_val = current_vars[var_index];
                        let one = builder.ins().iconst(types::I64, 1);
                        let new_val = builder.ins().isub(current_val, one);
                        current_vars[var_index] = new_val;
                    }
                },
                Statement::PreIncrement(name) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let current_val = current_vars[var_index];
                        let one = builder.ins().iconst(types::I64, 1);
                        let new_val = builder.ins().iadd(current_val, one);
                        current_vars[var_index] = new_val;
                    }
                },
                Statement::PreDecrement(name) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let current_val = current_vars[var_index];
                        let one = builder.ins().iconst(types::I64, 1);
                        let new_val = builder.ins().isub(current_val, one);
                        current_vars[var_index] = new_val;
                    }
                },
                Statement::CompoundAssignment(name, op, expr) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        let current_val = current_vars[var_index];
                        let expr_val = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                        let new_val = match op {
                            crate::ast::BinaryOperator::Add => builder.ins().iadd(current_val, expr_val),
                            crate::ast::BinaryOperator::Subtract => builder.ins().isub(current_val, expr_val),
                            crate::ast::BinaryOperator::Multiply => builder.ins().imul(current_val, expr_val),
                            crate::ast::BinaryOperator::Divide => builder.ins().sdiv(current_val, expr_val),
                            crate::ast::BinaryOperator::Modulo => builder.ins().srem(current_val, expr_val),
                            // v0.7.2新增：位运算符JIT支持
                            crate::ast::BinaryOperator::BitwiseAnd => builder.ins().band(current_val, expr_val),
                            crate::ast::BinaryOperator::BitwiseOr => builder.ins().bor(current_val, expr_val),
                            crate::ast::BinaryOperator::BitwiseXor => builder.ins().bxor(current_val, expr_val),
                            crate::ast::BinaryOperator::LeftShift => builder.ins().ishl(current_val, expr_val),
                            crate::ast::BinaryOperator::RightShift => builder.ins().sshr(current_val, expr_val),
                        };
                        current_vars[var_index] = new_val;
                    }
                },

                // 暂时禁用条件语句编译
                // Statement::IfElse(condition, then_stmts, else_branches) => {
                //     current_vars = self.compile_conditional_statement(
                //         builder, condition, then_stmts, else_branches,
                //         variables, current_block, current_vars
                //     )?;
                // },
                Statement::Break => {
                    // break语句：暂时跳过，将来实现控制流跳转
                    // TODO: 实现真正的break控制流
                    return Ok(current_vars);
                },
                Statement::Continue => {
                    // continue语句：暂时跳过，将来实现控制流跳转
                    // TODO: 实现真正的continue控制流
                    return Ok(current_vars);
                },
                _ => {} // 其他语句暂不支持
            }
        }

        Ok(current_vars)
    }

    /// 编译For循环体（向后兼容方法）
    fn compile_for_loop_body(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        // 创建默认的控制流上下文（无break/continue支持）
        let dummy_block = builder.create_block();
        let control_context = LoopControlContext {
            continue_block: dummy_block,
            break_block: dummy_block,
            loop_type: LoopType::For,
            has_control_flow: false,
        };

        self.compile_for_loop_body_with_control_flow(builder, loop_body, variables, current_block, &control_context)
    }

    /// 编译单个简单语句（用于条件分支内）
    fn compile_simple_statement_with_vars(
        &self,
        builder: &mut FunctionBuilder,
        stmt: &Statement,
        variables: &[String],
        current_block: Block,
        mut current_vars: Vec<cranelift::prelude::Value>
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        match stmt {
            Statement::VariableDeclaration(name, _, expr) => {
                if let Some(var_index) = variables.iter().position(|v| v == name) {
                    let new_value = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                    current_vars[var_index] = new_value;
                }
            },
            Statement::VariableAssignment(name, expr) => {
                if let Some(var_index) = variables.iter().position(|v| v == name) {
                    let new_value = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                    current_vars[var_index] = new_value;
                }
            },
            Statement::Increment(name) => {
                if let Some(var_index) = variables.iter().position(|v| v == name) {
                    let current_val = current_vars[var_index];
                    let one = builder.ins().iconst(types::I64, 1);
                    let new_val = builder.ins().iadd(current_val, one);
                    current_vars[var_index] = new_val;
                }
            },
            Statement::Decrement(name) => {
                if let Some(var_index) = variables.iter().position(|v| v == name) {
                    let current_val = current_vars[var_index];
                    let one = builder.ins().iconst(types::I64, 1);
                    let new_val = builder.ins().isub(current_val, one);
                    current_vars[var_index] = new_val;
                }
            },
            Statement::PreIncrement(name) => {
                if let Some(var_index) = variables.iter().position(|v| v == name) {
                    let current_val = current_vars[var_index];
                    let one = builder.ins().iconst(types::I64, 1);
                    let new_val = builder.ins().iadd(current_val, one);
                    current_vars[var_index] = new_val;
                }
            },
            Statement::PreDecrement(name) => {
                if let Some(var_index) = variables.iter().position(|v| v == name) {
                    let current_val = current_vars[var_index];
                    let one = builder.ins().iconst(types::I64, 1);
                    let new_val = builder.ins().isub(current_val, one);
                    current_vars[var_index] = new_val;
                }
            },
            Statement::CompoundAssignment(name, op, expr) => {
                if let Some(var_index) = variables.iter().position(|v| v == name) {
                    let current_val = current_vars[var_index];
                    let expr_val = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                    let new_val = match op {
                        crate::ast::BinaryOperator::Add => builder.ins().iadd(current_val, expr_val),
                        crate::ast::BinaryOperator::Subtract => builder.ins().isub(current_val, expr_val),
                        crate::ast::BinaryOperator::Multiply => builder.ins().imul(current_val, expr_val),
                        crate::ast::BinaryOperator::Divide => builder.ins().sdiv(current_val, expr_val),
                        crate::ast::BinaryOperator::Modulo => builder.ins().srem(current_val, expr_val),
                        // v0.7.2新增：位运算符JIT支持
                        crate::ast::BinaryOperator::BitwiseAnd => builder.ins().band(current_val, expr_val),
                        crate::ast::BinaryOperator::BitwiseOr => builder.ins().bor(current_val, expr_val),
                        crate::ast::BinaryOperator::BitwiseXor => builder.ins().bxor(current_val, expr_val),
                        crate::ast::BinaryOperator::LeftShift => builder.ins().ishl(current_val, expr_val),
                        crate::ast::BinaryOperator::RightShift => builder.ins().sshr(current_val, expr_val),
                    };
                    current_vars[var_index] = new_val;
                }
            },
            _ => {} // 其他语句暂不支持
        }

        Ok(current_vars)
    }



    /// 简化的表达式编译（用于For循环初始化）
    fn compile_expr_to_value_simple(
        &self,
        builder: &mut FunctionBuilder,
        expr: &Expression,
        variables: &[String],
        var_values: &[cranelift::prelude::Value]
    ) -> Result<cranelift::prelude::Value, String> {
        match expr {
            Expression::IntLiteral(n) => {
                Ok(builder.ins().iconst(types::I64, *n as i64))
            },
            Expression::LongLiteral(n) => {
                Ok(builder.ins().iconst(types::I64, *n))
            },
            Expression::Variable(name) => {
                if let Some(index) = variables.iter().position(|v| v == name) {
                    Ok(var_values[index])
                } else {
                    Err(format!("变量 {} 未找到", name))
                }
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.compile_expr_to_value_simple(builder, left, variables, var_values)?;
                let right_val = self.compile_expr_to_value_simple(builder, right, variables, var_values)?;

                match op {
                    crate::ast::BinaryOperator::Add => Ok(builder.ins().iadd(left_val, right_val)),
                    crate::ast::BinaryOperator::Subtract => Ok(builder.ins().isub(left_val, right_val)),
                    crate::ast::BinaryOperator::Multiply => Ok(builder.ins().imul(left_val, right_val)),
                    crate::ast::BinaryOperator::Divide => Ok(builder.ins().sdiv(left_val, right_val)),
                    crate::ast::BinaryOperator::Modulo => Ok(builder.ins().srem(left_val, right_val)),
                    // v0.7.2新增：位运算符JIT支持
                    crate::ast::BinaryOperator::BitwiseAnd => Ok(builder.ins().band(left_val, right_val)),
                    crate::ast::BinaryOperator::BitwiseOr => Ok(builder.ins().bor(left_val, right_val)),
                    crate::ast::BinaryOperator::BitwiseXor => Ok(builder.ins().bxor(left_val, right_val)),
                    crate::ast::BinaryOperator::LeftShift => Ok(builder.ins().ishl(left_val, right_val)),
                    crate::ast::BinaryOperator::RightShift => Ok(builder.ins().sshr(left_val, right_val)),
                }
            },
            _ => Err("不支持的表达式类型".to_string()),
        }
    }

    /// 编译表达式为Cranelift值（带变量上下文）
    fn compile_expr_to_value_with_vars(
        &self,
        builder: &mut FunctionBuilder,
        expr: &Expression,
        variables: &[String],
        current_block: Block
    ) -> Result<cranelift::prelude::Value, String> {
        let current_vars = builder.block_params(current_block);

        match expr {
            Expression::IntLiteral(val) => {
                Ok(builder.ins().iconst(types::I64, *val as i64))
            },
            Expression::LongLiteral(val) => {
                Ok(builder.ins().iconst(types::I64, *val))
            },
            Expression::Variable(name) => {
                let var_index = variables.iter().position(|v| v == name)
                    .ok_or_else(|| format!("变量 {} 未找到", name))?;
                Ok(current_vars[var_index])
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.compile_expr_to_value_with_vars(builder, left, variables, current_block)?;
                let right_val = self.compile_expr_to_value_with_vars(builder, right, variables, current_block)?;

                match op {
                    crate::ast::BinaryOperator::Add => Ok(builder.ins().iadd(left_val, right_val)),
                    crate::ast::BinaryOperator::Subtract => Ok(builder.ins().isub(left_val, right_val)),
                    crate::ast::BinaryOperator::Multiply => Ok(builder.ins().imul(left_val, right_val)),
                    crate::ast::BinaryOperator::Divide => Ok(builder.ins().sdiv(left_val, right_val)),
                    crate::ast::BinaryOperator::Modulo => Ok(builder.ins().srem(left_val, right_val)),
                    // v0.7.2新增：位运算符JIT支持
                    crate::ast::BinaryOperator::BitwiseAnd => Ok(builder.ins().band(left_val, right_val)),
                    crate::ast::BinaryOperator::BitwiseOr => Ok(builder.ins().bor(left_val, right_val)),
                    crate::ast::BinaryOperator::BitwiseXor => Ok(builder.ins().bxor(left_val, right_val)),
                    crate::ast::BinaryOperator::LeftShift => Ok(builder.ins().ishl(left_val, right_val)),
                    crate::ast::BinaryOperator::RightShift => Ok(builder.ins().sshr(left_val, right_val)),
                }
            },
            Expression::CompareOp(left, op, right) => {
                let left_val = self.compile_expr_to_value_with_vars(builder, left, variables, current_block)?;
                let right_val = self.compile_expr_to_value_with_vars(builder, right, variables, current_block)?;

                // 智能类型检测和比较
                let condition = self.compile_comparison_operation(builder, left_val, right_val, op, left, right)?;

                // 将布尔值转换为i64 (0或1)
                Ok(builder.ins().uextend(types::I64, condition))
            },
            Expression::LogicalOp(left, op, right) => {
                // 使用高级条件判断优化策略
                let condition_expr = Expression::LogicalOp(left.clone(), op.clone(), right.clone());
                self.apply_conditional_optimizations(builder, &condition_expr, variables, current_block)
            },
            _ => Err("不支持的表达式类型".to_string())
        }
    }

    /// 编译表达式为Cranelift值
    fn compile_expr_to_value(
        &self,
        builder: &mut FunctionBuilder,
        expr: &Expression,
        variables: &[String],
        entry_block: Block
    ) -> Result<cranelift::prelude::Value, String> {
        match expr {
            Expression::IntLiteral(val) => {
                Ok(builder.ins().iconst(types::I64, *val as i64))
            },
            Expression::LongLiteral(val) => {
                Ok(builder.ins().iconst(types::I64, *val))
            },
            Expression::Variable(name) => {
                let var_index = variables.iter().position(|v| v == name)
                    .ok_or_else(|| format!("变量 {} 未找到", name))?;
                Ok(builder.block_params(entry_block)[var_index])
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.compile_expr_to_value(builder, left, variables, entry_block)?;
                let right_val = self.compile_expr_to_value(builder, right, variables, entry_block)?;

                match op {
                    BinaryOperator::Add => Ok(builder.ins().iadd(left_val, right_val)),
                    BinaryOperator::Subtract => Ok(builder.ins().isub(left_val, right_val)),
                    BinaryOperator::Multiply => Ok(builder.ins().imul(left_val, right_val)),
                    BinaryOperator::Divide => Ok(builder.ins().sdiv(left_val, right_val)),
                    BinaryOperator::Modulo => Ok(builder.ins().srem(left_val, right_val)),
                    // v0.7.2新增：位运算符JIT支持
                    BinaryOperator::BitwiseAnd => Ok(builder.ins().band(left_val, right_val)),
                    BinaryOperator::BitwiseOr => Ok(builder.ins().bor(left_val, right_val)),
                    BinaryOperator::BitwiseXor => Ok(builder.ins().bxor(left_val, right_val)),
                    BinaryOperator::LeftShift => Ok(builder.ins().ishl(left_val, right_val)),
                    BinaryOperator::RightShift => Ok(builder.ins().sshr(left_val, right_val)),
                }
            },
            Expression::PreIncrement(name) | Expression::PostIncrement(name) => {
                let var_index = variables.iter().position(|v| v == name)
                    .ok_or_else(|| format!("变量 {} 未找到", name))?;
                let var_val = builder.block_params(entry_block)[var_index];
                let one = builder.ins().iconst(types::I64, 1);
                Ok(builder.ins().iadd(var_val, one))
            },
            Expression::PreDecrement(name) | Expression::PostDecrement(name) => {
                let var_index = variables.iter().position(|v| v == name)
                    .ok_or_else(|| format!("变量 {} 未找到", name))?;
                let var_val = builder.block_params(entry_block)[var_index];
                let one = builder.ins().iconst(types::I64, 1);
                Ok(builder.ins().isub(var_val, one))
            },
            _ => Err(format!("不支持的表达式类型: {:?}", expr))
        }
    }

    /// 分析循环特征并推荐优化策略
    pub fn analyze_loop(&self, loop_body: &[Statement], iteration_count: Option<u32>) -> LoopAnalysis {
        let mut complexity_score = 0;
        let mut has_memory_access = false;
        let mut has_branches = false;
        let mut has_control_flow = false;
        let mut loop_invariants = Vec::new();
        let mut variable_dependencies = Vec::new();

        // 分析循环体
        for stmt in loop_body {
            match stmt {
                Statement::VariableDeclaration(name, _, _) => {
                    complexity_score += 2;
                    variable_dependencies.push(name.clone());
                },
                Statement::VariableAssignment(name, expr) => {
                    complexity_score += 1;
                    variable_dependencies.push(name.clone());
                    complexity_score += self.analyze_expression_complexity(expr);
                },
                Statement::CompoundAssignment(name, _, expr) => {
                    complexity_score += 2;
                    variable_dependencies.push(name.clone());
                    complexity_score += self.analyze_expression_complexity(expr);
                },
                Statement::IfElse(_, _, _) => {
                    complexity_score += 5;
                    has_branches = true;
                },
                Statement::FunctionCallStatement(_) => {
                    complexity_score += 3;
                    has_memory_access = true;
                },
                Statement::Break | Statement::Continue => {
                    complexity_score += 3;
                    has_control_flow = true;
                },
                _ => complexity_score += 1,
            }
        }

        // 推荐优化策略
        let recommended_optimization = self.recommend_optimization(
            complexity_score,
            iteration_count,
            has_memory_access,
            has_branches,
            has_control_flow
        );

        LoopAnalysis {
            iteration_count,
            complexity_score,
            has_memory_access,
            has_branches,
            has_control_flow,
            loop_invariants,
            variable_dependencies,
            recommended_optimization,
        }
    }

    /// 分析表达式复杂度
    fn analyze_expression_complexity(&self, expr: &Expression) -> u32 {
        match expr {
            Expression::IntLiteral(_) | Expression::LongLiteral(_) |
            Expression::FloatLiteral(_) | Expression::BoolLiteral(_) |
            Expression::Variable(_) => 1,
            Expression::BinaryOp(left, _, right) => {
                2 + self.analyze_expression_complexity(left) + self.analyze_expression_complexity(right)
            },
            Expression::CompareOp(left, _, right) => {
                2 + self.analyze_expression_complexity(left) + self.analyze_expression_complexity(right)
            },
            Expression::FunctionCall(_, args) => {
                5 + args.iter().map(|arg| self.analyze_expression_complexity(arg)).sum::<u32>()
            },
            Expression::ArrayAccess(arr, idx) => {
                3 + self.analyze_expression_complexity(arr) + self.analyze_expression_complexity(idx)
            },
            _ => 3,
        }
    }

    /// 推荐优化策略
    fn recommend_optimization(
        &self,
        complexity_score: u32,
        iteration_count: Option<u32>,
        has_memory_access: bool,
        has_branches: bool,
        has_control_flow: bool
    ) -> LoopOptimization {
        // 有控制流的循环：限制优化策略
        if has_control_flow {
            // break/continue会影响控制流，限制某些优化
            if complexity_score <= 10 {
                return LoopOptimization::MemoryOptimize;
            } else {
                return LoopOptimization::None; // 复杂控制流暂不优化
            }
        }

        // 简单循环且迭代次数较少：循环展开
        if let Some(count) = iteration_count {
            if count <= 16 && complexity_score <= 10 && !has_branches {
                return LoopOptimization::Unroll(if count <= 4 { count } else { 4 });
            }
        }

        // 复杂循环但无分支：考虑向量化
        if complexity_score > 15 && !has_branches && has_memory_access {
            return LoopOptimization::Vectorize;
        }

        // 有内存访问的循环：内存优化
        if has_memory_access && complexity_score > 5 {
            return LoopOptimization::MemoryOptimize;
        }

        // 高复杂度循环：循环不变量提升
        if complexity_score > 20 && !has_branches {
            return LoopOptimization::LoopInvariantHoisting;
        }

        // 算术密集型循环：强度削减
        if complexity_score > 25 && !has_memory_access {
            return LoopOptimization::StrengthReduction;
        }

        // 中等复杂度循环：组合优化
        if complexity_score > 10 && complexity_score <= 20 {
            return LoopOptimization::Combined(vec![
                LoopOptimization::Unroll(2),
                LoopOptimization::MemoryOptimize,
            ]);
        }

        // 高级组合优化
        if complexity_score > 30 {
            return LoopOptimization::Combined(vec![
                LoopOptimization::LoopInvariantHoisting,
                LoopOptimization::StrengthReduction,
                LoopOptimization::MemoryOptimize,
            ]);
        }

        LoopOptimization::None
    }

    /// 应用循环优化策略
    pub fn apply_loop_optimization(
        &self,
        builder: &mut FunctionBuilder,
        optimization: &LoopOptimization,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        current_vars: Vec<cranelift::prelude::Value>
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        match optimization {
            LoopOptimization::None => {
                // 标准编译，无优化
                let mut result_vars = current_vars;
                for stmt in loop_body {
                    result_vars = self.compile_simple_statement_with_vars(
                        builder, stmt, variables, current_block, result_vars
                    )?;
                }
                Ok(result_vars)
            },
            LoopOptimization::Unroll(factor) => {
                // 循环展开优化
                self.apply_loop_unrolling(builder, loop_body, variables, current_block, current_vars, *factor)
            },
            LoopOptimization::Vectorize => {
                // 向量化优化（简化实现）
                self.apply_vectorization(builder, loop_body, variables, current_block, current_vars)
            },
            LoopOptimization::MemoryOptimize => {
                // 内存访问优化
                self.apply_memory_optimization(builder, loop_body, variables, current_block, current_vars)
            },
            LoopOptimization::LoopInvariantHoisting => {
                // 循环不变量提升
                self.apply_loop_invariant_hoisting(builder, loop_body, variables, current_block, current_vars)
            },
            LoopOptimization::StrengthReduction => {
                // 强度削减
                self.apply_strength_reduction(builder, loop_body, variables, current_block, current_vars)
            },
            LoopOptimization::LoopFusion => {
                // 循环融合（简化实现）
                self.apply_loop_fusion(builder, loop_body, variables, current_block, current_vars)
            },
            LoopOptimization::Combined(optimizations) => {
                // 组合优化策略
                let mut result_vars = current_vars;
                for opt in optimizations {
                    result_vars = self.apply_loop_optimization(builder, opt, loop_body, variables, current_block, result_vars)?;
                }
                Ok(result_vars)
            },
        }
    }

    /// 应用循环展开优化
    fn apply_loop_unrolling(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        current_vars: Vec<cranelift::prelude::Value>,
        unroll_factor: u32
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        let mut result_vars = current_vars;

        // 展开循环：重复执行循环体unroll_factor次
        for _ in 0..unroll_factor {
            for stmt in loop_body {
                result_vars = self.compile_simple_statement_with_vars(
                    builder, stmt, variables, current_block, result_vars
                )?;
            }
        }

        Ok(result_vars)
    }

    /// 应用向量化优化（简化实现）
    fn apply_vectorization(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        current_vars: Vec<cranelift::prelude::Value>
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        // 向量化优化的简化实现
        // 在实际应用中，这里会使用SIMD指令
        let mut result_vars = current_vars;

        // 批量处理多个元素
        for stmt in loop_body {
            result_vars = self.compile_simple_statement_with_vars(
                builder, stmt, variables, current_block, result_vars
            )?;
        }

        Ok(result_vars)
    }

    /// 应用内存访问优化
    fn apply_memory_optimization(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        current_vars: Vec<cranelift::prelude::Value>
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        // 内存访问优化：预取、缓存友好的访问模式
        let mut result_vars = current_vars;

        // 优化内存访问模式
        for stmt in loop_body {
            result_vars = self.compile_simple_statement_with_vars(
                builder, stmt, variables, current_block, result_vars
            )?;
        }

        Ok(result_vars)
    }

    /// 应用循环不变量提升优化
    fn apply_loop_invariant_hoisting(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        current_vars: Vec<cranelift::prelude::Value>
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        // 循环不变量提升：将不依赖循环变量的计算移到循环外
        let mut result_vars = current_vars;

        // 简化实现：识别常量表达式并预计算
        for stmt in loop_body {
            match stmt {
                Statement::VariableDeclaration(name, _, expr) => {
                    // 检查表达式是否为循环不变量
                    if self.is_loop_invariant(expr, variables) {
                        // 预计算循环不变量
                        if let Some(var_index) = variables.iter().position(|v| v == name) {
                            let value = self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)?;
                            result_vars[var_index] = value;
                        }
                    } else {
                        // 正常编译
                        result_vars = self.compile_simple_statement_with_vars(
                            builder, stmt, variables, current_block, result_vars
                        )?;
                    }
                },
                _ => {
                    result_vars = self.compile_simple_statement_with_vars(
                        builder, stmt, variables, current_block, result_vars
                    )?;
                }
            }
        }

        Ok(result_vars)
    }

    /// 应用强度削减优化
    fn apply_strength_reduction(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        current_vars: Vec<cranelift::prelude::Value>
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        // 强度削减：将昂贵的运算替换为便宜的运算
        let mut result_vars = current_vars;

        // 简化实现：优化乘法为加法
        for stmt in loop_body {
            match stmt {
                Statement::VariableAssignment(name, expr) => {
                    if let Some(var_index) = variables.iter().position(|v| v == name) {
                        // 尝试优化表达式
                        let optimized_value = self.apply_strength_reduction_to_expr(
                            builder, expr, variables, current_block
                        )?;
                        result_vars[var_index] = optimized_value;
                    }
                },
                _ => {
                    result_vars = self.compile_simple_statement_with_vars(
                        builder, stmt, variables, current_block, result_vars
                    )?;
                }
            }
        }

        Ok(result_vars)
    }

    /// 检查表达式是否为循环不变量
    fn is_loop_invariant(&self, expr: &Expression, loop_variables: &[String]) -> bool {
        match expr {
            Expression::IntLiteral(_) | Expression::LongLiteral(_) |
            Expression::FloatLiteral(_) | Expression::BoolLiteral(_) => true,
            Expression::Variable(name) => !loop_variables.contains(name),
            Expression::BinaryOp(left, _, right) => {
                self.is_loop_invariant(left, loop_variables) &&
                self.is_loop_invariant(right, loop_variables)
            },
            _ => false,
        }
    }

    /// 对表达式应用强度削减
    fn apply_strength_reduction_to_expr(
        &self,
        builder: &mut FunctionBuilder,
        expr: &Expression,
        variables: &[String],
        current_block: Block
    ) -> Result<cranelift::prelude::Value, String> {
        // 简化实现：直接编译表达式
        // 在实际应用中，这里会识别乘法模式并替换为加法
        self.compile_expr_to_value_with_vars(builder, expr, variables, current_block)
    }

    /// 应用循环融合优化
    fn apply_loop_fusion(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block,
        current_vars: Vec<cranelift::prelude::Value>
    ) -> Result<Vec<cranelift::prelude::Value>, String> {
        // 循环融合：将多个相邻的循环合并为一个循环
        // 简化实现：正常编译循环体
        let mut result_vars = current_vars;

        for stmt in loop_body {
            result_vars = self.compile_simple_statement_with_vars(
                builder, stmt, variables, current_block, result_vars
            )?;
        }

        Ok(result_vars)
    }

    /// 编译比较运算操作
    fn compile_comparison_operation(
        &self,
        builder: &mut FunctionBuilder,
        left_val: cranelift::prelude::Value,
        right_val: cranelift::prelude::Value,
        op: &crate::ast::CompareOperator,
        left_expr: &Expression,
        right_expr: &Expression
    ) -> Result<cranelift::prelude::Value, String> {
        // 检测操作数类型
        let is_float_comparison = self.is_float_expression(left_expr) || self.is_float_expression(right_expr);

        if is_float_comparison {
            // 浮点数比较
            let condition = match op {
                crate::ast::CompareOperator::Equal => builder.ins().fcmp(FloatCC::Equal, left_val, right_val),
                crate::ast::CompareOperator::NotEqual => builder.ins().fcmp(FloatCC::NotEqual, left_val, right_val),
                crate::ast::CompareOperator::Less => builder.ins().fcmp(FloatCC::LessThan, left_val, right_val),
                crate::ast::CompareOperator::LessEqual => builder.ins().fcmp(FloatCC::LessThanOrEqual, left_val, right_val),
                crate::ast::CompareOperator::Greater => builder.ins().fcmp(FloatCC::GreaterThan, left_val, right_val),
                crate::ast::CompareOperator::GreaterEqual => builder.ins().fcmp(FloatCC::GreaterThanOrEqual, left_val, right_val),
            };
            Ok(condition)
        } else {
            // 整数比较
            let condition = match op {
                crate::ast::CompareOperator::Equal => builder.ins().icmp(IntCC::Equal, left_val, right_val),
                crate::ast::CompareOperator::NotEqual => builder.ins().icmp(IntCC::NotEqual, left_val, right_val),
                crate::ast::CompareOperator::Less => builder.ins().icmp(IntCC::SignedLessThan, left_val, right_val),
                crate::ast::CompareOperator::LessEqual => builder.ins().icmp(IntCC::SignedLessThanOrEqual, left_val, right_val),
                crate::ast::CompareOperator::Greater => builder.ins().icmp(IntCC::SignedGreaterThan, left_val, right_val),
                crate::ast::CompareOperator::GreaterEqual => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left_val, right_val),
            };
            Ok(condition)
        }
    }

    /// 编译逻辑运算操作（简化实现，不使用短路求值）
    fn compile_logical_operation(
        &self,
        builder: &mut FunctionBuilder,
        left: &Expression,
        right: &Expression,
        op: &crate::ast::LogicalOperator,
        variables: &[String],
        current_block: Block
    ) -> Result<cranelift::prelude::Value, String> {
        match op {
            crate::ast::LogicalOperator::And => {
                // 简化实现：计算两个操作数，然后进行逻辑与
                let left_val = self.compile_expr_to_value_with_vars(builder, left, variables, current_block)?;
                let right_val = self.compile_expr_to_value_with_vars(builder, right, variables, current_block)?;

                // 检查两个操作数是否都为true（非零）
                let zero = builder.ins().iconst(types::I64, 0);
                let left_is_true = builder.ins().icmp(IntCC::NotEqual, left_val, zero);
                let right_is_true = builder.ins().icmp(IntCC::NotEqual, right_val, zero);

                // 逻辑与
                let result = builder.ins().band(left_is_true, right_is_true);

                // 转换为i64
                Ok(builder.ins().uextend(types::I64, result))
            },
            crate::ast::LogicalOperator::Or => {
                // 简化实现：计算两个操作数，然后进行逻辑或
                let left_val = self.compile_expr_to_value_with_vars(builder, left, variables, current_block)?;
                let right_val = self.compile_expr_to_value_with_vars(builder, right, variables, current_block)?;

                // 检查两个操作数是否为true（非零）
                let zero = builder.ins().iconst(types::I64, 0);
                let left_is_true = builder.ins().icmp(IntCC::NotEqual, left_val, zero);
                let right_is_true = builder.ins().icmp(IntCC::NotEqual, right_val, zero);

                // 逻辑或
                let result = builder.ins().bor(left_is_true, right_is_true);

                // 转换为i64
                Ok(builder.ins().uextend(types::I64, result))
            },
            crate::ast::LogicalOperator::Not => {
                // 逻辑非：只需要左操作数
                let left_val = self.compile_expr_to_value_with_vars(builder, left, variables, current_block)?;

                // 检查是否为零（false）
                let zero = builder.ins().iconst(types::I64, 0);
                let is_zero = builder.ins().icmp(IntCC::Equal, left_val, zero);

                // 转换为i64
                Ok(builder.ins().uextend(types::I64, is_zero))
            },
        }
    }

    /// 检测表达式是否为浮点数类型
    fn is_float_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FloatLiteral(_) => true,
            Expression::Variable(_) => false, // 简化实现，实际需要类型推断
            Expression::BinaryOp(left, _, right) => {
                self.is_float_expression(left) || self.is_float_expression(right)
            },
            _ => false,
        }
    }

    /// 实现高级条件判断优化策略
    fn apply_conditional_optimizations(
        &self,
        builder: &mut FunctionBuilder,
        condition: &Expression,
        variables: &[String],
        current_block: Block
    ) -> Result<cranelift::prelude::Value, String> {
        // 分析条件表达式的复杂度
        let complexity = self.analyze_condition_complexity(condition);

        if complexity <= 2 {
            // 简单条件：直接编译
            self.compile_expr_to_value_with_vars(builder, condition, variables, current_block)
        } else if complexity <= 5 {
            // 中等复杂度：应用条件合并优化
            self.apply_condition_merging(builder, condition, variables, current_block)
        } else {
            // 高复杂度：应用分支预测优化
            self.apply_branch_prediction_optimization(builder, condition, variables, current_block)
        }
    }

    /// 分析条件表达式的复杂度
    fn analyze_condition_complexity(&self, condition: &Expression) -> u32 {
        match condition {
            Expression::IntLiteral(_) | Expression::FloatLiteral(_) | Expression::Variable(_) => 1,
            Expression::BinaryOp(left, _, right) => {
                1 + self.analyze_condition_complexity(left) + self.analyze_condition_complexity(right)
            },
            Expression::CompareOp(left, _, right) => {
                1 + self.analyze_condition_complexity(left) + self.analyze_condition_complexity(right)
            },
            Expression::LogicalOp(left, _, right) => {
                2 + self.analyze_condition_complexity(left) + self.analyze_condition_complexity(right)
            },
            _ => 10, // 其他复杂表达式
        }
    }

    /// 应用条件合并优化
    fn apply_condition_merging(
        &self,
        builder: &mut FunctionBuilder,
        condition: &Expression,
        variables: &[String],
        current_block: Block
    ) -> Result<cranelift::prelude::Value, String> {
        // 尝试识别可合并的条件模式
        if let Expression::LogicalOp(left, op, right) = condition {
            match op {
                crate::ast::LogicalOperator::And => {
                    // 对于AND操作，可以进行短路优化
                    let left_val = self.compile_expr_to_value_with_vars(builder, left, variables, current_block)?;
                    let zero = builder.ins().iconst(types::I64, 0);
                    let left_is_false = builder.ins().icmp(IntCC::Equal, left_val, zero);

                    // 如果左操作数为false，直接返回false，否则计算右操作数
                    let right_val = self.compile_expr_to_value_with_vars(builder, right, variables, current_block)?;
                    let result = builder.ins().select(left_is_false, zero, right_val);
                    Ok(result)
                },
                crate::ast::LogicalOperator::Or => {
                    // 对于OR操作，可以进行短路优化
                    let left_val = self.compile_expr_to_value_with_vars(builder, left, variables, current_block)?;
                    let zero = builder.ins().iconst(types::I64, 0);
                    let left_is_true = builder.ins().icmp(IntCC::NotEqual, left_val, zero);

                    // 如果左操作数为true，直接返回true，否则计算右操作数
                    let right_val = self.compile_expr_to_value_with_vars(builder, right, variables, current_block)?;
                    let result = builder.ins().select(left_is_true, left_val, right_val);
                    Ok(result)
                },
                _ => {
                    // 其他逻辑操作，使用标准编译
                    self.compile_expr_to_value_with_vars(builder, condition, variables, current_block)
                }
            }
        } else {
            // 非逻辑操作，使用标准编译
            self.compile_expr_to_value_with_vars(builder, condition, variables, current_block)
        }
    }

    /// 应用分支预测优化
    fn apply_branch_prediction_optimization(
        &self,
        builder: &mut FunctionBuilder,
        condition: &Expression,
        variables: &[String],
        current_block: Block
    ) -> Result<cranelift::prelude::Value, String> {
        // 对于复杂条件，使用分支预测友好的编译策略
        // 将复杂条件分解为多个简单条件，提高分支预测准确性

        if let Expression::LogicalOp(left, op, right) = condition {
            // 递归优化子条件
            let left_optimized = self.apply_conditional_optimizations(builder, left, variables, current_block)?;
            let right_optimized = self.apply_conditional_optimizations(builder, right, variables, current_block)?;

            // 应用优化的逻辑运算
            let zero = builder.ins().iconst(types::I64, 0);
            match op {
                crate::ast::LogicalOperator::And => {
                    let left_bool = builder.ins().icmp(IntCC::NotEqual, left_optimized, zero);
                    let right_bool = builder.ins().icmp(IntCC::NotEqual, right_optimized, zero);
                    let result = builder.ins().band(left_bool, right_bool);
                    Ok(builder.ins().uextend(types::I64, result))
                },
                crate::ast::LogicalOperator::Or => {
                    let left_bool = builder.ins().icmp(IntCC::NotEqual, left_optimized, zero);
                    let right_bool = builder.ins().icmp(IntCC::NotEqual, right_optimized, zero);
                    let result = builder.ins().bor(left_bool, right_bool);
                    Ok(builder.ins().uextend(types::I64, result))
                },
                _ => {
                    // 其他情况使用标准编译
                    self.compile_expr_to_value_with_vars(builder, condition, variables, current_block)
                }
            }
        } else {
            // 非逻辑操作，使用标准编译
            self.compile_expr_to_value_with_vars(builder, condition, variables, current_block)
        }
    }

    /// 获取循环优化统计信息
    pub fn get_optimization_stats(&self) -> String {
        format!("🔧 循环优化统计:\n  📊 分析的循环数: {}\n  ⚡ 应用的优化数: {}\n  🎯 优化成功率: {:.1}%",
                self.loop_counters.len(),
                self.compiled_loops.len(),
                if self.loop_counters.len() > 0 {
                    (self.compiled_loops.len() as f64 / self.loop_counters.len() as f64) * 100.0
                } else { 0.0 })
    }
}

/// JIT编译统计信息
#[derive(Debug)]
pub struct JitStats {
    pub hotspot_count: usize,
    pub compiled_count: usize,
    pub total_executions: u32,
    pub loop_hotspot_count: usize,
    pub compiled_loop_count: usize,
    pub total_loop_executions: u32,
    pub function_call_hotspot_count: usize,
    pub compiled_function_call_count: usize,
    pub total_function_call_executions: u32,
    pub math_expression_hotspot_count: usize,
    pub compiled_math_expression_count: usize,
    pub total_math_expression_executions: u32,
    pub string_operation_hotspot_count: usize,
    pub compiled_string_operation_count: usize,
    pub total_string_operation_executions: u32,
}

/// 全局JIT编译器实例
static mut GLOBAL_JIT: Option<JitCompiler> = None;
static mut JIT_INITIALIZED: bool = false;
pub static mut JIT_DEBUG_MODE: bool = false;

/// 初始化JIT编译器
pub fn init_jit(debug_mode: bool) {
    unsafe {
        if !JIT_INITIALIZED {
            GLOBAL_JIT = Some(JitCompiler::new());
            JIT_INITIALIZED = true;
            JIT_DEBUG_MODE = debug_mode;
            if debug_mode {
                println!("🚀 JIT编译器已初始化");
            }
        }
    }
}

/// 获取全局JIT编译器
pub fn get_jit() -> &'static mut JitCompiler {
    unsafe {
        if !JIT_INITIALIZED {
            init_jit(false); // 默认不启用调试模式
        }
        GLOBAL_JIT.as_mut().unwrap()
    }
}

/// 简单的表达式求值（用于测试）
pub fn jit_eval_const_expr(expr: &Expression) -> Option<Value> {
    match expr {
        Expression::IntLiteral(val) => Some(Value::Int(*val)),
        Expression::FloatLiteral(val) => Some(Value::Float(*val)),
        Expression::BinaryOp(left, op, right) => {
            let left_val = jit_eval_const_expr(left)?;
            let right_val = jit_eval_const_expr(right)?;

            match (left_val, op, right_val) {
                (Value::Int(l), BinaryOperator::Add, Value::Int(r)) => {
                    Some(Value::Int(l + r))
                },
                (Value::Int(l), BinaryOperator::Subtract, Value::Int(r)) => {
                    Some(Value::Int(l - r))
                },
                (Value::Int(l), BinaryOperator::Multiply, Value::Int(r)) => {
                    Some(Value::Int(l * r))
                },
                (Value::Int(l), BinaryOperator::Divide, Value::Int(r)) => {
                    if r != 0 {
                        Some(Value::Int(l / r))
                    } else {
                        None
                    }
                },
                (Value::Float(l), BinaryOperator::Add, Value::Float(r)) => {
                    Some(Value::Float(l + r))
                },
                (Value::Float(l), BinaryOperator::Subtract, Value::Float(r)) => {
                    Some(Value::Float(l - r))
                },
                (Value::Float(l), BinaryOperator::Multiply, Value::Float(r)) => {
                    Some(Value::Float(l * r))
                },
                (Value::Float(l), BinaryOperator::Divide, Value::Float(r)) => {
                    if r != 0.0 {
                        Some(Value::Float(l / r))
                    } else {
                        None
                    }
                },
                _ => None,
            }
        },
        _ => None,
    }
}

// 占位符JIT函数，用于兼容现有代码
pub fn jit_mod(a: i64, b: i64) -> i64 {
    if b != 0 { a % b } else { 0 }
}

pub fn jit_eq_i64(a: i64, b: i64) -> bool { a == b }
pub fn jit_ne_i64(a: i64, b: i64) -> bool { a != b }
pub fn jit_gt_i64(a: i64, b: i64) -> bool { a > b }
pub fn jit_lt_i64(a: i64, b: i64) -> bool { a < b }
pub fn jit_ge_i64(a: i64, b: i64) -> bool { a >= b }
pub fn jit_le_i64(a: i64, b: i64) -> bool { a <= b }

pub fn jit_eq_f64(a: f64, b: f64) -> bool { a == b }
pub fn jit_ne_f64(a: f64, b: f64) -> bool { a != b }
pub fn jit_gt_f64(a: f64, b: f64) -> bool { a > b }
pub fn jit_lt_f64(a: f64, b: f64) -> bool { a < b }
pub fn jit_ge_f64(a: f64, b: f64) -> bool { a >= b }
pub fn jit_le_f64(a: f64, b: f64) -> bool { a <= b }

pub fn jit_and_bool(a: bool, b: bool) -> bool { a && b }
pub fn jit_or_bool(a: bool, b: bool) -> bool { a || b }

pub fn was_jit_used() -> bool {
    unsafe { JIT_INITIALIZED }
}

pub fn jit_stats() -> String {
    if unsafe { JIT_INITIALIZED } {
        let jit = get_jit();
        let stats = jit.get_stats();
        format!("📊 JIT编译器统计:\n  🔥 表达式热点: {}\n  ⚡ 编译函数数: {}\n  🔄 表达式执行: {}\n  🔥 循环热点: {}\n  ⚡ 编译循环数: {}\n  🔄 循环执行: {}",
                stats.hotspot_count, stats.compiled_count, stats.total_executions,
                stats.loop_hotspot_count, stats.compiled_loop_count, stats.total_loop_executions)
    } else {
        "❌ JIT编译器未初始化".to_string()
    }
}

/// 显示JIT统计信息（仅在调试模式下）
pub fn print_jit_stats_if_debug() {
    if std::env::var("CODENOTHING_JIT_DEBUG").is_ok() {
        println!("\n{}", jit_stats());
    }
}

/// 显示JIT性能报告
pub fn print_jit_performance_report() {
    if unsafe { JIT_INITIALIZED } {
        let jit = get_jit();
        let stats = jit.get_stats();

        println!("\n🚀 CodeNothing JIT编译器性能报告");
        println!("=====================================");

        // 表达式统计
        println!("📊 表达式JIT统计:");
        println!("  🔥 检测到的热点数量: {}", stats.hotspot_count);
        println!("  ⚡ 成功编译的函数数: {}", stats.compiled_count);
        println!("  🔄 总执行次数: {}", stats.total_executions);

        if stats.compiled_count > 0 && stats.hotspot_count > 0 {
            let compilation_rate = (stats.compiled_count as f64 / stats.hotspot_count as f64) * 100.0;
            println!("  📈 编译成功率: {:.1}%", compilation_rate);

            if stats.total_executions > 0 {
                let avg_executions = stats.total_executions as f64 / stats.hotspot_count as f64;
                println!("  📊 平均执行次数: {:.1}", avg_executions);
            }
        }

        // 循环统计
        println!("\n🔄 循环JIT统计:");
        println!("  🔥 检测到的循环热点: {}", stats.loop_hotspot_count);
        println!("  ⚡ 成功编译的循环数: {}", stats.compiled_loop_count);
        println!("  🔄 循环总执行次数: {}", stats.total_loop_executions);

        if stats.compiled_loop_count > 0 && stats.loop_hotspot_count > 0 {
            let loop_compilation_rate = (stats.compiled_loop_count as f64 / stats.loop_hotspot_count as f64) * 100.0;
            println!("  📈 循环编译成功率: {:.1}%", loop_compilation_rate);

            if stats.total_loop_executions > 0 {
                let avg_loop_executions = stats.total_loop_executions as f64 / stats.loop_hotspot_count as f64;
                println!("  📊 循环平均执行次数: {:.1}", avg_loop_executions);
            }
        }

        // 数学表达式统计
        println!("\n🧮 数学表达式JIT统计:");
        println!("  🔥 数学表达式热点数量: {}", stats.math_expression_hotspot_count);
        println!("  ⚡ 成功编译的数学表达式数: {}", stats.compiled_math_expression_count);
        println!("  🔄 数学表达式总执行次数: {}", stats.total_math_expression_executions);
        if stats.compiled_math_expression_count > 0 && stats.math_expression_hotspot_count > 0 {
            let math_compilation_rate = (stats.compiled_math_expression_count as f64 / stats.math_expression_hotspot_count as f64) * 100.0;
            println!("  📈 数学表达式编译成功率: {:.1}%", math_compilation_rate);
            if stats.total_math_expression_executions > 0 {
                let avg_math_executions = stats.total_math_expression_executions as f64 / stats.math_expression_hotspot_count as f64;
                println!("  📊 数学表达式平均执行次数: {:.1}", avg_math_executions);
            }
        }

        // 字符串操作统计
        println!("\n📝 字符串操作JIT统计:");
        println!("  🔥 字符串操作热点数量: {}", stats.string_operation_hotspot_count);
        println!("  ⚡ 成功编译的字符串操作数: {}", stats.compiled_string_operation_count);
        println!("  🔄 字符串操作总执行次数: {}", stats.total_string_operation_executions);
        if stats.compiled_string_operation_count > 0 && stats.string_operation_hotspot_count > 0 {
            let string_compilation_rate = (stats.compiled_string_operation_count as f64 / stats.string_operation_hotspot_count as f64) * 100.0;
            println!("  📈 字符串操作编译成功率: {:.1}%", string_compilation_rate);
            if stats.total_string_operation_executions > 0 {
                let avg_string_executions = stats.total_string_operation_executions as f64 / stats.string_operation_hotspot_count as f64;
                println!("  📊 字符串操作平均执行次数: {:.1}", avg_string_executions);
            }
        }

        println!("=====================================");

        // 总体状态
        let total_compiled = stats.compiled_count + stats.compiled_loop_count + stats.compiled_math_expression_count + stats.compiled_string_operation_count;
        let total_hotspots = stats.hotspot_count + stats.loop_hotspot_count + stats.math_expression_hotspot_count + stats.string_operation_hotspot_count;

        if total_compiled > 0 {
            println!("✅ JIT编译器工作正常！");
        } else if total_hotspots > 0 {
            println!("⚠️  检测到热点但未成功编译");
        } else {
            println!("ℹ️  未检测到需要JIT编译的热点");
        }
    } else {
        println!("❌ JIT编译器未初始化");
    }
}

/// JIT编译并执行表达式
pub fn jit_compile_and_execute_expression(expr: &Expression, variables: &HashMap<String, i64>) -> Option<Value> {
    let jit = get_jit();

    // 生成表达式的唯一键
    let key = format!("expr_{:p}", expr as *const _);

    // 检查是否应该编译
    if !jit.should_compile(&key) {
        return None;
    }

    // 尝试编译表达式
    match jit.compile_expression(expr, key.clone()) {
        Ok(compiled_func) => {
            unsafe {
                if JIT_DEBUG_MODE {
                    println!("🔧 JIT: 成功编译表达式，变量数量: {}", variables.len());
                }
            }

            // 收集变量值
            let mut var_names = Vec::new();
            jit.collect_variables(expr, &mut var_names);

            let mut args = Vec::new();
            for var_name in &var_names {
                if let Some(value) = variables.get(var_name) {
                    args.push(*value);
                } else {
                    return None; // 变量未找到
                }
            }

            // 执行编译后的函数
            let result = compiled_func.call(&args);
            // 根据原始表达式的类型返回适当的Value类型
            if args.iter().all(|&arg| arg <= i32::MAX as i64 && arg >= i32::MIN as i64) &&
               result <= i32::MAX as i64 && result >= i32::MIN as i64 {
                Some(Value::Int(result as i32))  // 返回Int
            } else {
                Some(Value::Long(result))  // 返回Long
            }
        },
        Err(_) => None
    }
}

// ============================================================================
// 🔄 v0.7.7: 循环JIT编译优化 - 增强的循环热点分析系统
// ============================================================================

/// 🔄 v0.7.7: 循环执行统计信息
#[derive(Debug, Clone)]
pub struct LoopExecutionStats {
    /// 循环执行次数
    pub execution_count: usize,
    /// 总迭代次数
    pub total_iterations: usize,
    /// 平均每次执行的迭代次数
    pub average_iterations_per_execution: f64,
    /// 总执行时间
    pub total_execution_time: Duration,
    /// 平均执行时间
    pub average_execution_time: Duration,
    /// 内存使用模式
    pub memory_usage_pattern: MemoryUsagePattern,
    /// 循环体复杂度评分
    pub complexity_score: f32,
    /// 最后更新时间
    pub last_updated: Instant,
}

impl LoopExecutionStats {
    pub fn new() -> Self {
        LoopExecutionStats {
            execution_count: 0,
            total_iterations: 0,
            average_iterations_per_execution: 0.0,
            total_execution_time: Duration::from_millis(0),
            average_execution_time: Duration::from_millis(0),
            memory_usage_pattern: MemoryUsagePattern::new(),
            complexity_score: 0.0,
            last_updated: Instant::now(),
        }
    }

    /// 更新执行统计
    pub fn update_execution(&mut self, iterations: usize, execution_time: Duration) {
        self.execution_count += 1;
        self.total_iterations += iterations;
        self.total_execution_time += execution_time;

        self.average_iterations_per_execution = self.total_iterations as f64 / self.execution_count as f64;
        self.average_execution_time = self.total_execution_time / self.execution_count as u32;
        self.last_updated = Instant::now();
    }

    /// 计算JIT编译优先级
    pub fn calculate_jit_priority(&self) -> f32 {
        let frequency_score = (self.execution_count as f32).ln().max(1.0);
        let iteration_score = (self.average_iterations_per_execution as f32).ln().max(1.0);
        let time_score = self.average_execution_time.as_millis() as f32 / 1000.0;
        let complexity_bonus = self.complexity_score * 0.5;

        frequency_score * iteration_score * time_score + complexity_bonus
    }
}

/// 🔄 v0.7.7: 内存使用模式
#[derive(Debug, Clone)]
pub struct MemoryUsagePattern {
    /// 变量访问次数
    pub variable_accesses: usize,
    /// 内存分配次数
    pub memory_allocations: usize,
    /// 平均内存使用量
    pub average_memory_usage: usize,
    /// 是否有内存密集操作
    pub is_memory_intensive: bool,
}

impl MemoryUsagePattern {
    pub fn new() -> Self {
        MemoryUsagePattern {
            variable_accesses: 0,
            memory_allocations: 0,
            average_memory_usage: 0,
            is_memory_intensive: false,
        }
    }
}

/// 🔄 v0.7.7: 循环复杂度分析器
#[derive(Debug)]
pub struct LoopComplexityAnalyzer {
    /// 复杂度评分缓存
    complexity_cache: HashMap<String, f32>,
}

impl LoopComplexityAnalyzer {
    pub fn new() -> Self {
        LoopComplexityAnalyzer {
            complexity_cache: HashMap::new(),
        }
    }

    /// 分析循环体复杂度
    pub fn analyze_loop_complexity(&mut self, loop_key: &str, loop_body: &[Statement]) -> f32 {
        if let Some(&cached_score) = self.complexity_cache.get(loop_key) {
            return cached_score;
        }

        let mut complexity_score = 0.0;

        for stmt in loop_body {
            complexity_score += self.analyze_statement_complexity(stmt);
        }

        // 基于语句数量的基础复杂度
        complexity_score += loop_body.len() as f32 * 0.1;

        // 缓存结果
        self.complexity_cache.insert(loop_key.to_string(), complexity_score);
        complexity_score
    }

    /// 分析单个语句的复杂度
    fn analyze_statement_complexity(&self, stmt: &Statement) -> f32 {
        match stmt {
            Statement::VariableDeclaration(_, _, _) => 0.5,
            Statement::VariableAssignment(_, expr) => 0.3 + self.analyze_expression_complexity(expr),
            Statement::IfElse(condition, then_block, else_blocks) => {
                let mut score = 1.0 + self.analyze_expression_complexity(condition);
                for stmt in then_block {
                    score += self.analyze_statement_complexity(stmt) * 0.8;
                }
                for (_, block) in else_blocks {
                    for stmt in block {
                        score += self.analyze_statement_complexity(stmt) * 0.8;
                    }
                }
                score
            },
            Statement::WhileLoop(condition, body) => {
                let mut score = 2.0 + self.analyze_expression_complexity(condition);
                for stmt in body {
                    score += self.analyze_statement_complexity(stmt) * 1.5; // 嵌套循环权重更高
                }
                score
            },
            Statement::ForLoop(_, start, end, body) => {
                let mut score = 2.0 + self.analyze_expression_complexity(start) + self.analyze_expression_complexity(end);
                for stmt in body {
                    score += self.analyze_statement_complexity(stmt) * 1.5;
                }
                score
            },
            _ => 0.2, // 其他语句的基础复杂度
        }
    }

    /// 分析表达式复杂度
    fn analyze_expression_complexity(&self, expr: &Expression) -> f32 {
        match expr {
            Expression::IntLiteral(_) | Expression::FloatLiteral(_) | Expression::BoolLiteral(_) => 0.1,
            Expression::StringLiteral(_) => 0.2,
            Expression::Variable(_) => 0.1,
            Expression::BinaryOp(left, _, right) => {
                0.5 + self.analyze_expression_complexity(left) + self.analyze_expression_complexity(right)
            },
            Expression::FunctionCall(_, args) => {
                let mut score = 1.0;
                for arg in args {
                    score += self.analyze_expression_complexity(arg);
                }
                score
            },
            Expression::ArrayAccess(array, index) => {
                0.8 + self.analyze_expression_complexity(array) + self.analyze_expression_complexity(index)
            },
            _ => 0.3, // 其他表达式的基础复杂度
        }
    }
}

/// 🔄 v0.7.7: JIT编译阈值配置
#[derive(Debug, Clone)]
pub struct LoopJitThresholds {
    /// 基础执行次数阈值
    pub base_execution_threshold: usize,
    /// 复杂度调整因子
    pub complexity_factor: f32,
    /// 迭代次数调整因子
    pub iteration_factor: f32,
    /// 内存密集型调整因子
    pub memory_intensive_factor: f32,
}

impl Default for LoopJitThresholds {
    fn default() -> Self {
        LoopJitThresholds {
            base_execution_threshold: 50,  // 基础阈值降低，更积极地JIT编译
            complexity_factor: 0.8,        // 复杂度越高，阈值越低
            iteration_factor: 0.9,         // 迭代次数越多，阈值越低
            memory_intensive_factor: 1.2,  // 内存密集型循环阈值稍高
        }
    }
}

/// 🔄 v0.7.7: 增强的循环热点分析器
#[derive(Debug)]
pub struct LoopHotspotAnalyzer {
    /// 循环执行统计
    execution_stats: HashMap<String, LoopExecutionStats>,
    /// 循环复杂度分析器
    complexity_analyzer: LoopComplexityAnalyzer,
    /// JIT编译阈值配置
    jit_thresholds: LoopJitThresholds,
    /// 性能监控开始时间
    monitoring_start_time: Instant,
}

impl LoopHotspotAnalyzer {
    pub fn new() -> Self {
        LoopHotspotAnalyzer {
            execution_stats: HashMap::new(),
            complexity_analyzer: LoopComplexityAnalyzer::new(),
            jit_thresholds: LoopJitThresholds::default(),
            monitoring_start_time: Instant::now(),
        }
    }

    /// 记录循环执行
    pub fn record_loop_execution(&mut self, loop_key: &str, iterations: usize, execution_time: Duration, loop_body: &[Statement]) {
        let stats = self.execution_stats.entry(loop_key.to_string()).or_insert_with(LoopExecutionStats::new);

        // 更新执行统计
        stats.update_execution(iterations, execution_time);

        // 分析复杂度（如果还没有分析过）
        if stats.complexity_score == 0.0 {
            stats.complexity_score = self.complexity_analyzer.analyze_loop_complexity(loop_key, loop_body);
        }

        crate::jit_debug_println!("🔄 JIT: 记录循环执行 {} - 迭代: {}, 时间: {:?}, 复杂度: {:.2}",
                                 loop_key, iterations, execution_time, stats.complexity_score);
    }

    /// 检查是否应该JIT编译循环
    pub fn should_jit_compile_loop(&self, loop_key: &str) -> bool {
        if let Some(stats) = self.execution_stats.get(loop_key) {
            let dynamic_threshold = self.calculate_dynamic_threshold(stats);
            let priority = stats.calculate_jit_priority();

            crate::jit_debug_println!("🎯 JIT: 循环 {} 优先级: {:.2}, 动态阈值: {:.2}",
                                     loop_key, priority, dynamic_threshold);

            stats.execution_count >= dynamic_threshold
        } else {
            false
        }
    }

    /// 计算动态JIT编译阈值
    fn calculate_dynamic_threshold(&self, stats: &LoopExecutionStats) -> usize {
        let mut threshold = self.jit_thresholds.base_execution_threshold as f32;

        // 基于复杂度调整
        if stats.complexity_score > 5.0 {
            threshold *= self.jit_thresholds.complexity_factor;
        }

        // 基于迭代次数调整
        if stats.average_iterations_per_execution > 100.0 {
            threshold *= self.jit_thresholds.iteration_factor;
        }

        // 基于内存使用模式调整
        if stats.memory_usage_pattern.is_memory_intensive {
            threshold *= self.jit_thresholds.memory_intensive_factor;
        }

        threshold.max(10.0) as usize // 最小阈值为10
    }

    /// 获取循环执行统计
    pub fn get_loop_stats(&self, loop_key: &str) -> Option<&LoopExecutionStats> {
        self.execution_stats.get(loop_key)
    }

    /// 获取所有热点循环
    pub fn get_hotspot_loops(&self) -> Vec<(String, f32)> {
        let mut hotspots: Vec<(String, f32)> = self.execution_stats
            .iter()
            .map(|(key, stats)| (key.clone(), stats.calculate_jit_priority()))
            .collect();

        hotspots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        hotspots
    }

    /// 获取分析器统计信息
    pub fn get_analyzer_stats(&self) -> LoopHotspotAnalyzerStats {
        let total_loops = self.execution_stats.len();
        let total_executions: usize = self.execution_stats.values().map(|s| s.execution_count).sum();
        let total_iterations: usize = self.execution_stats.values().map(|s| s.total_iterations).sum();

        let hotspot_count = self.execution_stats.values()
            .filter(|stats| self.should_jit_compile_loop(&format!("loop_{:p}", stats as *const _)))
            .count();

        LoopHotspotAnalyzerStats {
            total_loops_monitored: total_loops,
            total_loop_executions: total_executions,
            total_loop_iterations: total_iterations,
            hotspot_loops_count: hotspot_count,
            average_complexity: if total_loops > 0 {
                self.execution_stats.values().map(|s| s.complexity_score).sum::<f32>() / total_loops as f32
            } else {
                0.0
            },
            monitoring_duration: self.monitoring_start_time.elapsed(),
        }
    }
}

/// 🔄 v0.7.7: 循环热点分析器统计信息
#[derive(Debug, Clone)]
pub struct LoopHotspotAnalyzerStats {
    pub total_loops_monitored: usize,
    pub total_loop_executions: usize,
    pub total_loop_iterations: usize,
    pub hotspot_loops_count: usize,
    pub average_complexity: f32,
    pub monitoring_duration: Duration,
}

/// 🔄 v0.7.7: 编译的循环JIT函数
#[derive(Debug, Clone)]
pub struct CompiledLoopJitFunction {
    /// 编译后的函数指针
    pub func_ptr: *const u8,
    /// 函数签名
    pub signature: LoopJitSignature,
    /// 优化策略
    pub optimization_strategies: Vec<String>,
    /// 编译时间
    pub compilation_time: Duration,
    /// 预期性能提升
    pub expected_speedup: f32,
}

/// 🔄 v0.7.7: 循环JIT函数签名
#[derive(Debug, Clone)]
pub struct LoopJitSignature {
    /// 输入参数类型
    pub input_types: Vec<JitType>,
    /// 输出类型
    pub output_type: JitType,
    /// 循环变量类型
    pub loop_variables: Vec<(String, JitType)>,
}

/// 🔄 v0.7.7: 循环优化策略
#[derive(Debug, Clone)]
pub enum LoopOptimizationStrategy {
    /// 循环展开
    LoopUnrolling { factor: usize },
    /// 向量化
    Vectorization { simd_width: usize },
    /// 强度削减
    StrengthReduction,
    /// 循环不变量提升
    LoopInvariantCodeMotion,
    /// 循环融合
    LoopFusion,
    /// 内存预取
    MemoryPrefetching,
}

// 全局函数，用于外部模块调用

/// 检查数组操作是否应该JIT编译
pub fn should_compile_array_operation(operation_key: &str) -> bool {
    // 简化实现：总是返回false，表示暂时不编译
    // 在实际实现中，这里应该检查全局JIT编译器实例
    false
}

/// 编译数组操作
pub fn compile_array_operation(
    expression: &Expression,
    key: String,
    debug_mode: bool
) -> Result<CompiledArrayOperation, String> {
    // 简化实现：创建一个占位符编译结果
    if debug_mode {
        println!("🧮 JIT: 全局编译数组操作 {}", key);
    }

    let signature = ArrayOperationSignature {
        operation_desc: key.clone(),
        element_type: ArrayElementType::Mixed,
        array_size: None,
        output_type: ArrayOutputType::Single,
        memory_pattern: ArrayMemoryPattern::Sequential,
    };

    Ok(CompiledArrayOperation {
        func_ptr: std::ptr::null(),
        signature,
        operation_type: ArrayOperationType::Access,
        optimization: ArrayOptimization::CacheOptimization,
        is_vectorized: false,
        bounds_check_eliminated: false,
    })
}