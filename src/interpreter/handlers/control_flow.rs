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
    // 计算范围的起始值和结束值
    let start_value = interpreter.evaluate_expression(&range_start);
    let end_value = interpreter.evaluate_expression(&range_end);
    
    // 获取起始和结束的整数值
    let (start, end) = match (&start_value, &end_value) {
        (Value::Int(s), Value::Int(e)) => (*s, *e),
        _ => panic!("for循环的范围必须是整数类型"),
    };
    
    // 在局部环境中声明循环变量
    interpreter.local_env.insert(variable_name.clone(), Value::Int(start));
    
    // 执行循环
    for i in start..=end {
        // 更新循环变量的值
        interpreter.local_env.insert(variable_name.clone(), Value::Int(i));
        
        // 执行循环体
        for stmt in &loop_body {
            match interpreter.execute_statement_direct(stmt.clone()) {
                ExecutionResult::None => {},
                ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                ExecutionResult::Break => return ExecutionResult::None, // 跳出循环，但不向上传递break
                ExecutionResult::Continue => break, // 跳过当前迭代的剩余语句，继续下一次迭代
                ExecutionResult::Throw(value) => return ExecutionResult::Throw(value), // 异常向上传播
            }
        }
    }
    
    ExecutionResult::None
}

pub fn handle_foreach_loop(interpreter: &mut Interpreter, variable_name: String, collection_expr: Expression, loop_body: Vec<Statement>) -> ExecutionResult {
    // 计算集合表达式
    let collection = interpreter.evaluate_expression(&collection_expr);
    
    // 根据集合类型执行不同的迭代逻辑
    match collection {
        Value::Array(items) => {
            // 数组迭代
            for item in items {
                // 在局部环境中设置迭代变量
                interpreter.local_env.insert(variable_name.clone(), item);
                
                // 执行循环体
                for stmt in &loop_body {
                    match interpreter.execute_statement_direct(stmt.clone()) {
                        ExecutionResult::None => {},
                        ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                        ExecutionResult::Break => return ExecutionResult::None, // 跳出循环，但不向上传递break
                        ExecutionResult::Continue => break, // 跳过当前迭代的剩余语句，继续下一次迭代
                        ExecutionResult::Throw(value) => return ExecutionResult::Throw(value), // 异常向上传播
                    }
                }
            }
        },
        Value::Map(map) => {
            // 映射迭代（迭代键）
            for key in map.keys() {
                // 在局部环境中设置迭代变量（键）
                interpreter.local_env.insert(variable_name.clone(), Value::String(key.clone()));
                
                // 执行循环体
                for stmt in &loop_body {
                    match interpreter.execute_statement_direct(stmt.clone()) {
                        ExecutionResult::None => {},
                        ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                        ExecutionResult::Break => return ExecutionResult::None, // 跳出循环，但不向上传递break
                        ExecutionResult::Continue => break, // 跳过当前迭代的剩余语句，继续下一次迭代
                        ExecutionResult::Throw(value) => return ExecutionResult::Throw(value), // 异常向上传播
                    }
                }
            }
        },
        Value::String(s) => {
            // 字符串迭代（按字符迭代）
            for c in s.chars() {
                // 在局部环境中设置迭代变量（单个字符）
                interpreter.local_env.insert(variable_name.clone(), Value::String(c.to_string()));
                
                // 执行循环体
                for stmt in &loop_body {
                    match interpreter.execute_statement_direct(stmt.clone()) {
                        ExecutionResult::None => {},
                        ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                        ExecutionResult::Break => return ExecutionResult::None, // 跳出循环，但不向上传递break
                        ExecutionResult::Continue => break, // 跳过当前迭代的剩余语句，继续下一次迭代
                        ExecutionResult::Throw(value) => return ExecutionResult::Throw(value), // 异常向上传播
                    }
                }
            }
        },
        _ => panic!("foreach循环的集合必须是数组、映射或字符串类型"),
    }
    
    ExecutionResult::None
}

pub fn handle_while_loop(interpreter: &mut Interpreter, condition: Expression, loop_body: Vec<Statement>) -> ExecutionResult {
    // 循环执行，直到条件为假
    loop {
        // 计算条件表达式
        let condition_value = interpreter.evaluate_expression(&condition);
        
        // 检查条件是否为真
        let is_true = match condition_value {
            Value::Bool(b) => b,
            _ => panic!("while循环的条件必须是布尔类型"),
        };
        
        if !is_true {
            break; // 条件为假，退出循环
        }
        
        // 执行循环体
        for stmt in &loop_body {
            match interpreter.execute_statement_direct(stmt.clone()) {
                ExecutionResult::None => {},
                ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                ExecutionResult::Break => return ExecutionResult::None, // 跳出循环，但不向上传递break
                ExecutionResult::Continue => break, // 跳过当前迭代的剩余语句，继续下一次迭代
                ExecutionResult::Throw(value) => return ExecutionResult::Throw(value), // 异常向上传播
            }
        }
    }
    
    ExecutionResult::None
} 