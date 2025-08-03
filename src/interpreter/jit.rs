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
    /// 表达式热点检测计数器
    hotspot_counters: HashMap<String, u32>,
    /// 循环热点检测计数器
    loop_counters: HashMap<String, u32>,
    /// 编译缓存
    compiled_functions: HashMap<String, CompiledFunction>,
    /// 编译的循环缓存
    compiled_loops: HashMap<String, CompiledLoop>,
    /// 表达式热点阈值
    hotspot_threshold: u32,
    /// 循环热点阈值
    loop_threshold: u32,
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

/// 循环类型
#[derive(Debug, Clone, PartialEq)]
pub enum LoopType {
    While,
    For,
    ForEach,
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
            compiled_functions: HashMap::new(),
            compiled_loops: HashMap::new(),
            hotspot_threshold: 100, // 表达式执行100次后触发JIT编译
            loop_threshold: 100,    // 循环执行100次后触发JIT编译
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

    /// 生成循环的唯一键
    pub fn generate_loop_key(&self, loop_type: &str, location: &str) -> String {
        format!("loop_{}_{}", loop_type, location)
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
            // 暂时禁用条件语句编译，专注于基本语句
            // Statement::IfElse(condition, then_stmts, else_branches) => {
            //     self.can_compile_expression(condition) &&
            //     then_stmts.len() <= 3 && // 限制then分支语句数量
            //     else_branches.len() <= 1 && // 只支持一个else分支
            //     then_stmts.iter().all(|s| self.can_compile_simple_statement(s)) &&
            //     else_branches.iter().all(|(cond, stmts)| {
            //         cond.is_none() && // 只支持else，不支持else-if
            //         stmts.len() <= 3 &&
            //         stmts.iter().all(|s| self.can_compile_simple_statement(s))
            //     })
            // },
            // 暂不支持嵌套控制流，但支持break/continue
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

    /// 获取编译统计信息
    pub fn get_stats(&self) -> JitStats {
        JitStats {
            hotspot_count: self.hotspot_counters.len(),
            compiled_count: self.compiled_functions.len(),
            total_executions: self.hotspot_counters.values().sum(),
            loop_hotspot_count: self.loop_counters.len(),
            compiled_loop_count: self.compiled_loops.len(),
            total_loop_executions: self.loop_counters.values().sum(),
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

    /// 编译循环体
    fn compile_loop_body(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block
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
                Statement::Break | Statement::Continue => {
                    // TODO: 实现break/continue的控制流
                    // 目前跳过，将来可以通过特殊返回值或异常处理
                },
                _ => {} // 其他语句暂不支持
            }
        }

        Ok(current_vars)
    }

    /// 编译For循环体
    fn compile_for_loop_body(
        &self,
        builder: &mut FunctionBuilder,
        loop_body: &[Statement],
        variables: &[String],
        current_block: Block
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
                Statement::Break | Statement::Continue => {
                    // TODO: 实现break/continue的控制流
                    // 目前跳过，将来可以通过特殊返回值或异常处理
                },
                _ => {} // 其他语句暂不支持
            }
        }

        Ok(current_vars)
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
                }
            },
            Expression::CompareOp(left, op, right) => {
                let left_val = self.compile_expr_to_value_with_vars(builder, left, variables, current_block)?;
                let right_val = self.compile_expr_to_value_with_vars(builder, right, variables, current_block)?;

                let condition = match op {
                    crate::ast::CompareOperator::Equal => builder.ins().icmp(IntCC::Equal, left_val, right_val),
                    crate::ast::CompareOperator::NotEqual => builder.ins().icmp(IntCC::NotEqual, left_val, right_val),
                    crate::ast::CompareOperator::Less => builder.ins().icmp(IntCC::SignedLessThan, left_val, right_val),
                    crate::ast::CompareOperator::LessEqual => builder.ins().icmp(IntCC::SignedLessThanOrEqual, left_val, right_val),
                    crate::ast::CompareOperator::Greater => builder.ins().icmp(IntCC::SignedGreaterThan, left_val, right_val),
                    crate::ast::CompareOperator::GreaterEqual => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left_val, right_val),
                };

                // 将布尔值转换为i64 (0或1)
                Ok(builder.ins().uextend(types::I64, condition))
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
    pub loop_hotspot_count: usize,
    pub compiled_loop_count: usize,
    pub total_loop_executions: u32,
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

        println!("=====================================");

        // 总体状态
        let total_compiled = stats.compiled_count + stats.compiled_loop_count;
        let total_hotspots = stats.hotspot_count + stats.loop_hotspot_count;

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