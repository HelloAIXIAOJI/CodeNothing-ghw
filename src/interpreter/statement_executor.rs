use crate::ast::{Statement, Expression, Type, NamespaceType, Function};
use super::value::Value;
use super::executor::{Executor, ExecutionResult, update_variable_value, handle_increment, handle_decrement};
use super::library_loader::{load_library, call_library_function, convert_values_to_string_args};
use super::interpreter_core::{Interpreter, debug_println};
use super::expression_evaluator::ExpressionEvaluator;

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
                self.handle_compound_assignment(name, op, expr)
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
                    _ => self.handle_import_namespace(ns_type, path)
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
                self.handle_namespaced_function_call_statement(path, args)
            },
            Statement::LibraryFunctionCallStatement(lib_name, func_name, args) => {
                self.handle_library_function_call_statement(lib_name, func_name, args)
            },
            Statement::IfElse(condition, if_block, else_blocks) => {
                self.handle_if_else(condition, if_block, else_blocks)
            },
            Statement::ForLoop(variable_name, range_start, range_end, loop_body) => {
                self.handle_for_loop(variable_name, range_start, range_end, loop_body)
            },
            Statement::ForEachLoop(variable_name, collection_expr, loop_body) => {
                self.handle_foreach_loop(variable_name, collection_expr, loop_body)
            },
            Statement::WhileLoop(condition, loop_body) => {
                self.handle_while_loop(condition, loop_body)
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
                self.handle_try_catch(try_block, catch_blocks, finally_block)
            },
            Statement::Throw(exception_expr) => {
                // 计算异常表达式并抛出
                let exception_value = self.evaluate_expression(&exception_expr);
                ExecutionResult::Throw(exception_value)
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
    fn handle_compound_assignment(&mut self, name: String, op: crate::ast::BinaryOperator, expr: Expression) -> ExecutionResult {
        // 先获取变量当前值
        let current_value = if self.local_env.contains_key(&name) {
            self.local_env.get(&name).unwrap().clone()
        } else if self.global_env.contains_key(&name) {
            self.global_env.get(&name).unwrap().clone()
        } else {
            panic!("未定义的变量: {}", name);
        };
        
        // 计算右侧表达式的值
        let right_value = self.evaluate_expression(&expr);
        
        // 执行复合赋值操作
        let new_value = self.perform_binary_operation(&current_value, &op, &right_value);
        
        // 更新变量值
        if self.local_env.contains_key(&name) {
            self.local_env.insert(name, new_value);
        } else {
            self.global_env.insert(name, new_value);
        }
        
        ExecutionResult::None
    }
    
    fn handle_import_namespace(&mut self, ns_type: NamespaceType, path: Vec<String>) -> ExecutionResult {
        match ns_type {
            NamespaceType::Code => {
                // 导入代码命名空间
                let namespace_path = path.join("::");
                debug_println(&format!("导入代码命名空间: {}", namespace_path));
                
                // 遍历命名空间中的所有函数
                let mut found = false;
                for (full_path, _) in &self.namespaced_functions {
                    // 检查函数是否属于指定的命名空间
                    if full_path.starts_with(&namespace_path) {
                        // 获取函数名（路径的最后一部分）
                        let parts: Vec<&str> = full_path.split("::").collect();
                        if let Some(func_name) = parts.last() {
                            // 将函数添加到导入的命名空间列表
                            self.imported_namespaces
                                .entry(func_name.to_string())
                                .or_insert_with(Vec::new)
                                .push(full_path.clone());
                            
                            found = true;
                            debug_println(&format!("  导入函数: {}", full_path));
                        }
                    }
                }
                
                if !found {
                    debug_println(&format!("警告: 命名空间 '{}' 中没有找到函数", namespace_path));
                }
            },
            NamespaceType::Library => {
                // 导入库命名空间
                if path.len() != 1 {
                    panic!("库名称应该是单个标识符");
                }
                
                let lib_name = &path[0];
                debug_println(&format!("导入动态库: {}", lib_name));
                
                // 尝试加载库
                match load_library(lib_name) {
                    Ok(functions) => {
                        // 库加载成功，将其添加到已导入库列表
                        self.imported_libraries.insert(lib_name.clone(), functions);
                        debug_println(&format!("库 '{}' 加载成功", lib_name));
                        
                        // 将库中的所有函数添加到全局函数列表
                        if let Some(lib_functions) = self.imported_libraries.get(lib_name) {
                            debug_println(&format!("库 '{}' 中的函数:", lib_name));
                            for (func_name, _) in lib_functions.iter() {
                                debug_println(&format!("  - {}", func_name));
                                
                                // 检查是否是命名空间函数（包含::）
                                if func_name.contains("::") {
                                    let parts: Vec<&str> = func_name.split("::").collect();
                                    if parts.len() >= 2 {
                                        let ns_name = parts[0];
                                        debug_println(&format!("  检测到命名空间: {} 在库 {}", ns_name, lib_name));
                                        // 记录命名空间与库的映射关系
                                        self.library_namespaces.insert(ns_name.to_string(), lib_name.to_string());
                                    }
                                }
                                
                                // 直接将库函数注册为全局函数，这样可以直接调用
                                self.library_functions.insert(func_name.to_string(), (lib_name.clone(), func_name.to_string()));
                            }
                        }
                    },
                    Err(err) => {
                        // 尝试查找常见的库命名约定变体
                        let lib_variants = [
                            format!("{}", lib_name),        // 原始名称
                            format!("cn_{}", lib_name),     // cn_前缀
                            format!("library_{}", lib_name) // library_前缀
                        ];
                        
                        for variant in &lib_variants {
                            if variant == lib_name {
                                continue; // 跳过已尝试过的名称
                            }
                            
                            debug_println(&format!("尝试加载替代库名称: {}", variant));
                            match load_library(variant) {
                                Ok(functions) => {
                                    // 库加载成功，将其添加到已导入库列表
                                    self.imported_libraries.insert(lib_name.clone(), functions.clone());
                                    debug_println(&format!("库 '{}' 通过替代名称 '{}' 加载成功", lib_name, variant));
                                    
                                    // 将库中的所有函数添加到全局函数列表
                                    debug_println(&format!("库 '{}' 中的函数:", lib_name));
                                    for (func_name, _) in functions.iter() {
                                        debug_println(&format!("  - {}", func_name));
                                        
                                        // 检查是否是命名空间函数（包含::）
                                        if func_name.contains("::") {
                                            let parts: Vec<&str> = func_name.split("::").collect();
                                            if parts.len() >= 2 {
                                                let ns_name = parts[0];
                                                debug_println(&format!("  检测到命名空间: {} 在库 {}", ns_name, lib_name));
                                                // 记录命名空间与库的映射关系
                                                self.library_namespaces.insert(ns_name.to_string(), lib_name.to_string());
                                            }
                                        }
                                        
                                        // 直接将库函数注册为全局函数，这样可以直接调用
                                        self.library_functions.insert(func_name.to_string(), (lib_name.clone(), func_name.to_string()));
                                    }
                                    
                                    // 成功找到替代库名称，不需要继续尝试
                                    return ExecutionResult::None;
                                },
                                Err(_) => {
                                    // 继续尝试下一个名称
                                }
                            }
                        }
                        
                        // 所有尝试都失败了，报告错误
                        panic!("无法加载库 '{}': {}。尝试了替代名称但均失败。", lib_name, err);
                    }
                }
            }
        }
        
        ExecutionResult::None
    }
    
    fn handle_namespaced_function_call_statement(&mut self, path: Vec<String>, args: Vec<Expression>) -> ExecutionResult {
        // 命名空间函数调用语句
        debug_println(&format!("命名空间函数调用: {:?}", path));

        // 检查路径长度
        if path.len() < 2 {
            panic!("无效的命名空间函数调用路径");
        }

        // 构建完整的函数路径
        let full_path = path.join("::");
        debug_println(&format!("尝试调用命名空间函数: {}", full_path));
        
        // 调试输出已注册的命名空间函数
        debug_println("已注册的命名空间函数:");
        for (path, _) in &self.namespaced_functions {
            debug_println(&format!("  - {}", path));
        }
        
        // 计算参数值
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expression(&arg));
        }
        
        // 检查是否是库函数调用
        let ns_name = &path[0];
        if let Some(lib_name) = self.library_namespaces.get(ns_name) {
            debug_println(&format!("检测到库命名空间: {} -> 库: {}", ns_name, lib_name));
            
            // 构建库函数名 - 直接使用原始命名空间路径
            let func_name = full_path.clone();
            
            debug_println(&format!("尝试调用库函数: {}", func_name));
            
            // 将参数转换为字符串
            let string_args = convert_values_to_string_args(&arg_values);
            
            // 调用库函数
            match call_library_function(lib_name, &func_name, string_args) {
                Ok(result) => {
                    debug_println(&format!("库函数调用成功: {} -> {}", func_name, result));
                    return ExecutionResult::None;
                },
                Err(err) => {
                    debug_println(&format!("调用库函数失败: {}", err));
                    // 继续尝试其他方式
                }
            }
        }
        
        // 尝试作为普通命名空间函数调用
        debug_println(&format!("尝试作为普通命名空间函数调用: {}", full_path));
        
        // 直接查找完整路径函数
        if let Some(function) = self.namespaced_functions.get(&full_path) {
            // 调用命名空间函数
            debug_println(&format!("找到并调用命名空间函数: {}", full_path));
            self.call_function_impl(function, arg_values);
            return ExecutionResult::None;
        }

        // 新增：在所有已导入库的函数表里查找完整路径（如std::println、path::join等）
        for (lib_name, lib_functions) in &self.imported_libraries {
            if let Some(func) = lib_functions.get(&full_path) {
                debug_println(&format!("在库 '{}' 中找到命名空间函数 '{}', 调用之", lib_name, full_path));
                let string_args = convert_values_to_string_args(&arg_values);
                let _ = func(string_args); // 忽略返回值（如有需要可处理）
                return ExecutionResult::None;
            }
        }
        
        // 如果是嵌套命名空间函数调用，需要特殊处理
        if path.len() > 2 {
            // 构建嵌套命名空间的完整路径
            let nested_path = path.join("::");
            debug_println(&format!("尝试调用嵌套命名空间函数: {}", nested_path));
            
            // 查找嵌套命名空间函数
            if let Some(function) = self.namespaced_functions.get(&nested_path) {
                debug_println(&format!("找到并调用嵌套命名空间函数: {}", nested_path));
                self.call_function_impl(function, arg_values);
                    return ExecutionResult::None;
            }
        }
        
        panic!("未找到命名空间函数: {}", full_path);
    }
    
    fn handle_library_function_call_statement(&mut self, lib_name: String, func_name: String, args: Vec<Expression>) -> ExecutionResult {
        // 库函数调用语句
        debug_println(&format!("库函数调用语句: {}::{}", lib_name, func_name));
        
        // 计算参数值
        let mut arg_values = Vec::new();
        for arg in args {
            let value = self.evaluate_expression(&arg);
            // 将Value转换为String
            arg_values.push(value.to_string());
        }

        // 检查库是否已加载
        if !self.imported_libraries.contains_key(&lib_name) {
            // 尝试加载库
            match load_library(&lib_name) {
                Ok(functions) => {
                    self.imported_libraries.insert(lib_name.clone(), functions);
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
    
    fn handle_if_else(&mut self, condition: Expression, if_block: Vec<Statement>, else_blocks: Vec<(Option<Expression>, Vec<Statement>)>) -> ExecutionResult {
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
                match self.execute_statement_direct(stmt.clone()) {
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
                                match self.execute_statement_direct(stmt.clone()) {
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
                            match self.execute_statement_direct(stmt.clone()) {
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
    
    fn handle_for_loop(&mut self, variable_name: String, range_start: Expression, range_end: Expression, loop_body: Vec<Statement>) -> ExecutionResult {
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
                match self.execute_statement_direct(stmt.clone()) {
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
    
    fn handle_foreach_loop(&mut self, variable_name: String, collection_expr: Expression, loop_body: Vec<Statement>) -> ExecutionResult {
        // 计算集合表达式
        let collection = self.evaluate_expression(&collection_expr);
        
        // 根据集合类型执行不同的迭代逻辑
        match collection {
            Value::Array(items) => {
                // 数组迭代
                for item in items {
                    // 在局部环境中设置迭代变量
                    self.local_env.insert(variable_name.clone(), item);
                    
                    // 执行循环体
                    for stmt in &loop_body {
                        match self.execute_statement_direct(stmt.clone()) {
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
                    self.local_env.insert(variable_name.clone(), Value::String(key.clone()));
                    
                    // 执行循环体
                    for stmt in &loop_body {
                        match self.execute_statement_direct(stmt.clone()) {
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
                    self.local_env.insert(variable_name.clone(), Value::String(c.to_string()));
                    
                    // 执行循环体
                    for stmt in &loop_body {
                        match self.execute_statement_direct(stmt.clone()) {
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
    
    fn handle_while_loop(&mut self, condition: Expression, loop_body: Vec<Statement>) -> ExecutionResult {
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
                match self.execute_statement_direct(stmt.clone()) {
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

    fn handle_try_catch(&mut self, try_block: Vec<Statement>, catch_blocks: Vec<(String, Type, Vec<Statement>)>, finally_block: Option<Vec<Statement>>) -> ExecutionResult {
        // 执行 try 块
        let try_result = {
            let mut exception_caught = false;
            let mut exception_value = None;
            
            // 执行 try 块中的语句
            for stmt in try_block {
                match self.execute_statement_direct(stmt) {
                    ExecutionResult::None => {},
                    ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                    ExecutionResult::Break => return ExecutionResult::Break,
                    ExecutionResult::Continue => return ExecutionResult::Continue,
                    ExecutionResult::Throw(value) => {
                        exception_caught = true;
                        exception_value = Some(value);
                        break;
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
                self.local_env.insert(exception_name, exception_value.clone());
                
                // 执行 catch 块
                for stmt in catch_block {
                    match self.execute_statement_direct(stmt) {
                        ExecutionResult::None => {},
                        ExecutionResult::Return(value) => {
                            // 执行 finally 块（如果存在）
                            if let Some(ref finally_block) = finally_block {
                                for stmt in finally_block {
                                    self.execute_statement_direct(stmt.clone());
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
                                    self.execute_statement_direct(stmt.clone());
                                }
                            }
                            return ExecutionResult::Throw(value);
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
                match self.execute_statement_direct(stmt) {
                    ExecutionResult::None => {},
                    ExecutionResult::Return(value) => return ExecutionResult::Return(value),
                    ExecutionResult::Break => return ExecutionResult::Break,
                    ExecutionResult::Continue => return ExecutionResult::Continue,
                    ExecutionResult::Throw(value) => return ExecutionResult::Throw(value),
                }
            }
        }
        
        ExecutionResult::None
    }
} 