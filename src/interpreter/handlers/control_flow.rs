use crate::ast::{Statement, Expression, Type};
use crate::interpreter::value::Value;
use crate::interpreter::executor::ExecutionResult;
use crate::interpreter::interpreter_core::Interpreter;
use crate::interpreter::expression_evaluator::ExpressionEvaluator;
use crate::interpreter::statement_executor::StatementExecutor;
use crate::interpreter::jit;
use crate::interpreter::memory_manager::{batch_memory_operations};
use crate::loop_memory::{LoopVariableType, enter_loop, exit_loop};

pub fn handle_if_else(interpreter: &mut Interpreter, condition: Expression, if_block: Vec<Statement>, else_blocks: Vec<(Option<Expression>, Vec<Statement>)>) -> ExecutionResult {
    // 修复借用问题：不直接传递self，而是分别计算条件和执行语句块
    let condition_value = interpreter.evaluate_expression(&condition);
    
    // 检查条件是否为真
    let is_true = match condition_value {
        Value::Bool(b) => b,
        _ => panic!("条件表达式必须是布尔类型"),
    };
    
    if is_true {
        // 执行 if 块
        for stmt in if_block {
            match interpreter.execute_statement_direct(stmt.clone()) {
                ExecutionResult::None => {},
                result => return result, // 如果有特殊结果（返回值、break、continue），则传递给上层
            }
        }
    } else {
        // 尝试执行 else-if 或 else 块
        for (maybe_condition, block) in else_blocks {
            match maybe_condition {
                Some(else_if_condition) => {
                    // 这是 else-if 块，需要计算条件
                    let else_if_value = interpreter.evaluate_expression(&else_if_condition);
                    let else_if_is_true = match else_if_value {
                        Value::Bool(b) => b,
                        _ => panic!("else-if 条件表达式必须是布尔类型"),
                    };
                    
                    if else_if_is_true {
                        // 条件为真，执行这个 else-if 块
                        for stmt in block {
                            match interpreter.execute_statement_direct(stmt.clone()) {
                                ExecutionResult::None => {},
                                result => return result, // 如果有特殊结果，则传递给上层
                            }
                        }
                        // 执行完一个 else-if 块后，不再执行后续块
                        break;
                    }
                    // 条件为假，继续检查下一个块
                },
                None => {
                    // 这是 else 块，直接执行
                    for stmt in block {
                        match interpreter.execute_statement_direct(stmt.clone()) {
                            ExecutionResult::None => {},
                            result => return result, // 如果有特殊结果，则传递给上层
                        }
                    }
                    // else 块是最后一个块，执行完后退出
                    break;
                }
            }
        }
    }
    
    ExecutionResult::None
}

