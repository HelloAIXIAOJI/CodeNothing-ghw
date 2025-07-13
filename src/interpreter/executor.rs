use crate::ast::{Statement, Expression, BinaryOperator, LogicalOperator};
use crate::interpreter::value::Value;
use crate::interpreter::evaluator::{Evaluator, evaluate_compare_operation};
use std::collections::HashMap;

pub trait Executor: Evaluator {
    fn execute_statement(&mut self, statement: Statement) -> Option<Value>;
    fn execute_function(&mut self, function_name: &str) -> Value;
    fn update_variable(&mut self, name: &str, value: Value) -> Result<(), String>;
}

// 处理变量更新逻辑
pub fn update_variable_value(
    local_env: &mut HashMap<String, Value>,
    global_env: &mut HashMap<String, Value>,
    name: &str,
    value: Value
) -> Result<(), String> {
    if local_env.contains_key(name) {
        local_env.insert(name.to_string(), value);
        Ok(())
    } else if global_env.contains_key(name) {
        global_env.insert(name.to_string(), value);
        Ok(())
    } else {
        Err(format!("未定义的变量: {}", name))
    }
}

// 处理自增操作
pub fn handle_increment(
    local_env: &mut HashMap<String, Value>,
    global_env: &mut HashMap<String, Value>,
    name: &str
) -> Result<(), String> {
    let value = if local_env.contains_key(name) {
        local_env.get(name).unwrap().clone()
    } else if global_env.contains_key(name) {
        global_env.get(name).unwrap().clone()
    } else {
        return Err(format!("未定义的变量: {}", name));
    };
    
    // 根据变量类型执行自增
    let new_value = match value {
        Value::Int(i) => Value::Int(i + 1),
        Value::Float(f) => Value::Float(f + 1.0),
        Value::Long(l) => Value::Long(l + 1),
        _ => return Err(format!("不能对类型 {:?} 执行自增操作", value)),
    };
    
    // 更新变量值
    if local_env.contains_key(name) {
        local_env.insert(name.to_string(), new_value);
    } else {
        global_env.insert(name.to_string(), new_value);
    }
    
    Ok(())
}

// 处理自减操作
pub fn handle_decrement(
    local_env: &mut HashMap<String, Value>,
    global_env: &mut HashMap<String, Value>,
    name: &str
) -> Result<(), String> {
    let value = if local_env.contains_key(name) {
        local_env.get(name).unwrap().clone()
    } else if global_env.contains_key(name) {
        global_env.get(name).unwrap().clone()
    } else {
        return Err(format!("未定义的变量: {}", name));
    };
    
    // 根据变量类型执行自减
    let new_value = match value {
        Value::Int(i) => Value::Int(i - 1),
        Value::Float(f) => Value::Float(f - 1.0),
        Value::Long(l) => Value::Long(l - 1),
        _ => return Err(format!("不能对类型 {:?} 执行自减操作", value)),
    };
    
    // 更新变量值
    if local_env.contains_key(name) {
        local_env.insert(name.to_string(), new_value);
    } else {
        global_env.insert(name.to_string(), new_value);
    }
    
    Ok(())
}

// 执行if-else语句
pub fn execute_if_else<E: Evaluator>(
    evaluator: &mut E,
    condition: &Expression,
    if_block: &[Statement],
    else_blocks: &[(Option<Expression>, Vec<Statement>)],
    executor: &mut impl Executor
) -> Option<Value> {
    // 计算条件表达式
    let condition_value = evaluator.evaluate_expression(condition);
    
    // 检查条件是否为真
    let is_true = match condition_value {
        Value::Bool(b) => b,
        _ => panic!("条件表达式必须是布尔类型"),
    };
    
    if is_true {
        // 执行 if 块
        for stmt in if_block {
            if let Some(value) = executor.execute_statement(stmt.clone()) {
                return Some(value); // 如果有返回值，则提前返回
            }
        }
    } else {
        // 尝试执行 else-if 或 else 块
        for (maybe_condition, block) in else_blocks {
            match maybe_condition {
                Some(else_if_condition) => {
                    // 这是 else-if 块，需要计算条件
                    let else_if_value = evaluator.evaluate_expression(else_if_condition);
                    let else_if_is_true = match else_if_value {
                        Value::Bool(b) => b,
                        _ => panic!("else-if 条件表达式必须是布尔类型"),
                    };
                    
                    if else_if_is_true {
                        // 条件为真，执行这个 else-if 块
                        for stmt in block {
                            if let Some(value) = executor.execute_statement(stmt.clone()) {
                                return Some(value); // 如果有返回值，则提前返回
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
                        if let Some(value) = executor.execute_statement(stmt.clone()) {
                            return Some(value); // 如果有返回值，则提前返回
                        }
                    }
                    // else 块是最后一个块，执行完后退出
                    break;
                }
            }
        }
    }
    
    None
} 