use crate::ast::{Statement, Expression, Type};
use crate::interpreter::value::Value;
use crate::interpreter::executor::ExecutionResult;
use crate::interpreter::interpreter_core::Interpreter;
use crate::interpreter::expression_evaluator::ExpressionEvaluator;
use crate::interpreter::statement_executor::StatementExecutor;

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
    // 优化：预计算范围值，避免重复求值
    let (start, end) = evaluate_for_loop_range(interpreter, &range_start, &range_end);

    // 优化：检查范围有效性，避免无效循环
    if start > end {
        return ExecutionResult::None; // 空范围，直接返回
    }

    // 优化：预分配循环变量，避免重复字符串操作
    let var_name_key = variable_name.clone();

    // 在局部环境中声明循环变量
    interpreter.local_env.insert(var_name_key.clone(), Value::Int(start));

    // 优化的循环执行：使用更高效的迭代方式
    execute_for_loop_optimized(interpreter, &var_name_key, start, end, &loop_body)
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
    // 优化：预检查条件类型，避免每次循环都检查
    let is_simple_condition = is_simple_boolean_condition(&condition);

    // 循环执行，直到条件为假
    loop {
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

/// 优化的循环体执行
fn execute_loop_body_optimized(interpreter: &mut Interpreter, loop_body: &[Statement]) -> Option<ExecutionResult> {
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