pub fn handle_for_loop(interpreter: &mut Interpreter, variable_name: String, range_start: Expression, range_end: Expression, loop_body: Vec<Statement>) -> ExecutionResult {
    // 生成循环的唯一键用于热点检测
    let loop_key = format!("for_loop_{}_{:p}_{:p}", variable_name, &range_start as *const _, &range_end as *const _);

    // 优化：预计算范围值，避免重复求值
    let (start, end) = evaluate_for_loop_range(interpreter, &range_start, &range_end);

    // 🔄 v0.7.6: 循环内存管理 - 预分析循环变量
    let expected_variables = analyze_loop_variables(&variable_name, &loop_body);

    // 进入循环内存管理
    if let Err(e) = enter_loop(&expected_variables) {
        crate::memory_debug_println!("⚠️ 循环内存管理启动失败: {}", e);
    }

    // 优化：检查范围有效性，避免无效循环
    if start > end {
        return ExecutionResult::None; // 空范围，直接返回
    }

    // JIT热点检测和编译
    let jit_compiler = jit::get_jit();
    if jit_compiler.should_compile_loop(&loop_key) {
        // 检查循环是否适合JIT编译
        let for_stmt = Statement::ForLoop(variable_name.clone(), range_start.clone(), range_end.clone(), loop_body.clone());
        if jit_compiler.can_compile_loop(&for_stmt) {
            // 尝试JIT编译For循环
            let debug_mode = unsafe { jit::JIT_DEBUG_MODE };
            match jit_compiler.compile_for_loop(&variable_name, &range_start, &range_end, &loop_body, loop_key.clone(), debug_mode) {
                Ok(compiled_loop) => {
                    if debug_mode {
                        println!("🚀 JIT: 成功编译For循环");
                    }

                    // 收集变量值
                    let mut var_values = Vec::new();
                    let mut var_names = Vec::new();
                    var_names.push(variable_name.clone()); // 循环变量
                    jit_compiler.collect_variables(&range_start, &mut var_names);
                    jit_compiler.collect_variables(&range_end, &mut var_names);
                    for stmt in &loop_body {
                        jit_compiler.collect_statement_variables(stmt, &mut var_names);
                    }

                    // 获取其他变量的当前值（跳过循环变量，它由start_expr确定）
                    for var_name in &var_names[1..] {
                        if let Some(value) = interpreter.local_env.get(var_name).or_else(|| interpreter.global_env.get(var_name)) {
                            match value {
                                Value::Int(i) => var_values.push(*i as i64),
                                Value::Long(l) => var_values.push(*l),
                                _ => var_values.push(0), // 不支持的类型默认为0
                            }
                        } else {
                            var_values.push(0); // 未找到的变量默认为0
                        }
                    }

                    // 执行编译后的For循环
                    let result_values = compiled_loop.call(&var_values);

                    // 更新所有变量的最终值
                    if result_values.len() == var_names.len() {
                        for (i, var_name) in var_names.iter().enumerate() {
                            let final_value = result_values[i];
                            if final_value <= i32::MAX as i64 && final_value >= i32::MIN as i64 {
                                interpreter.local_env.insert(var_name.clone(), Value::Int(final_value as i32));
                            } else {
                                interpreter.local_env.insert(var_name.clone(), Value::Long(final_value));
                            }
                        }
                    }

                    return ExecutionResult::None;
                },
                Err(e) => {
                    if debug_mode {
                        println!("⚠️ JIT: For循环编译失败: {}", e);
                    }
                    // 编译失败，回退到解释执行
                }
            }
        }
    }

    // 优化：预分配循环变量，避免重复字符串操作
    let var_name_key = variable_name.clone();

    // 在局部环境中声明循环变量
    interpreter.local_env.insert(var_name_key.clone(), Value::Int(start));

    // 优化的循环执行：使用更高效的迭代方式
    let result = execute_for_loop_optimized(interpreter, &var_name_key, start, end, &loop_body);

    // 🔄 v0.7.6: 退出循环内存管理
    if let Err(e) = exit_loop() {
        crate::memory_debug_println!("⚠️ 循环内存管理退出失败: {}", e);
    }

    result
}

/// 优化的范围计算
fn evaluate_for_loop_range(interpreter: &mut Interpreter, range_start: &Expression, range_end: &Expression) -> (i32, i32) {
    // 快速路径：如果是常量，直接返回
    if let (Expression::IntLiteral(s), Expression::IntLiteral(e)) = (range_start, range_end) {
        return (*s, *e);
    }

    // 计算范围的起始值和结束值
    let start_value = interpreter.evaluate_expression(range_start);
    let end_value = interpreter.evaluate_expression(range_end);

    // 获取起始和结束的整数值
    match (&start_value, &end_value) {
        (Value::Int(s), Value::Int(e)) => (*s, *e),
        _ => panic!("for循环的范围必须是整数类型"),
    }
}

/// 优化的for循环执行
fn execute_for_loop_optimized(interpreter: &mut Interpreter, var_name: &str, start: i32, end: i32, loop_body: &[Statement]) -> ExecutionResult {
    // 优化：使用手动循环而不是Rust的for..in，减少迭代器开销
    let mut i = start;
    while i <= end {
        // 优化：直接更新变量值，避免重复的HashMap查找
        if let Some(var_value) = interpreter.local_env.get_mut(var_name) {
            *var_value = Value::Int(i);
        } else {
            interpreter.local_env.insert(var_name.to_string(), Value::Int(i));
        }

        // 优化的循环体执行
        if let Some(result) = execute_loop_body_optimized(interpreter, loop_body) {
            return result;
        }

        i += 1;
    }

    ExecutionResult::None
}

