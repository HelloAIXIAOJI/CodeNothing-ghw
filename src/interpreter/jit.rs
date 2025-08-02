// 🚀 CodeNothing JIT编译器 v0.6.4
// 基于Cranelift的即时编译系统

use crate::ast::{Expression, BinaryOperator, Statement};
use crate::interpreter::value::Value;
use std::collections::HashMap;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Module, Linkage};

/// JIT编译器状态
pub struct JitCompiler {
    /// 热点检测计数器
    hotspot_counters: HashMap<String, u32>,
    /// 编译缓存
    compiled_functions: HashMap<String, CompiledFunction>,
    /// 热点阈值
    hotspot_threshold: u32,
}

/// 编译后的函数
#[derive(Clone)]
pub struct CompiledFunction {
    /// 函数指针
    func_ptr: *const u8,
    /// 函数签名信息
    signature: FunctionSignature,
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
            compiled_functions: HashMap::new(),
            hotspot_threshold: 100, // 执行100次后触发JIT编译
        }
    }

    /// 检查是否应该JIT编译
    pub fn should_compile(&mut self, key: &str) -> bool {
        let counter = self.hotspot_counters.entry(key.to_string()).or_insert(0);
        *counter += 1;
        *counter >= self.hotspot_threshold
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
                body.iter().all(|s| self.can_compile_statement(s))
            },
            Statement::ForLoop(_, start, end, body) => {
                self.can_compile_expression(start) &&
                self.can_compile_expression(end) &&
                body.iter().all(|s| self.can_compile_statement(s))
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
            BinaryOperator::Divide
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

    /// 编译循环（占位符实现）
    pub fn compile_loop(&mut self, loop_body: &[Statement], key: String, debug_mode: bool) -> Result<(), String> {
        // TODO: 实现循环的JIT编译
        if debug_mode {
            println!("🔧 JIT: 编译循环 {}", key);
        }
        Ok(())
    }

    /// 获取编译统计信息
    pub fn get_stats(&self) -> JitStats {
        JitStats {
            hotspot_count: self.hotspot_counters.len(),
            compiled_count: self.compiled_functions.len(),
            total_executions: self.hotspot_counters.values().sum(),
        }
    }

    /// 收集表达式中的变量
    fn collect_variables(&self, expr: &Expression, variables: &mut Vec<String>) {
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
}

/// JIT编译统计信息
#[derive(Debug)]
pub struct JitStats {
    pub hotspot_count: usize,
    pub compiled_count: usize,
    pub total_executions: u32,
}

/// 全局JIT编译器实例
static mut GLOBAL_JIT: Option<JitCompiler> = None;
static mut JIT_INITIALIZED: bool = false;
static mut JIT_DEBUG_MODE: bool = false;

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
        format!("📊 JIT编译器统计:\n  🔥 热点数量: {}\n  ⚡ 编译函数数: {}\n  🔄 总执行次数: {}",
                stats.hotspot_count, stats.compiled_count, stats.total_executions)
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
        println!("🔥 检测到的热点数量: {}", stats.hotspot_count);
        println!("⚡ 成功编译的函数数: {}", stats.compiled_count);
        println!("🔄 总执行次数: {}", stats.total_executions);

        if stats.compiled_count > 0 {
            let compilation_rate = (stats.compiled_count as f64 / stats.hotspot_count as f64) * 100.0;
            println!("📈 编译成功率: {:.1}%", compilation_rate);

            if stats.total_executions > 0 {
                let avg_executions = stats.total_executions as f64 / stats.hotspot_count as f64;
                println!("📊 平均执行次数: {:.1}", avg_executions);
            }
        }

        println!("=====================================");

        if stats.compiled_count > 0 {
            println!("✅ JIT编译器工作正常！");
        } else if stats.hotspot_count > 0 {
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