pub mod value;
pub mod evaluator;
pub mod executor;
pub mod library_loader;

use crate::ast::{Program, Expression, Statement, BinaryOperator, Type, Namespace, CompareOperator, LogicalOperator, Function};
use std::collections::HashMap;
use value::Value;
use evaluator::{Evaluator, perform_binary_operation, evaluate_compare_operation};
use executor::{Executor, ExecutionResult, update_variable_value, handle_increment, handle_decrement, execute_if_else};
use library_loader::{load_library, call_library_function};
use std::sync::Arc;
use std::env;

// 添加调试模式检查函数
fn is_debug_mode() -> bool {
    env::args().any(|arg| arg == "--cn-debug")
}

// 添加条件打印函数
pub fn debug_println(msg: &str) {
    if is_debug_mode() {
        println!("{}", msg);
    }
}

pub fn interpret(program: &Program) -> Value {
    let mut interpreter = Interpreter::new(program);
    
    // 处理顶层的库导入
    for lib_name in &program.library_imports {
        debug_println(&format!("导入顶层动态库: {}", lib_name));
        
        // 尝试加载库
        match load_library(lib_name) {
            Ok(functions) => {
                // 库加载成功，将其添加到已导入库列表
                interpreter.imported_libraries.insert(lib_name.to_string(), functions);
                debug_println(&format!("顶层库 '{}' 加载成功", lib_name));
                
                // 将库中的所有函数添加到全局函数列表
                if let Some(lib_functions) = interpreter.imported_libraries.get(lib_name) {
                    for (func_name, _) in lib_functions.iter() {
                        debug_println(&format!("注册顶层库函数: {}", func_name));
                        
                        // 直接将库函数注册为全局函数，这样可以直接调用
                        interpreter.library_functions.insert(func_name.to_string(), (lib_name.to_string(), func_name.to_string()));
                    }
                }
            },
            Err(err) => {
                panic!("无法加载顶层库 '{}': {}", lib_name, err);
            }
        }
    }
    
    interpreter.run()
}

struct Interpreter<'a> {
    program: &'a Program,
    functions: HashMap<String, &'a crate::ast::Function>,
    // 命名空间函数映射，键是完整路径，如 "math::add"
    namespaced_functions: HashMap<String, &'a crate::ast::Function>,
    // 导入的命名空间，键是函数名，值是完整路径
    imported_namespaces: HashMap<String, Vec<String>>,
    // 导入的库，键是库名
    imported_libraries: HashMap<String, Arc<HashMap<String, library_loader::LibraryFunction>>>,
    // 库函数映射，键是函数名，值是(库名, 函数名)
    library_functions: HashMap<String, (String, String)>,
    // 全局变量环境
    global_env: HashMap<String, Value>,
    // 局部变量环境（函数内）
    local_env: HashMap<String, Value>,
}

impl<'a> Interpreter<'a> {
    fn new(program: &'a Program) -> Self {
        let mut functions = HashMap::new();
        let mut namespaced_functions = HashMap::new();
        
        // 注册全局函数
        for function in &program.functions {
            functions.insert(function.name.clone(), function);
        }
        
        // 注册命名空间函数
        for namespace in &program.namespaces {
            Self::register_namespace_functions(namespace, &mut namespaced_functions, "");
        }
        
        Interpreter {
            program,
            functions,
            namespaced_functions,
            imported_namespaces: HashMap::new(),
            imported_libraries: HashMap::new(),
            library_functions: HashMap::new(),
            global_env: HashMap::new(),
            local_env: HashMap::new(),
        }
    }
    
    // 递归注册命名空间中的所有函数
    fn register_namespace_functions(
        namespace: &'a Namespace, 
        map: &mut HashMap<String, &'a crate::ast::Function>,
        prefix: &str
    ) {
        let current_prefix = if prefix.is_empty() {
            namespace.name.clone()
        } else {
            format!("{}::{}", prefix, namespace.name)
        };
        
        // 注册当前命名空间中的函数
        for function in &namespace.functions {
            let full_path = format!("{}::{}", current_prefix, function.name);
            map.insert(full_path, function);
        }
        
        // 递归注册子命名空间中的函数
        for sub_namespace in &namespace.namespaces {
            Self::register_namespace_functions(sub_namespace, map, &current_prefix);
        }
    }
    