pub fn handle_foreach_loop(interpreter: &mut Interpreter, variable_name: String, collection_expr: Expression, loop_body: Vec<Statement>) -> ExecutionResult {
    // 计算集合表达式
    let collection = interpreter.evaluate_expression(&collection_expr);

    // 优化：预分配变量名，避免重复克隆
    let var_name_key = variable_name;

    // 根据集合类型执行不同的迭代逻辑
    match collection {
        Value::Array(items) => {
            execute_array_foreach_optimized(interpreter, &var_name_key, items, &loop_body)
        },
        Value::Map(map) => {
            execute_map_foreach_optimized(interpreter, &var_name_key, map, &loop_body)
        },
        Value::String(s) => {
            execute_string_foreach_optimized(interpreter, &var_name_key, s, &loop_body)
        },
        _ => panic!("foreach循环的集合必须是数组、映射或字符串类型"),
    }
}

/// 优化的数组foreach循环
fn execute_array_foreach_optimized(interpreter: &mut Interpreter, var_name: &str, items: Vec<Value>, loop_body: &[Statement]) -> ExecutionResult {
    for item in items {
        // 优化：直接更新变量值
        update_loop_variable_optimized(interpreter, var_name, item);

        // 优化的循环体执行
        if let Some(result) = execute_loop_body_optimized(interpreter, loop_body) {
            return result;
        }
    }
    ExecutionResult::None
}

/// 优化的映射foreach循环
fn execute_map_foreach_optimized(interpreter: &mut Interpreter, var_name: &str, map: std::collections::HashMap<String, Value>, loop_body: &[Statement]) -> ExecutionResult {
    for key in map.keys() {
        // 优化：直接更新变量值
        update_loop_variable_optimized(interpreter, var_name, Value::String(key.clone()));

        // 优化的循环体执行
        if let Some(result) = execute_loop_body_optimized(interpreter, loop_body) {
            return result;
        }
    }
    ExecutionResult::None
}

/// 优化的字符串foreach循环
fn execute_string_foreach_optimized(interpreter: &mut Interpreter, var_name: &str, s: String, loop_body: &[Statement]) -> ExecutionResult {
    for c in s.chars() {
        // 优化：直接更新变量值
        update_loop_variable_optimized(interpreter, var_name, Value::String(c.to_string()));

        // 优化的循环体执行
        if let Some(result) = execute_loop_body_optimized(interpreter, loop_body) {
            return result;
        }
    }
    ExecutionResult::None
}

/// 优化的循环变量更新
fn update_loop_variable_optimized(interpreter: &mut Interpreter, var_name: &str, value: Value) {
    // 优化：直接更新现有变量，避免重复的HashMap操作
    if let Some(existing_value) = interpreter.local_env.get_mut(var_name) {
        *existing_value = value;
    } else {
        interpreter.local_env.insert(var_name.to_string(), value);
    }

    // 暂时禁用缓存更新以避免副作用
    // if !interpreter.variable_cache.contains_key(var_name) {
    //     interpreter.variable_cache.insert(var_name.to_string(), crate::interpreter::interpreter_core::VariableLocation::Local);
    // }
}



