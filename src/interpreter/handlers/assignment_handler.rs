use crate::ast::{Expression, BinaryOperator};
use crate::interpreter::executor::ExecutionResult;
use crate::interpreter::interpreter_core::Interpreter;
use crate::interpreter::expression_evaluator::ExpressionEvaluator;

pub fn handle_compound_assignment(interpreter: &mut Interpreter, name: String, op: BinaryOperator, expr: Expression) -> ExecutionResult {
    // 先获取变量当前值
    let current_value = if interpreter.local_env.contains_key(&name) {
        interpreter.local_env.get(&name).unwrap().clone()
    } else if interpreter.global_env.contains_key(&name) {
        interpreter.global_env.get(&name).unwrap().clone()
    } else {
        panic!("未定义的变量: {}", name);
    };
    
    // 计算右侧表达式的值
    let right_value = interpreter.evaluate_expression(&expr);
    
    // 执行复合赋值操作
    let new_value = interpreter.perform_binary_operation(&current_value, &op, &right_value);
    
    // 更新变量值
    if interpreter.local_env.contains_key(&name) {
        interpreter.local_env.insert(name, new_value);
    } else {
        interpreter.global_env.insert(name, new_value);
    }
    
    ExecutionResult::None
} 