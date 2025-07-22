use crate::ast::{Statement, Expression, Type, NamespaceType, Function};
use super::value::Value;
use super::executor::{Executor, ExecutionResult, update_variable_value, handle_increment, handle_decrement};
use super::library_loader::{load_library, call_library_function, convert_values_to_string_args};
use super::interpreter_core::{Interpreter, debug_println};
use super::expression_evaluator::ExpressionEvaluator;
use super::handlers;

pub trait StatementExecutor {
    fn execute_statement(&mut self, statement: Statement) -> ExecutionResult;
    fn execute_function(&mut self, function: &Function) -> Value;
    fn update_variable(&mut self, name: &str, value: Value) -> Result<(), String>;
    fn call_function(&mut self, function_name: &str, args: Vec<Value>) -> Value;
}

impl<'a> StatementExecutor for Interpreter<'a> {
    fn execute_statement(&mut self, statement: Statement) -> ExecutionResult {
        match statement {
            Statement::Return(expr) => {
                // 返回语句，计算表达式值并返回
                let value = self.evaluate_expression(&expr);
                ExecutionResult::Return(value)
            },
            Statement::VariableDeclaration(name, _type, expr) => {
                let value = self.evaluate_expression(&expr);
                self.local_env.insert(name, value);
                ExecutionResult::None
            },
            Statement::ConstantDeclaration(name, typ, expr) => {
                // 计算常量值
                let value = self.evaluate_expression(&expr);
                
                // 检查类型是否匹配
                let type_matches = match (&typ, &value) {
                    (Type::Int, Value::Int(_)) => true,
                    (Type::Float, Value::Float(_)) => true,
                    (Type::Bool, Value::Bool(_)) => true,
                    (Type::String, Value::String(_)) => true,
                    (Type::Long, Value::Long(_)) => true,
                    _ => false
                };
                
                if !type_matches {
                    panic!("常量 '{}' 的类型不匹配", name);
                }
                
                // 检查是否已存在同名常量
                if self.constants.contains_key(&name) {
                    panic!("常量 '{}' 已定义", name);
                }
                
                // 存储常量值
                self.constants.insert(name, value);
                
                ExecutionResult::None
            },
            Statement::VariableAssignment(name, expr) => {
                // 检查是否尝试修改常量
                if self.constants.contains_key(&name) {
                    panic!("无法修改常量 '{}'", name);
                }
                
                let value = self.evaluate_expression(&expr);
                // 先检查局部变量，再检查全局变量
                if self.local_env.contains_key(&name) {
                    self.local_env.insert(name, value);
                } else if self.global_env.contains_key(&name) {
                    self.global_env.insert(name, value);
                } else {
                    panic!("未定义的变量: {}", name);
                }
                ExecutionResult::None
            },
            Statement::Increment(name) => {
                // 使用辅助函数处理后置自增操作
                if let Err(err) = handle_increment(&mut self.local_env, &mut self.global_env, &name) {
                    panic!("{}", err);
                }
                ExecutionResult::None
            },
            Statement::Decrement(name) => {
                // 使用辅助函数处理后置自减操作
                if let Err(err) = handle_decrement(&mut self.local_env, &mut self.global_env, &name) {
                    panic!("{}", err);
                }
                ExecutionResult::None
            },
            Statement::PreIncrement(name) => {
                // 使用辅助函数处理前置自增操作
                if let Err(err) = handle_increment(&mut self.local_env, &mut self.global_env, &name) {
                    panic!("{}", err);
                }
                ExecutionResult::None
            },
            Statement::PreDecrement(name) => {
                // 使用辅助函数处理前置自减操作
                if let Err(err) = handle_decrement(&mut self.local_env, &mut self.global_env, &name) {
                    panic!("{}", err);
                }
                ExecutionResult::None
            },
            Statement::CompoundAssignment(name, op, expr) => {
                handlers::assignment_handler::handle_compound_assignment(self, name, op, expr)
            },
            Statement::ImportNamespace(ns_type, path) => {
                match ns_type {
                    NamespaceType::Code => {
                        // 作用域级别导入：将命名空间下所有函数名映射到完整路径
                        let ns_path = path.join("::");
                        let mut import_map = self.namespace_import_stack.last_mut().unwrap();
                        for (full_path, _) in &self.namespaced_functions {
                            if full_path.starts_with(&ns_path) {
                                // 获取函数名
                                let parts: Vec<&str> = full_path.split("::").collect();
                                if let Some(func_name) = parts.last() {
                                    import_map.entry(func_name.to_string()).or_insert_with(Vec::new).push(full_path.clone());
                                }
                            }
                        }
                        ExecutionResult::None
                    },
                    _ => handlers::namespace_handler::handle_import_namespace(self, ns_type, path)
                }
            },
            Statement::FileImport(file_path) => {
                // 导入文件
                debug_println(&format!("导入文件: {}", file_path));
                
                // 文件导入已在main.rs中预处理，这里不需要额外处理
                // 只需记录日志并返回None
                debug_println("文件导入已在预处理阶段处理");
                ExecutionResult::None
            },
            Statement::FunctionCallStatement(expr) => {
                // 函数调用语句，计算表达式值但不返回
                self.evaluate_expression(&expr);
                ExecutionResult::None
            },
            Statement::NamespacedFunctionCallStatement(path, args) => {
                handlers::namespace_handler::handle_namespaced_function_call_statement(self, path, args)
            },
            Statement::LibraryFunctionCallStatement(lib_name, func_name, args) => {
                handlers::library_handler::handle_library_function_call_statement(self, lib_name, func_name, args)
            },
            Statement::IfElse(condition, if_block, else_blocks) => {
                handlers::control_flow::handle_if_else(self, condition, if_block, else_blocks)
            },
            Statement::ForLoop(variable_name, range_start, range_end, loop_body) => {
                handlers::control_flow::handle_for_loop(self, variable_name, range_start, range_end, loop_body)
            },
            Statement::ForEachLoop(variable_name, collection_expr, loop_body) => {
                handlers::control_flow::handle_foreach_loop(self, variable_name, collection_expr, loop_body)
            },
            Statement::WhileLoop(condition, loop_body) => {
                handlers::control_flow::handle_while_loop(self, condition, loop_body)
            },
            Statement::Break => {
                // 返回Break结果，由循环处理
                ExecutionResult::Break
            },
            Statement::Continue => {
                // 返回Continue结果，由循环处理
                ExecutionResult::Continue
            },
            Statement::TryCatch(try_block, catch_blocks, finally_block) => {
                handlers::exception_handler::handle_try_catch(self, try_block, catch_blocks, finally_block)
            },
            Statement::Throw(exception_expr) => {
                // 计算异常表达式并抛出
                let exception_value = self.evaluate_expression(&exception_expr);
                ExecutionResult::Throw(exception_value)
            },
            // OOP相关语句的临时实现
            Statement::ClassDeclaration(_) => {
                ExecutionResult::Continue // 临时跳过，后续实现
            },
            Statement::FieldAssignment(_, _, _) => {
                ExecutionResult::Continue // 临时跳过，后续实现
            },
        }
    }
    
