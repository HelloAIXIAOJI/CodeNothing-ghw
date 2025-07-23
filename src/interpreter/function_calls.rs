use crate::ast::{Expression, Function};
use std::collections::HashMap;
use super::value::Value;
use super::library_loader::{call_library_function, convert_values_to_string_args};
use super::interpreter_core::{Interpreter, debug_println};
use super::expression_evaluator::ExpressionEvaluator;

pub trait FunctionCallHandler {
    fn handle_function_call(&mut self, name: &str, args: &[Expression]) -> Value;
    fn handle_namespaced_function_call(&mut self, path: &[String], args: &[Expression]) -> Value;
    fn handle_global_function_call(&mut self, name: &str, args: &[Expression]) -> Value;
    fn handle_library_function_call(&mut self, lib_name: &str, func_name: &str, args: &[Expression]) -> Value;
}

impl<'a> FunctionCallHandler for Interpreter<'a> {
    fn handle_function_call(&mut self, name: &str, args: &[Expression]) -> Value {
        // 检查是否是命名空间函数调用（包含::）
        if name.contains("::") {
            debug_println(&format!("检测到命名空间函数调用: {}", name));
            let path: Vec<String> = name.split("::").map(|s| s.to_string()).collect();
            
            // 计算所有参数值
            let mut arg_values = Vec::new();
            for arg_expr in args {
                arg_values.push(self.evaluate_expression(arg_expr));
            }
            
            // 检查是否是库命名空间函数
            let ns_name = &path[0];
            if let Some(lib_name) = self.library_namespaces.get(ns_name) {
                debug_println(&format!("检测到库命名空间: {} -> 库: {}", ns_name, lib_name));
                
                // 将参数转换为字符串
                let string_args = convert_values_to_string_args(&arg_values);
                
                // 尝试调用库函数 - 使用完整的命名空间路径
                match call_library_function(lib_name, name, string_args) {
                    Ok(result) => {
                        debug_println(&format!("库函数调用成功: {} -> {}", name, result));
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
                        debug_println(&format!("调用库函数失败: {}", err));
                        // 继续尝试其他方式
                    }
                }
            }
            
            // 尝试在所有库中查找该函数
            for (lib_name, lib_functions) in &self.imported_libraries {
                debug_println(&format!("尝试在库 '{}' 中查找函数 '{}'", lib_name, name));
                
                if let Some(func) = lib_functions.get(name) {
                    debug_println(&format!("在库 '{}' 中找到函数 '{}'", lib_name, name));
                    
                    // 将参数转换为字符串
                    let string_args = convert_values_to_string_args(&arg_values);
                    
                    let result = func(string_args);
                    debug_println(&format!("库函数调用成功: {} -> {}", name, result));
                    
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
                }
            }
            
            // 查找命名空间函数
            if let Some(function) = self.namespaced_functions.get(name) {
                debug_println(&format!("找到并调用嵌套命名空间函数: {}", name));
                return self.call_function_impl(function, arg_values);
            }
            
            // 如果找不到，尝试将其转换为NamespacedFunctionCall处理
            debug_println(&format!("转换为NamespacedFunctionCall处理: {}", name));
            return self.handle_namespaced_function_call(&path, args);
        }
        
        // 先计算所有参数值
        let mut arg_values = Vec::new();
        for arg_expr in args {
            arg_values.push(self.evaluate_expression(arg_expr));
        }
        
        // 检查是否是库函数
        if let Some((lib_name, func_name)) = self.library_functions.get(name) {
            debug_println(&format!("调用库函数: {}", func_name));
            
            // 使用新函数将参数转换为字符串
            let string_args = convert_values_to_string_args(&arg_values);
            
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
                    
                    // 使用新函数将参数转换为字符串
                    let string_args = convert_values_to_string_args(&arg_values);
                    
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
                            debug_println(&format!("调用库函数失败: {}", err));
                        }
                    }
                }
            }
        }
        
        // 检查是否是嵌套命名空间函数调用
        if name.contains("::") {
            let path: Vec<String> = name.split("::").map(|s| s.to_string()).collect();
            debug_println(&format!("检测到嵌套命名空间函数调用: {}", name));
            
            // 查找命名空间函数
            if let Some(function) = self.namespaced_functions.get(name) {
                debug_println(&format!("找到并调用嵌套命名空间函数: {}", name));
                return self.call_function_impl(function, arg_values);
            } else {
                debug_println(&format!("未找到嵌套命名空间函数: {}", name));
                
                // 尝试解析为命名空间函数调用
                if path.len() >= 2 {
                    // 最后一部分是函数名
                    let func_name = path.last().unwrap();
                    
                    // 前面部分是命名空间路径
                    debug_println(&format!("尝试查找函数 '{}' 在命名空间中", func_name));
                    
                    // 遍历所有已注册的命名空间函数
                    for (ns_path, ns_func) in &self.namespaced_functions {
                        if ns_path.ends_with(&format!("::{}", func_name)) {
                            debug_println(&format!("找到匹配的命名空间函数: {}", ns_path));
                            return self.call_function_impl(ns_func, arg_values);
                        }
                    }
                }
            }
        }
        
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
        
        // 尝试在所有库中查找该函数
        let string_args = convert_values_to_string_args(&arg_values);
        for (lib_name, lib_functions) in &self.imported_libraries {
            // 尝试直接查找函数名
            debug_println(&format!("尝试在库 '{}' 中查找函数 '{}'", lib_name, name));
            
            if let Some(func) = lib_functions.get(name) {
                debug_println(&format!("在库 '{}' 中找到函数 '{}'", lib_name, name));
                let result = func(string_args.clone());
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
            }
            
            // 尝试查找命名空间函数
            for ns_name in self.library_namespaces.keys() {
                let ns_func_name = format!("{}::{}", ns_name, name);
                debug_println(&format!("尝试在库 '{}' 中查找命名空间函数 '{}'", lib_name, ns_func_name));
                
                if let Some(func) = lib_functions.get(&ns_func_name) {
                    debug_println(&format!("在库 '{}' 中找到命名空间函数 '{}'", lib_name, ns_func_name));
                    let result = func(string_args.clone());
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
                }
            }
        }
        
        // 如果不是导入的函数，再检查全局函数
        if let Some(function) = self.functions.get(name) {
            debug_println(&format!("找到全局函数: {}", name));
            // 执行全局函数
            self.call_function_impl(function, arg_values)
        } else {
            // 最后一次尝试，检查是否是嵌套命名空间中的函数
            let mut found = false;
            for (ns_path, ns_func) in &self.namespaced_functions {
                if ns_path.ends_with(&format!("::{}", name)) {
                    debug_println(&format!("找到嵌套命名空间中的函数: {}", ns_path));
                    found = true;
                    return self.call_function_impl(ns_func, arg_values);
                }
            }
            
            if !found {
                panic!("未定义的函数: {}", name);
            }
            
            // 这里不会执行到，只是为了编译通过
            unreachable!();
        }
    }

    fn handle_namespaced_function_call(&mut self, path: &[String], args: &[Expression]) -> Value {
        // 构建完整的函数路径
        let full_path = path.join("::");
        
        // 先计算所有参数值
        let mut arg_values = Vec::new();
        for arg_expr in args {
            arg_values.push(self.evaluate_expression(arg_expr));
        }
        
        debug_println(&format!("调用命名空间函数: {}", full_path));
        
        // 检查是否是库命名空间函数
        if path.len() >= 2 {
            let ns_name = &path[0];
            if let Some(lib_name) = self.library_namespaces.get(ns_name) {
                debug_println(&format!("检测到库命名空间: {} -> 库: {}", ns_name, lib_name));
                
                // 将参数转换为字符串
                let string_args = convert_values_to_string_args(&arg_values);
                
                // 尝试调用库函数 - 使用完整的命名空间路径
                match call_library_function(lib_name, &full_path, string_args) {
                    Ok(result) => {
                        debug_println(&format!("库函数调用成功: {} -> {}", full_path, result));
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
                        debug_println(&format!("调用库函数失败: {}", err));
                        // 继续尝试其他方式
                    }
                }
            }
        }
        
        // 查找命名空间函数
        if let Some(function) = self.namespaced_functions.get(&full_path) {
            self.call_function_impl(function, arg_values)
        } else {
            // 检查是否是导入命名空间的嵌套命名空间函数
            let mut found = false;
            
            // 尝试各种可能的路径组合
            for (key, _) in &self.imported_namespaces {
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
            
            // 如果是两级以上的路径，尝试查找完整路径
            if !found && path.len() >= 2 {
                debug_println(&format!("尝试查找完整路径函数: {}", full_path));
                
                if let Some(function) = self.namespaced_functions.get(&full_path) {
                    found = true;
                    return self.call_function_impl(function, arg_values);
                }
            }
            
            // 尝试在所有库中查找该命名空间函数
            if !found {
                let string_args = convert_values_to_string_args(&arg_values);
                for (lib_name, lib_functions) in &self.imported_libraries {
                    debug_println(&format!("尝试在库 '{}' 中查找命名空间函数 '{}'", lib_name, full_path));
                    
                    if let Some(func) = lib_functions.get(&full_path) {
                        debug_println(&format!("在库 '{}' 中找到命名空间函数 '{}'", lib_name, full_path));
                        let result = func(string_args.clone());
                        found = true;
                        
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
                    }
                }
            }
            
            // 检查是否为静态方法调用
            if !found {
                let parts: Vec<&str> = full_path.split("::").collect();
                if parts.len() == 2 {
                    let class_name = parts[0];
                    let method_name = parts[1];
                    
                    if let Some(class) = self.classes.get(class_name) {
                        if let Some(method) = class.methods.iter().find(|m| m.is_static && m.name == method_name) {
                            // 创建方法参数环境
                            let mut method_env = HashMap::new();
                            for (i, param) in method.parameters.iter().enumerate() {
                                if i < arg_values.len() {
                                    method_env.insert(param.name.clone(), arg_values[i].clone());
                                }
                            }
                            
                            // 简单执行静态方法体
                            for statement in &method.body {
                                if let crate::ast::Statement::Return(expr) = statement {
                                    // 简单的变量替换
                                    if let crate::ast::Expression::Variable(var_name) = expr {
                                        if let Some(value) = method_env.get(var_name) {
                                            return value.clone();
                                        }
                                    } else if let crate::ast::Expression::BinaryOp(left, op, right) = expr {
                                        // 简单的二元操作
                                        let left_val = if let crate::ast::Expression::Variable(var) = &**left {
                                            method_env.get(var).cloned().unwrap_or(Value::None)
                                        } else {
                                            self.evaluate_expression(left)
                                        };
                                        let right_val = if let crate::ast::Expression::Variable(var) = &**right {
                                            method_env.get(var).cloned().unwrap_or(Value::None)
                                        } else {
                                            self.evaluate_expression(right)
                                        };
                                        
                                        if let crate::ast::BinaryOperator::Add = op {
                                            match (&left_val, &right_val) {
                                                (Value::Int(a), Value::Int(b)) => return Value::Int(a + b),
                                                (Value::Float(a), Value::Float(b)) => return Value::Float(a + b),
                                                (Value::String(a), Value::String(b)) => return Value::String(a.clone() + b),
                                                _ => return Value::None,
                                            }
                                        } else if let crate::ast::BinaryOperator::Multiply = op {
                                            match (&left_val, &right_val) {
                                                (Value::Int(a), Value::Int(b)) => return Value::Int(a * b),
                                                (Value::Float(a), Value::Float(b)) => return Value::Float(a * b),
                                                _ => return Value::None,
                                            }
                                        } else if let crate::ast::BinaryOperator::Subtract = op {
                                            match (&left_val, &right_val) {
                                                (Value::Int(a), Value::Int(b)) => return Value::Int(a - b),
                                                (Value::Float(a), Value::Float(b)) => return Value::Float(a - b),
                                                _ => return Value::None,
                                            }
                                        }
                                    }
                                    return self.evaluate_expression(expr);
                                }
                            }
                            return Value::None;
                        }
                    }
                }
                
                panic!("未定义的命名空间函数或静态方法: {}", full_path);
            }
            
            // 这里不会执行到，只是为了编译通过
            unreachable!();
        }
    }

    fn handle_global_function_call(&mut self, name: &str, args: &[Expression]) -> Value {
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
    }

    fn handle_library_function_call(&mut self, lib_name: &str, func_name: &str, args: &[Expression]) -> Value {
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
                match super::library_loader::load_library(lib_name) {
                Ok(functions) => {
                    self.imported_libraries.insert(lib_name.to_string(), functions);
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