pub fn handle_while_loop(interpreter: &mut Interpreter, condition: Expression, loop_body: Vec<Statement>) -> ExecutionResult {
    // 生成循环的唯一键用于热点检测
    let loop_key = format!("while_loop_{:p}", &condition as *const _);

    // 🔄 v0.7.6: 循环内存管理 - 预分析循环变量
    let expected_variables = analyze_while_loop_variables(&loop_body);

    // 进入循环内存管理
    if let Err(e) = enter_loop(&expected_variables) {
        crate::memory_debug_println!("⚠️ While循环内存管理启动失败: {}", e);
    }

    // 优化：预检查条件类型，避免每次循环都检查
    let is_simple_condition = is_simple_boolean_condition(&condition);

    // 循环执行，直到条件为假
    loop {
        // JIT热点检测和编译
        let jit_compiler = jit::get_jit();
        if jit_compiler.should_compile_loop(&loop_key) {
            // 检查循环是否适合JIT编译
            let while_stmt = Statement::WhileLoop(condition.clone(), loop_body.clone());
            if jit_compiler.can_compile_loop(&while_stmt) {
                // 尝试JIT编译循环
                let debug_mode = unsafe { jit::JIT_DEBUG_MODE };
                match jit_compiler.compile_while_loop(&condition, &loop_body, loop_key.clone(), debug_mode) {
                    Ok(compiled_loop) => {
                        if debug_mode {
                            println!("🚀 JIT: 成功编译While循环");
                        }

                        // 收集变量值
                        let mut var_values = Vec::new();
                        let mut var_names = Vec::new();
                        jit_compiler.collect_variables(&condition, &mut var_names);
                        for stmt in &loop_body {
                            jit_compiler.collect_statement_variables(stmt, &mut var_names);
                        }

                        // 获取变量的当前值
                        for var_name in &var_names {
                            if let Some(value) = interpreter.local_env.get(var_name).or_else(|| interpreter.global_env.get(var_name)) {
                                match value {
                                    Value::Int(i) => var_values.push(*i as i64),
                                    Value::Long(l) => var_values.push(*l),
                                    _ => var_values.push(0), // 不支持的类型默认为0
                                }
                            } else {
                                var_values.push(0); // 未找到的变量默认为0
                            }
                        }

                        // 执行编译后的循环
                        let result_values = compiled_loop.call(&var_values);

                        // 更新变量值
                        if !result_values.is_empty() && !var_names.is_empty() {
                            let result_value = result_values[0];
                            if result_value <= i32::MAX as i64 && result_value >= i32::MIN as i64 {
                                interpreter.local_env.insert(var_names[0].clone(), Value::Int(result_value as i32));
                            } else {
                                interpreter.local_env.insert(var_names[0].clone(), Value::Long(result_value));
                            }
                        }

                        return ExecutionResult::None;
                    },
                    Err(e) => {
                        if debug_mode {
                            println!("⚠️ JIT: While循环编译失败: {}", e);
                        }
                        // 编译失败，回退到解释执行
                    }
                }
            }
        }
        // 优化的条件求值
        let is_true = if is_simple_condition {
            evaluate_simple_condition(interpreter, &condition)
        } else {
            // 计算条件表达式
            let condition_value = interpreter.evaluate_expression(&condition);

            // 检查条件是否为真
            match condition_value {
                Value::Bool(b) => b,
                _ => panic!("while循环的条件必须是布尔类型"),
            }
        };

        if !is_true {
            break; // 条件为假，退出循环
        }

        // 优化的循环体执行：减少克隆和匹配开销
        if let Some(result) = execute_loop_body_optimized(interpreter, &loop_body) {
            return result;
        }
    }

    // 🔄 v0.7.6: 退出循环内存管理
    if let Err(e) = exit_loop() {
        crate::memory_debug_println!("⚠️ While循环内存管理退出失败: {}", e);
    }

    ExecutionResult::None
}

/// 检查是否为简单的布尔条件（变量或简单比较）
fn is_simple_boolean_condition(condition: &Expression) -> bool {
    match condition {
        Expression::Variable(_) => true,
        Expression::BoolLiteral(_) => true,
        Expression::CompareOp(_, op, _) => {
            matches!(op,
                crate::ast::CompareOperator::Equal |
                crate::ast::CompareOperator::NotEqual |
                crate::ast::CompareOperator::Less |
                crate::ast::CompareOperator::LessEqual |
                crate::ast::CompareOperator::Greater |
                crate::ast::CompareOperator::GreaterEqual
            )
        },
        _ => false,
    }
}

/// 快速求值简单条件
fn evaluate_simple_condition(interpreter: &mut Interpreter, condition: &Expression) -> bool {
    match condition {
        Expression::Variable(name) => {
            // 直接查找变量，避免完整的表达式求值
            let value = interpreter.get_variable_fast(name);
            match value {
                Value::Bool(b) => b,
                Value::None => panic!("未找到变量: {}", name),
                _ => panic!("while循环的条件必须是布尔类型"),
            }
        },
        Expression::BoolLiteral(b) => *b,
        _ => {
            // 回退到完整求值
            let condition_value = interpreter.evaluate_expression(condition);
            match condition_value {
                Value::Bool(b) => b,
                _ => panic!("while循环的条件必须是布尔类型"),
            }
        }
    }
}

/// 🚀 v0.6.10 智能批量内存操作优化
fn execute_loop_body_optimized(interpreter: &mut Interpreter, loop_body: &[Statement]) -> Option<ExecutionResult> {
    // 🧠 智能判断是否需要批量内存操作优化
    let optimization_strategy = determine_memory_optimization_strategy(loop_body, interpreter);

    match optimization_strategy {
        MemoryOptimizationStrategy::None => {
            // 简单循环，使用标准路径
            execute_loop_body_standard(interpreter, loop_body)
        },
        MemoryOptimizationStrategy::Lightweight => {
            // 中等复杂度，使用轻量级优化
            execute_loop_body_lightweight_optimized(interpreter, loop_body)
        },
        MemoryOptimizationStrategy::FullBatch => {
            // 复杂循环，使用完整批量操作
            execute_loop_body_with_smart_batch_memory(interpreter, loop_body)
        }
    }
}