    fn execute_function(&mut self, function: &Function) -> Value {
        // 进入新作用域，push一层导入表
        self.namespace_import_stack.push(self.namespace_import_stack.last().cloned().unwrap_or_default());
        // 执行函数体
        for statement in &function.body {
            match self.execute_statement_direct(statement.clone()) {
                ExecutionResult::Return(value) => {
                    self.namespace_import_stack.pop();
                    return value
                },
                ExecutionResult::None => {},
                ExecutionResult::Break => {
                    self.namespace_import_stack.pop();
                    panic!("break语句只能在循环内部使用")
                },
                ExecutionResult::Continue => {
                    self.namespace_import_stack.pop();
                    panic!("continue语句只能在循环内部使用")
                },
                ExecutionResult::Throw(value) => {
                    self.namespace_import_stack.pop();
                    panic!("未捕获的异常: {:?}", value);
                }
            }
        }
        // 如果函数没有明确的返回语句，则返回空值
        self.namespace_import_stack.pop();
        Value::None
    }
    
    fn update_variable(&mut self, name: &str, value: Value) -> Result<(), String> {
        update_variable_value(&mut self.local_env, &mut self.global_env, name, value)
    }
    
    fn call_function(&mut self, function_name: &str, args: Vec<Value>) -> Value {
        // 优先查找当前作用域导入的命名空间
        if let Some(import_map) = self.namespace_import_stack.last() {
            if let Some(paths) = import_map.get(function_name) {
                if paths.len() == 1 {
                    let full_path = &paths[0];
                    if let Some(function) = self.namespaced_functions.get(full_path) {
                        return self.call_function_impl(function, args);
                    }
                } else if paths.len() > 1 {
                    panic!("函数名 '{}' 有多个匹配: {:?}", function_name, paths);
                }
            }
        }
        // 先检查是否是导入的命名空间函数
        if let Some(paths) = self.imported_namespaces.get(function_name) {
            if paths.len() == 1 {
                // 只有一个匹配的函数，直接调用
                let full_path = &paths[0];
                if let Some(function) = self.namespaced_functions.get(full_path) {
                    return self.call_function_impl(function, args);
                }
            }
        }
        
        // 如果不是导入的函数，再检查全局函数
        if let Some(function) = self.functions.get(function_name) {
            self.call_function_impl(function, args)
        } else {
            panic!("未定义的函数: {}", function_name);
        }
    }
}

impl<'a> Interpreter<'a> {
} 