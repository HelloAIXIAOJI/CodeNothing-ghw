use crate::ast::{Statement, Type};
use crate::interpreter::executor::ExecutionResult;
use crate::interpreter::interpreter_core::Interpreter;
use crate::interpreter::statement_executor::StatementExecutor;

pub fn handle_try_catch(interpreter: &mut Interpreter, try_block: Vec<Statement>, catch_blocks: Vec<(String, Type, Vec<Statement>)>, finally_block: Option<Vec<Statement>>) -> ExecutionResult {
    // 执行 try 块
    let try_result = {
        let mut exception_caught = false;
        let mut exception_value = None;
        
        // 执行 try 块中的语句
        for stmt in try_block {
            match interpreter.execute_statement_direct(stmt) {
                ExecutionResult::None => {},
                ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                ExecutionResult::Break => return ExecutionResult::Break,
                ExecutionResult::Continue => return ExecutionResult::Continue,
                ExecutionResult::Throw(value) => {
                    exception_caught = true;
                    exception_value = Some(value);
                    break;
                },
                ExecutionResult::Error(msg) => {
                    eprintln!("执行错误: {}", msg);
                    return ExecutionResult::Error(msg);
                }
            }
        }
        
        if exception_caught {
            exception_value
        } else {
            None
        }
    };
    
    // 如果有异常被抛出，尝试匹配 catch 块
    if let Some(exception_value) = try_result {
        // 遍历 catch 块，尝试匹配异常类型
        for (exception_name, exception_type, catch_block) in catch_blocks {
            // 检查异常类型是否匹配（这里简化处理，所有异常都匹配）
            // 在实际实现中，你可能需要更复杂的类型匹配逻辑
            
            // 将异常值绑定到异常变量
            interpreter.local_env.insert(exception_name, exception_value.clone());
            
            // 执行 catch 块
            for stmt in catch_block {
                match interpreter.execute_statement_direct(stmt) {
                    ExecutionResult::None => {},
                    ExecutionResult::Return(value) => {
                        // 执行 finally 块（如果存在）
                        if let Some(ref finally_block) = finally_block {
                            for stmt in finally_block {
                                interpreter.execute_statement_direct(stmt.clone());
                            }
                        }
                        return ExecutionResult::Return(value);
                    },
                    ExecutionResult::Break => return ExecutionResult::Break,
                    ExecutionResult::Continue => return ExecutionResult::Continue,
                    ExecutionResult::Throw(value) => {
                        // 执行 finally 块（如果存在）
                        if let Some(ref finally_block) = finally_block {
                            for stmt in finally_block {
                                interpreter.execute_statement_direct(stmt.clone());
                            }
                        }
                        return ExecutionResult::Throw(value);
                    },
                    ExecutionResult::Error(msg) => {
                        // 执行 finally 块（如果存在）
                        if let Some(ref finally_block) = finally_block {
                            for stmt in finally_block {
                                interpreter.execute_statement_direct(stmt.clone());
                            }
                        }
                        return ExecutionResult::Error(msg);
                    }
                }
            }
            
            // 如果执行到这里，说明异常已经被处理
            break;
        }
    }
    
    // 执行 finally 块（如果存在）
    if let Some(finally_block) = finally_block {
        for stmt in finally_block {
            match interpreter.execute_statement_direct(stmt) {
                ExecutionResult::None => {},
                ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                ExecutionResult::Break => return ExecutionResult::Break,
                ExecutionResult::Continue => return ExecutionResult::Continue,
                ExecutionResult::Throw(value) => return ExecutionResult::Throw(value),
                ExecutionResult::Error(msg) => return ExecutionResult::Error(msg),
            }
        }
    }
    
    ExecutionResult::None
} 