/// 标准的循环体执行（无内存操作优化）
fn execute_loop_body_standard(interpreter: &mut Interpreter, loop_body: &[Statement]) -> Option<ExecutionResult> {
    for stmt in loop_body {
        // 避免克隆：直接引用语句
        match execute_statement_no_clone(interpreter, stmt) {
            ExecutionResult::None => {},
            ExecutionResult::Return(value) => return Some(ExecutionResult::Return(value)),
            ExecutionResult::Break => return Some(ExecutionResult::None), // 跳出循环，但不向上传递break
            ExecutionResult::Continue => break, // 跳过当前迭代的剩余语句，继续下一次迭代
            ExecutionResult::Throw(value) => return Some(ExecutionResult::Throw(value)), // 异常向上传播
        }
    }
    None
}

/// 🚀 v0.6.10 带批量内存操作的循环体执行
fn execute_loop_body_with_batch_memory(
    interpreter: &mut Interpreter,
    loop_body: &[Statement],
    _memory_operations: Vec<MemoryOperation>
) -> Option<ExecutionResult> {
    // 使用批量内存操作执行循环体
    let result = batch_memory_operations(|_memory_manager| {
        // 在单次锁获取内执行所有语句
        for stmt in loop_body {
            match execute_statement_no_clone(interpreter, stmt) {
                ExecutionResult::None => {},
                ExecutionResult::Return(value) => return Some(ExecutionResult::Return(value)),
                ExecutionResult::Break => return Some(ExecutionResult::None),
                ExecutionResult::Continue => break,
                ExecutionResult::Throw(value) => return Some(ExecutionResult::Throw(value)),
            }
        }
        None
    });

    result
}

/// 🚀 v0.6.10 内存操作类型
#[derive(Debug, Clone)]
enum MemoryOperation {
    Allocate(String),      // 变量分配
    Read(String),          // 变量读取
    Write(String),         // 变量写入
    Deallocate(String),    // 变量释放
}

/// 🚀 v0.6.10 收集循环体中的内存操作
fn collect_memory_operations(loop_body: &[Statement]) -> Vec<MemoryOperation> {
    let mut operations = Vec::new();

    for stmt in loop_body {
        match stmt {
            Statement::VariableDeclaration(name, _, _) => {
                operations.push(MemoryOperation::Allocate(name.clone()));
            },
            Statement::VariableAssignment(name, _) => {
                operations.push(MemoryOperation::Write(name.clone()));
            },
            Statement::FunctionCallStatement(expr) => {
                collect_expression_memory_operations(expr, &mut operations);
            },
            _ => {
                // 其他语句类型暂不优化
            }
        }
    }

    operations
}

/// 收集表达式中的内存操作
fn collect_expression_memory_operations(expr: &Expression, operations: &mut Vec<MemoryOperation>) {
    match expr {
        Expression::Variable(name) => {
            operations.push(MemoryOperation::Read(name.clone()));
        },
        Expression::BinaryOp(left, _, right) => {
            collect_expression_memory_operations(left, operations);
            collect_expression_memory_operations(right, operations);
        },
        Expression::PreIncrement(name) | Expression::PreDecrement(name) => {
            operations.push(MemoryOperation::Read(name.clone()));
            operations.push(MemoryOperation::Write(name.clone()));
        },
        Expression::FunctionCall(_, args) => {
            for arg in args {
                collect_expression_memory_operations(arg, operations);
            }
        },
        _ => {
            // 其他表达式类型暂不分析
        }
    }
}

/// 执行语句但不克隆（优化版本）
fn execute_statement_no_clone(interpreter: &mut Interpreter, statement: &Statement) -> ExecutionResult {
    // 为了安全起见，只对最简单的语句使用快速路径
    // 复杂的语句（涉及类型检查、作用域等）回退到原有实现
    match statement {
        Statement::Break => ExecutionResult::Break,
        Statement::Continue => ExecutionResult::Continue,
        // 对于其他语句，回退到原有实现以确保正确性
        _ => interpreter.execute_statement_direct(statement.clone()),
    }
}

// 🚀 v0.6.10 智能内存优化策略
#[derive(Debug, Clone, PartialEq)]
enum MemoryOptimizationStrategy {
    None,           // 无优化 - 简单循环
    Lightweight,    // 轻量级优化 - 中等复杂度
    FullBatch,      // 完整批量操作 - 复杂循环
}