    fn run(&mut self) -> Value {
        // 查找 main 函数并执行
        if let Some(main_fn) = self.functions.get("main") {
            self.execute_function(main_fn)
        } else {
            panic!("没有找到 main 函数");
        }
    }
    
    // 辅助函数：调用函数并处理参数
    fn call_function_impl(&mut self, function: &'a crate::ast::Function, arg_values: Vec<Value>) -> Value {
        // 检查参数数量是否匹配
        if arg_values.len() != function.parameters.len() {
            panic!("函数 '{}' 需要 {} 个参数，但提供了 {} 个", 
                function.name, function.parameters.len(), arg_values.len());
        }
        
        // 保存当前的局部环境
        let old_local_env = self.local_env.clone();
        
        // 清空局部环境，为新函数调用准备
        self.local_env.clear();
        
        // 绑定参数值到参数名
        for (i, arg_value) in arg_values.into_iter().enumerate() {
            let param_name = &function.parameters[i].name;
            self.local_env.insert(param_name.clone(), arg_value);
        }
        
        // 执行函数体
        let result = self.execute_function(function);
        
        // 恢复之前的局部环境
        self.local_env = old_local_env;
        
        result
    }
}

impl<'a> Evaluator for Interpreter<'a> {
    fn evaluate_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::IntLiteral(value) => Value::Int(*value),
            Expression::FloatLiteral(value) => Value::Float(*value),
            Expression::BoolLiteral(value) => Value::Bool(*value),
            Expression::StringLiteral(value) => Value::String(value.clone()),
            Expression::LongLiteral(value) => Value::Long(*value),
            Expression::ArrayLiteral(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate_expression(elem));
                }
                Value::Array(values)
            },
            Expression::MapLiteral(entries) => {
                let mut map = HashMap::new();
                for (key_expr, value_expr) in entries {
                    let key = match self.evaluate_expression(key_expr) {
                        Value::String(s) => s,
                        _ => panic!("映射键必须是字符串类型"),
                    };
                    let value = self.evaluate_expression(value_expr);
                    map.insert(key, value);
                }
                Value::Map(map)
            },
            Expression::FunctionCall(name, args) => {
                // 先计算所有参数值
                let mut arg_values = Vec::new();
                for arg_expr in args {
                    arg_values.push(self.evaluate_expression(arg_expr));
                }
                
                // 检查是否是库函数
                if let Some((lib_name, func_name)) = self.library_functions.get(name) {
                    debug_println(&format!("调用库函数: {}::{}", lib_name, func_name));
                    
                    // 将Value转换为String
                    let string_args: Vec<String> = arg_values.iter().map(|v| v.to_string()).collect();
                    
                    // 调用库函数
                    match call_library_function(lib_name, func_name, string_args) {
                        Ok(result) => {
                            // 尝试将结果转换为适当的值类型
                            if let Ok(int_val) = result.parse::<i32>() {
                                return Value::Int(int_val);
                            } else if let Ok(float_val) = result.parse::<f64>() {
                                return Value::Float(float_val);
                            } else if result == "true" {
                                return Value::Bool(true);
                            } else if result == "false" {
                                return Value::Bool(false);
                            } else {
                                return Value::String(result);
                            }
                        },
                        Err(err) => {
                            panic!("调用库函数失败: {}", err);
                        }
                    }
                }
                
                // 检查是否是库函数调用（以库名_函数名的形式）
                if name.contains('_') {
                    let parts: Vec<&str> = name.split('_').collect();
                    if parts.len() >= 2 {
                        let lib_name = parts[0];
                        let func_name = &parts[1..].join("_");
                        
                        debug_println(&format!("检测到可能的库函数调用: {}_{}", lib_name, func_name));
                        
                        // 检查库是否已加载
                        if self.imported_libraries.contains_key(lib_name) {
                            debug_println(&format!("库已加载，尝试调用函数: {}", func_name));
                            
                            // 将Value转换为String
                            let string_args: Vec<String> = arg_values.iter().map(|v| v.to_string()).collect();
                            
                            // 调用库函数
                            match call_library_function(lib_name, func_name, string_args) {
                                Ok(result) => {
                                    // 尝试将结果转换为适当的值类型
                                    if let Ok(int_val) = result.parse::<i32>() {
                                        return Value::Int(int_val);
                                    } else if let Ok(float_val) = result.parse::<f64>() {
                                        return Value::Float(float_val);
                                    } else if result == "true" {
                                        return Value::Bool(true);
                                    } else if result == "false" {
                                        return Value::Bool(false);
                                    } else {
                                        return Value::String(result);
                                    }
                                },
                                Err(_) => {
                                    // 如果库函数调用失败，尝试作为普通函数调用处理
                                }
                            }
                        }
                    }
                }
                
                // 如果不是库函数调用或库函数调用失败，则作为普通函数调用处理
                debug_println(&format!("调用函数: {}", name));
                
                // 先检查是否是导入的命名空间函数
                if let Some(paths) = self.imported_namespaces.get(name) {
                    debug_println(&format!("找到导入的函数: {} -> {:?}", name, paths));
                    if paths.len() == 1 {
                        // 只有一个匹配的函数，直接调用
                        let full_path = &paths[0];
                        if let Some(function) = self.namespaced_functions.get(full_path) {
                            return self.call_function_impl(function, arg_values);
                        } else {
                            panic!("未找到函数: {}", full_path);
                        }
                    } else {
                        // 有多个匹配的函数，需要解决歧义
                        panic!("函数名 '{}' 有多个匹配: {:?}", name, paths);
                    }
                }
                
                // 如果不是导入的函数，再检查全局函数
                if let Some(function) = self.functions.get(name) {
                    debug_println(&format!("找到全局函数: {}", name));
                    // 执行全局函数
                    self.call_function_impl(function, arg_values)
                } else {
                    panic!("未定义的函数: {}", name);
                }
            },
            Expression::GlobalFunctionCall(name, args) => {
                // 先计算所有参数值
                let mut arg_values = Vec::new();
                for arg_expr in args {
                    arg_values.push(self.evaluate_expression(arg_expr));
                }
                
                debug_println(&format!("调用全局函数: {}", name));
                
                // 只在全局函数表中查找
                if let Some(function) = self.functions.get(name) {
                    self.call_function_impl(function, arg_values)
                } else {
                    panic!("未定义的全局函数: {}", name);
                }
            },
            Expression::NamespacedFunctionCall(path, args) => {
                // 构建完整的函数路径
                let full_path = path.join("::");
                
                // 先计算所有参数值
                let mut arg_values = Vec::new();
                for arg_expr in args {
                    arg_values.push(self.evaluate_expression(arg_expr));
                }
                
                debug_println(&format!("调用命名空间函数: {}", full_path));
                
                // 查找命名空间函数
                if let Some(function) = self.namespaced_functions.get(&full_path) {
                    self.call_function_impl(function, arg_values)
                } else {
                    // 检查是否是导入命名空间的嵌套命名空间函数
                    let mut found = false;
                    for (key, value) in &self.imported_namespaces {
                        if key.starts_with("__NAMESPACE__") {
                            let imported_namespace = &key[13..]; // 跳过"__NAMESPACE__"前缀
                            let potential_path = format!("{}::{}", imported_namespace, full_path);
                            
                            debug_println(&format!("尝试查找导入的嵌套命名空间函数: {}", potential_path));
                            
                            if let Some(function) = self.namespaced_functions.get(&potential_path) {
                                found = true;
                                return self.call_function_impl(function, arg_values);
                            }
                        }
                    }
                    
                    if !found {
                        panic!("未定义的命名空间函数: {}", full_path);
                    }
                    
                    // 这里不会执行到，只是为了编译通过
                    unreachable!();
                }
            },
            Expression::Variable(name) => {
                // 先检查局部变量，再检查全局变量
                if let Some(value) = self.local_env.get(name) {
                    value.clone()
                } else if let Some(value) = self.global_env.get(name) {
                    value.clone()
                } else {
                    panic!("未定义的变量: {}", name);
                }
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                
                self.perform_binary_operation(&left_val, op, &right_val)
            },
            Expression::CompareOp(left, op, right) => {
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                
                evaluate_compare_operation(&left_val, op, &right_val)
            },
            Expression::LogicalOp(left, op, right) => {
                match op {
                    LogicalOperator::And => {
                        // 短路求值：如果左操作数为假，直接返回假
                        let left_val = self.evaluate_expression(left);
                        let left_bool = match left_val {
                            Value::Bool(b) => b,
                            _ => panic!("逻辑操作符的操作数必须是布尔类型"),
                        };
                        
                        if !left_bool {
                            return Value::Bool(false);
                        }
                        
                        // 左操作数为真，计算右操作数
                        let right_val = self.evaluate_expression(right);
                        match right_val {
                            Value::Bool(b) => Value::Bool(b),
                            _ => panic!("逻辑操作符的操作数必须是布尔类型"),
                        }
                    },
                    LogicalOperator::Or => {
                        // 短路求值：如果左操作数为真，直接返回真
                        let left_val = self.evaluate_expression(left);
                        let left_bool = match left_val {
                            Value::Bool(b) => b,
                            _ => panic!("逻辑操作符的操作数必须是布尔类型"),
                        };
                        
                        if left_bool {
                            return Value::Bool(true);
                        }
                        
                        // 左操作数为假，计算右操作数
                        let right_val = self.evaluate_expression(right);
                        match right_val {
                            Value::Bool(b) => Value::Bool(b),
                            _ => panic!("逻辑操作符的操作数必须是布尔类型"),
                        }
                    },
                    LogicalOperator::Not => {
                        // 逻辑非操作
                        let val = self.evaluate_expression(left);
                        match val {
                            Value::Bool(b) => Value::Bool(!b),
                            _ => panic!("逻辑操作符的操作数必须是布尔类型"),
                        }
                    }
                }
            },
            Expression::PreIncrement(name) => {
                // 前置自增：先增加变量值，再返回新值
                
                // 获取变量当前值
                let value = if self.local_env.contains_key(name) {
                    self.local_env.get(name).unwrap().clone()
                } else if self.global_env.contains_key(name) {
                    self.global_env.get(name).unwrap().clone()
                } else {
                    panic!("未定义的变量: {}", name);
                };
                
                // 根据变量类型执行自增
                let new_value = match value {
                    Value::Int(i) => Value::Int(i + 1),
                    Value::Float(f) => Value::Float(f + 1.0),
                    Value::Long(l) => Value::Long(l + 1),
                    _ => panic!("不能对类型 {:?} 执行自增操作", value),
                };
                
                // 更新变量值
                if self.local_env.contains_key(name) {
                    self.local_env.insert(name.clone(), new_value.clone());
                } else {
                    self.global_env.insert(name.clone(), new_value.clone());
                }
                
                // 返回新值
                new_value
            },
            Expression::PreDecrement(name) => {
                // 前置自减：先减少变量值，再返回新值
                
                // 获取变量当前值
                let value = if self.local_env.contains_key(name) {
                    self.local_env.get(name).unwrap().clone()
                } else if self.global_env.contains_key(name) {
                    self.global_env.get(name).unwrap().clone()
                } else {
                    panic!("未定义的变量: {}", name);
                };
                
                // 根据变量类型执行自减
                let new_value = match value {
                    Value::Int(i) => Value::Int(i - 1),
                    Value::Float(f) => Value::Float(f - 1.0),
                    Value::Long(l) => Value::Long(l - 1),
                    _ => panic!("不能对类型 {:?} 执行自减操作", value),
                };
                
                // 更新变量值
                if self.local_env.contains_key(name) {
                    self.local_env.insert(name.clone(), new_value.clone());
                } else {
                    self.global_env.insert(name.clone(), new_value.clone());
                }
                
                // 返回新值
                new_value
            },
            Expression::PostIncrement(name) => {
                // 后置自增：先返回原值，再增加变量值
                
                // 获取变量当前值
                let value = if self.local_env.contains_key(name) {
                    self.local_env.get(name).unwrap().clone()
                } else if self.global_env.contains_key(name) {
                    self.global_env.get(name).unwrap().clone()
                } else {
                    panic!("未定义的变量: {}", name);
                };
                
                // 根据变量类型执行自增
                let new_value = match &value {
                    Value::Int(i) => Value::Int(i + 1),
                    Value::Float(f) => Value::Float(f + 1.0),
                    Value::Long(l) => Value::Long(l + 1),
                    _ => panic!("不能对类型 {:?} 执行自增操作", value),
                };
                
                // 更新变量值
                if self.local_env.contains_key(name) {
                    self.local_env.insert(name.clone(), new_value);
                } else {
                    self.global_env.insert(name.clone(), new_value);
                }
                
                // 返回原值
                value
            },
            Expression::PostDecrement(name) => {
                // 后置自减：先返回原值，再减少变量值
                
                // 获取变量当前值
                let value = if self.local_env.contains_key(name) {
                    self.local_env.get(name).unwrap().clone()
                } else if self.global_env.contains_key(name) {
                    self.global_env.get(name).unwrap().clone()
                } else {
                    panic!("未定义的变量: {}", name);
                };
                
                // 根据变量类型执行自减
                let new_value = match &value {
                    Value::Int(i) => Value::Int(i - 1),
                    Value::Float(f) => Value::Float(f - 1.0),
                    Value::Long(l) => Value::Long(l - 1),
                    _ => panic!("不能对类型 {:?} 执行自减操作", value),
                };
                
                // 更新变量值
                if self.local_env.contains_key(name) {
                    self.local_env.insert(name.clone(), new_value);
                } else {
                    self.global_env.insert(name.clone(), new_value);
                }
                
                // 返回原值
                value
            },
            Expression::LibraryFunctionCall(lib_name, func_name, args) => {
                // 先计算所有参数值
                let mut arg_values = Vec::new();
                for arg_expr in args {
                    let value = self.evaluate_expression(arg_expr);
                    // 将Value转换为String
                    arg_values.push(value.to_string());
                }
                
                debug_println(&format!("调用库函数: {}::{}", lib_name, func_name));
                
                // 检查库是否已加载
                if !self.imported_libraries.contains_key(lib_name) {
                    // 尝试加载库
                    match load_library(lib_name) {
                        Ok(functions) => {
                            self.imported_libraries.insert(lib_name.clone(), functions);
                        },
                        Err(err) => {
                            panic!("无法加载库 '{}': {}", lib_name, err);
                        }
                    }
                }
                
                // 调用库函数
                match call_library_function(lib_name, func_name, arg_values) {
                    Ok(result) => {
                        // 尝试将结果转换为适当的值类型
                        if let Ok(int_val) = result.parse::<i32>() {
                            Value::Int(int_val)
                        } else if let Ok(float_val) = result.parse::<f64>() {
                            Value::Float(float_val)
                        } else if result == "true" {
                            Value::Bool(true)
                        } else if result == "false" {
                            Value::Bool(false)
                        } else {
                            Value::String(result)
                        }
                    },
                    Err(err) => {
                        panic!("调用库函数失败: {}", err);
                    }
                }
            }
        }
    }
    
    fn perform_binary_operation(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Value {
        perform_binary_operation(left, op, right)
    }
    
    fn get_variable(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.local_env.get(name) {
            Some(value.clone())
        } else if let Some(value) = self.global_env.get(name) {
            Some(value.clone())
        } else {
            None
        }
    }
    
    fn call_function(&mut self, function_name: &str, args: Vec<Value>) -> Value {
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

impl<'a> Executor for Interpreter<'a> {
    fn execute_statement(&mut self, statement: Statement) -> ExecutionResult {
        match statement {
            Statement::Return(expr) => {
                // 返回语句，计算表达式值并返回
                ExecutionResult::Return(self.evaluate_expression(&expr))
            },
            Statement::VariableDeclaration(name, _type, expr) => {
                let value = self.evaluate_expression(&expr);
                self.local_env.insert(name, value);
                ExecutionResult::None
            },
            Statement::VariableAssignment(name, expr) => {
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
                // 复合赋值操作 (+=, -=, *=, /=, %=)
                let right_value = self.evaluate_expression(&expr);
                
                // 获取变量当前值
                let left_value = if self.local_env.contains_key(&name) {
                    self.local_env.get(&name).unwrap().clone()
                } else if self.global_env.contains_key(&name) {
                    self.global_env.get(&name).unwrap().clone()
                } else {
                    panic!("未定义的变量: {}", name);
                };
                
                // 执行对应的二元运算
                let new_value = self.perform_binary_operation(&left_value, &op, &right_value);
                
                // 更新变量值
                if self.local_env.contains_key(&name) {
                    self.local_env.insert(name, new_value);
                } else {
                    self.global_env.insert(name, new_value);
                }
                ExecutionResult::None
            },
            Statement::UsingNamespace(path) => {
                // 导入命名空间，将命名空间中的函数添加到导入表中
                let namespace_path = path.join("::");
                debug_println(&format!("导入命名空间: {}", namespace_path));
                
                // 记录导入的命名空间前缀，用于后续查找嵌套命名空间
                let imported_namespace = namespace_path.clone();
                
                // 查找命名空间中的所有函数
                let mut found_functions = false;
                for (full_path, _) in &self.namespaced_functions {
                    if full_path.starts_with(&namespace_path) && full_path.len() > namespace_path.len() {
                        // 确保是该命名空间下的函数，而不是子命名空间
                        let remaining = &full_path[namespace_path.len() + 2..]; // +2 是为了跳过"::"
                        if !remaining.contains("::") {
                            // 这是命名空间直接包含的函数
                            let func_name = remaining.to_string();
                            debug_println(&format!("导入函数: {} -> {}", func_name, full_path));
                            
                            // 将函数名和完整路径添加到导入表
                            self.imported_namespaces.entry(func_name)
                                .or_insert_with(Vec::new)
                                .push(full_path.clone());
                            
                            found_functions = true;
                        }
                    }
                }
                
                if !found_functions {
                    panic!("未找到命名空间: {}", namespace_path);
                }
                
                // 记录导入的命名空间，用于后续查找嵌套命名空间
                self.imported_namespaces.insert("__NAMESPACE__".to_string() + &imported_namespace, vec![imported_namespace]);
                ExecutionResult::None
            },
            Statement::LibraryImport(lib_name) => {
                // 导入动态库
                debug_println(&format!("导入动态库: {}", lib_name));
                
                // 尝试加载库
                match load_library(&lib_name) {
                    Ok(functions) => {
                        // 库加载成功，将其添加到已导入库列表
                        self.imported_libraries.insert(lib_name.clone(), functions);
                        debug_println(&format!("库 '{}' 加载成功", lib_name));
                        
                        // 将库中的所有函数添加到全局函数列表
                        if let Some(lib_functions) = self.imported_libraries.get(&lib_name) {
                            for (func_name, _) in lib_functions.iter() {
                                debug_println(&format!("注册库函数: {}", func_name));
                                
                                // 直接将库函数注册为全局函数，这样可以直接调用
                                self.library_functions.insert(func_name.to_string(), (lib_name.clone(), func_name.to_string()));
                            }
                        }
                    },
                    Err(err) => {
                        panic!("无法加载库 '{}': {}", lib_name, err);
                    }
                }
                
                ExecutionResult::None
            },
            Statement::IfElse(condition, if_block, else_blocks) => {
                // 修复借用问题：不直接传递self，而是分别计算条件和执行语句块
                let condition_value = self.evaluate_expression(&condition);
                
                // 检查条件是否为真
                let is_true = match condition_value {
                    Value::Bool(b) => b,
                    _ => panic!("条件表达式必须是布尔类型"),
                };
                
                if is_true {
                    // 执行 if 块
                    for stmt in if_block {
                        match self.execute_statement(stmt.clone()) {
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
                                let else_if_value = self.evaluate_expression(&else_if_condition);
                                let else_if_is_true = match else_if_value {
                                    Value::Bool(b) => b,
                                    _ => panic!("else-if 条件表达式必须是布尔类型"),
                                };
                                
                                if else_if_is_true {
                                    // 条件为真，执行这个 else-if 块
                                    for stmt in block {
                                        match self.execute_statement(stmt.clone()) {
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
                                    match self.execute_statement(stmt.clone()) {
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
            },
            Statement::ForLoop(variable_name, range_start, range_end, loop_body) => {
                // 计算范围的起始值和结束值
                let start_value = self.evaluate_expression(&range_start);
                let end_value = self.evaluate_expression(&range_end);
                
                // 获取起始和结束的整数值
                let (start, end) = match (&start_value, &end_value) {
                    (Value::Int(s), Value::Int(e)) => (*s, *e),
                    _ => panic!("for循环的范围必须是整数类型"),
                };
                
                // 在局部环境中声明循环变量
                self.local_env.insert(variable_name.clone(), Value::Int(start));
                
                // 执行循环
                for i in start..=end {
                    // 更新循环变量的值
                    self.local_env.insert(variable_name.clone(), Value::Int(i));
                    
                    // 执行循环体
                    for stmt in &loop_body {
                        match self.execute_statement(stmt.clone()) {
                            ExecutionResult::None => {},
                            ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                            ExecutionResult::Break => return ExecutionResult::None, // 跳出循环，但不向上传递break
                            ExecutionResult::Continue => break, // 跳过当前迭代的剩余语句，继续下一次迭代
                        }
                    }
                }
                
                ExecutionResult::None
            },
            
            Statement::WhileLoop(condition, loop_body) => {
                // 循环执行，直到条件为假
                loop {
                    // 计算条件表达式
                    let condition_value = self.evaluate_expression(&condition);
                    
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
                        match self.execute_statement(stmt.clone()) {
                            ExecutionResult::None => {},
                            ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                            ExecutionResult::Break => return ExecutionResult::None, // 跳出循环，但不向上传递break
                            ExecutionResult::Continue => break, // 跳过当前迭代的剩余语句，继续下一次迭代
                        }
                    }
                }
                
                ExecutionResult::None
            },
            Statement::Break => {
                // 返回Break结果，由循环处理
                ExecutionResult::Break
            },
            Statement::Continue => {
                // 返回Continue结果，由循环处理
                ExecutionResult::Continue
            },
            Statement::FunctionCallStatement(expr) => {
                // 执行函数调用表达式，但忽略返回值
                self.evaluate_expression(&expr);
                ExecutionResult::None
            }
        }
    }
    
    fn execute_function(&mut self, function: &Function) -> Value {
        // 执行函数体
        for statement in &function.body {
            match self.execute_statement(statement.clone()) {
                ExecutionResult::Return(value) => return value,
                ExecutionResult::None => {},
                ExecutionResult::Break => panic!("break语句只能在循环内部使用"),
                ExecutionResult::Continue => panic!("continue语句只能在循环内部使用"),
            }
        }
        
        // 如果没有返回语句，默认返回0
        Value::Int(0)
    }
    
    fn update_variable(&mut self, name: &str, value: Value) -> Result<(), String> {
        update_variable_value(&mut self.local_env, &mut self.global_env, name, value)
    }
} 