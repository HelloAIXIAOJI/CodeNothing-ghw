use crate::ast::Expression;
use crate::interpreter::executor::ExecutionResult;
use crate::interpreter::interpreter_core::{Interpreter, debug_println};
use crate::interpreter::expression_evaluator::ExpressionEvaluator;
use crate::interpreter::library_loader::{load_library, call_library_function};

pub fn handle_library_function_call_statement(interpreter: &mut Interpreter, lib_name: String, func_name: String, args: Vec<Expression>) -> ExecutionResult {
    // 库函数调用语句
    debug_println(&format!("库函数调用语句: {}::{}", lib_name, func_name));
    
    // 计算参数值
    let mut arg_values = Vec::new();
    for arg in args {
        let value = interpreter.evaluate_expression(&arg);
        // 将Value转换为String
        arg_values.push(value.to_string());
    }

    // 检查库是否已加载
    if !interpreter.imported_libraries.contains_key(&lib_name) {
        // 尝试加载库
        match load_library(&lib_name) {
            Ok(functions) => {
                interpreter.imported_libraries.insert(lib_name.clone(), functions);
            },
            Err(err) => {
                panic!("无法加载库 '{}': {}", lib_name, err);
            }
        }
    }

    // 调用库函数
    match call_library_function(&lib_name, &func_name, arg_values) {
        Ok(result) => {
            // 库函数调用成功，但我们不需要返回值
            debug_println(&format!("库函数调用成功: {}::{}", lib_name, func_name));
        },
        Err(err) => {
            panic!("调用库函数 {}::{} 失败: {}", lib_name, func_name, err);
        }
    }
    
    ExecutionResult::None
} 