/// 🧠 智能判断内存优化策略
fn determine_memory_optimization_strategy(loop_body: &[Statement], interpreter: &Interpreter) -> MemoryOptimizationStrategy {
    // 分析循环体复杂度
    let complexity_score = analyze_loop_complexity(loop_body);

    // 估算循环迭代次数（基于变量状态）
    let estimated_iterations = estimate_loop_iterations(interpreter);

    // 计算内存操作密度
    let memory_operations = collect_memory_operations(loop_body);
    let memory_density = memory_operations.len();

    // 🎯 智能决策逻辑
    if complexity_score <= 3 && estimated_iterations <= 10 && memory_density <= 2 {
        // 简单循环：直接执行，避免优化开销
        MemoryOptimizationStrategy::None
    } else if complexity_score <= 10 && estimated_iterations <= 100 && memory_density <= 10 {
        // 中等复杂度：轻量级优化
        MemoryOptimizationStrategy::Lightweight
    } else {
        // 复杂循环：完整批量操作
        MemoryOptimizationStrategy::FullBatch
    }
}

/// 📊 分析循环体复杂度
fn analyze_loop_complexity(loop_body: &[Statement]) -> usize {
    let mut complexity = 0;

    for stmt in loop_body {
        complexity += match stmt {
            Statement::VariableDeclaration(_, _, _) => 1,
            Statement::VariableAssignment(_, _) => 1,
            Statement::FunctionCallStatement(_) => 1,
            Statement::IfElse(_, _, _) => 3,  // 条件分支增加复杂度
            Statement::WhileLoop(_, _) => 5,  // 嵌套循环大幅增加复杂度
            Statement::ForLoop(_, _, _, _) => 5,
            _ => 1,
        };
    }

    complexity
}

/// 🔢 估算循环迭代次数
fn estimate_loop_iterations(interpreter: &Interpreter) -> usize {
    // 基于当前变量状态的简单启发式估算
    // 这里使用保守估计，避免过度优化小循环

    // 检查是否有明显的循环计数器模式
    // 例如：i <= 100, i < n 等

    // 暂时返回保守估计
    50  // 默认估计50次迭代
}

/// 🚀 轻量级优化执行
fn execute_loop_body_lightweight_optimized(interpreter: &mut Interpreter, loop_body: &[Statement]) -> Option<ExecutionResult> {
    // 轻量级优化：只对明显的内存操作进行简单批量处理
    // 避免复杂的分析开销

    // 检查是否有连续的变量声明
    let consecutive_declarations = count_consecutive_declarations(loop_body);

    if consecutive_declarations >= 3 {
        // 有多个连续声明，使用轻量级批量分配
        execute_with_lightweight_batch_allocation(interpreter, loop_body)
    } else {
        // 使用标准路径
        execute_loop_body_standard(interpreter, loop_body)
    }
}

/// 📝 计算连续变量声明数量
fn count_consecutive_declarations(loop_body: &[Statement]) -> usize {
    let mut count = 0;
    let mut max_consecutive = 0;

    for stmt in loop_body {
        match stmt {
            Statement::VariableDeclaration(_, _, _) => {
                count += 1;
                max_consecutive = max_consecutive.max(count);
            },
            _ => {
                count = 0;
            }
        }
    }

    max_consecutive
}

/// 🔄 v0.7.6: 分析循环变量，为循环内存管理做准备
fn analyze_loop_variables(_loop_var: &str, loop_body: &[Statement]) -> Vec<(&'static str, LoopVariableType, usize)> {
    let mut variables = Vec::new();

    // 添加循环计数器变量
    variables.push(("loop_counter", LoopVariableType::Counter, std::mem::size_of::<i32>()));

    // 分析循环体中的变量
    for stmt in loop_body {
        match stmt {
            Statement::VariableDeclaration(_name, var_type, _) => {
                let size = match var_type {
                    Type::Int => std::mem::size_of::<i32>(),
                    Type::Long => std::mem::size_of::<i64>(),
                    Type::Float => std::mem::size_of::<f64>(),
                    Type::Bool => std::mem::size_of::<bool>(),
                    Type::String => 64, // 预估字符串大小
                    _ => 32, // 默认大小
                };

                // 根据变量名推断类型（简化处理）
                let loop_var_type = LoopVariableType::Temporary;

                // 使用静态字符串引用
                variables.push(("temp_var", loop_var_type, size));
            },
            Statement::VariableAssignment(_name, _) => {
                // 简化处理：添加一个通用的累加器变量
                variables.push(("accumulator", LoopVariableType::Accumulator, std::mem::size_of::<i32>()));
            },
            _ => {
                // 其他语句类型暂不分析
            }
        }
    }

    // 去重并限制数量
    variables.truncate(10); // 最多预分配10个变量
    variables
}

/// 🔄 v0.7.6: 分析while循环变量
fn analyze_while_loop_variables(loop_body: &[Statement]) -> Vec<(&'static str, LoopVariableType, usize)> {
    let mut variables = Vec::new();

    // 分析循环体中的变量
    for stmt in loop_body {
        match stmt {
            Statement::VariableDeclaration(_name, var_type, _) => {
                let size = match var_type {
                    Type::Int => std::mem::size_of::<i32>(),
                    Type::Long => std::mem::size_of::<i64>(),
                    Type::Float => std::mem::size_of::<f64>(),
                    Type::Bool => std::mem::size_of::<bool>(),
                    Type::String => 64, // 预估字符串大小
                    _ => 32, // 默认大小
                };

                // 简化类型推断
                let loop_var_type = LoopVariableType::Temporary;

                variables.push(("while_var", loop_var_type, size));
            },
            Statement::VariableAssignment(_name, _) => {
                // 简化处理：添加通用变量
                variables.push(("while_counter", LoopVariableType::Counter, std::mem::size_of::<i32>()));
            },
            _ => {
                // 其他语句类型暂不分析
            }
        }
    }

    // 去重并限制数量
    variables.truncate(8); // while循环预分配较少变量
    variables
}

/// 🔧 轻量级批量分配执行
fn execute_with_lightweight_batch_allocation(interpreter: &mut Interpreter, loop_body: &[Statement]) -> Option<ExecutionResult> {
    // 简单的批量分配优化：预分配变量空间
    // 避免复杂的分析和批量操作

    // 预分析需要分配的变量
    let mut variables_to_allocate = Vec::new();

    for stmt in loop_body {
        if let Statement::VariableDeclaration(var_name, _, init_expr) = stmt {
            // VariableDeclaration总是有初始化表达式
            if is_simple_expression(init_expr) {
                variables_to_allocate.push((var_name.clone(), init_expr.clone()));
            }
        }
    }

    // 如果有足够的简单变量声明，使用批量分配
    if variables_to_allocate.len() >= 2 {
        execute_with_batch_variable_allocation(interpreter, loop_body, variables_to_allocate)
    } else {
        execute_loop_body_standard(interpreter, loop_body)
    }
}

/// 🔍 判断是否为简单表达式
fn is_simple_expression(expr: &Expression) -> bool {
    match expr {
        Expression::IntLiteral(_) => true,
        Expression::FloatLiteral(_) => true,
        Expression::BoolLiteral(_) => true,
        Expression::StringLiteral(_) => true,
        Expression::Variable(_) => true,
        Expression::BinaryOp(left, _, right) => {
            is_simple_expression(left) && is_simple_expression(right)
        },
        _ => false,
    }
}

/// 📦 批量变量分配执行
fn execute_with_batch_variable_allocation(
    interpreter: &mut Interpreter,
    loop_body: &[Statement],
    _variables: Vec<(String, Expression)>
) -> Option<ExecutionResult> {
    // 实现简单的批量变量分配
    // 这里先使用标准路径，后续可以优化
    execute_loop_body_standard(interpreter, loop_body)
}

/// 🚀 智能批量内存操作执行
fn execute_loop_body_with_smart_batch_memory(interpreter: &mut Interpreter, loop_body: &[Statement]) -> Option<ExecutionResult> {
    // 完整的智能批量内存操作
    // 只在确实需要时才使用

    batch_memory_operations(|_| {
        // 缓存内存操作分析结果
        let memory_operations = collect_memory_operations(loop_body);

        if memory_operations.len() >= 5 {
            // 足够多的内存操作，值得批量处理
            execute_loop_body_with_batch_memory(interpreter, loop_body, memory_operations)
        } else {
            // 内存操作不多，使用轻量级优化
            execute_loop_body_lightweight_optimized(interpreter, loop_body)
        }
    })
}