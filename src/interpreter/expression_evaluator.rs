use crate::ast::{Expression, BinaryOperator, CompareOperator, LogicalOperator};
use super::value::{Value, ObjectInstance};
use super::interpreter_core::{Interpreter, debug_println};
use std::collections::HashMap;
use super::function_calls::FunctionCallHandler;
use super::jit;

pub trait ExpressionEvaluator {
    fn evaluate_expression(&mut self, expr: &Expression) -> Value;
    fn perform_binary_operation(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Value;
    fn get_variable(&self, name: &str) -> Option<Value>;
    fn is_pure_int_expression(&self, expr: &Expression) -> bool;
}

impl<'a> ExpressionEvaluator for Interpreter<'a> {
    fn evaluate_expression(&mut self, expr: &Expression) -> Value {
        // 检查是否包含方法调用，如果包含则跳过JIT编译
        if !self.contains_method_call(expr) && self.is_pure_int_expression(expr) {
        if let Some(val) = jit::jit_eval_const_expr(expr) {
            return val;
        }
        // 尝试整体JIT int型带变量表达式
        if let Some(jit_expr) = jit::jit_compile_int_expr(expr) {
            // 收集变量名和当前作用域变量值
            let mut vars = std::collections::HashMap::new();
            for name in &jit_expr.var_names {
                // 只支持Int类型变量
                let val = if let Some(Value::Int(i)) = self.local_env.get(name) {
                    *i as i64
                } else if let Some(Value::Int(i)) = self.global_env.get(name) {
                    *i as i64
                } else {
                    panic!("JIT表达式变量{}未赋Int值", name);
                };
                vars.insert(name.clone(), val);
            }
            let result = jit_expr.call(&vars);
            return Value::Int(result as i32);
            }
        }
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
                let mut map = std::collections::HashMap::new();
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
                self.handle_function_call(name, args)
            },
            Expression::GlobalFunctionCall(name, args) => {
                self.handle_global_function_call(name, args)
            },
            Expression::NamespacedFunctionCall(path, args) => {
                self.handle_namespaced_function_call(path, args)
            },
            Expression::Variable(name) => {
                // 先检查常量
                if let Some(value) = self.constants.get(name) {
                    return value.clone();
                }
                
                // 再检查局部变量
                if let Some(value) = self.local_env.get(name) {
                    return value.clone();
                }
                
                // 最后检查全局变量
                if let Some(value) = self.global_env.get(name) {
                    return value.clone();
                }
                
                // 如果都找不到，返回None
                Value::None
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                
                self.perform_binary_operation(&left_val, op, &right_val)
            },
            Expression::CompareOp(left, op, right) => {
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                
                self.evaluate_compare_operation(&left_val, op, &right_val)
            },
            Expression::LogicalOp(left, op, right) => {
                self.evaluate_logical_operation(left, op, right)
            },
            Expression::TernaryOp(condition, true_expr, false_expr) => {
                self.evaluate_ternary_operation(condition, true_expr, false_expr)
            },
            Expression::PreIncrement(name) => {
                self.evaluate_pre_increment(name)
            },
            Expression::PreDecrement(name) => {
                self.evaluate_pre_decrement(name)
            },
            Expression::PostIncrement(name) => {
                self.evaluate_post_increment(name)
            },
            Expression::PostDecrement(name) => {
                self.evaluate_post_decrement(name)
            },
            Expression::LibraryFunctionCall(lib_name, func_name, args) => {
                self.handle_library_function_call(lib_name, func_name, args)
            },
            Expression::MethodCall(obj_expr, method_name, args) => {
                self.handle_method_call(obj_expr, method_name, args)
            },
            Expression::ChainCall(obj_expr, chain_calls) => {
                self.handle_chain_call(obj_expr, chain_calls)
            },
            Expression::Throw(exception_expr) => {
                // 计算异常表达式并抛出
                let exception_value = self.evaluate_expression(exception_expr);
                // 注意：这里我们返回异常值，但实际的抛出逻辑在语句执行器中处理
                exception_value
            },
            // OOP相关表达式的实现
            Expression::ObjectCreation(class_name, args) => {
                self.create_object(class_name, args)
            },
            Expression::FieldAccess(obj_expr, field_name) => {
                self.access_field(obj_expr, field_name)
            },
            Expression::This => {
                // TODO: 实现this关键字，需要当前对象上下文
                Value::None
            },
        }
    }
    
    fn perform_binary_operation(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Value {
        use super::evaluator::perform_binary_operation;
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
    
    fn is_pure_int_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::IntLiteral(_) => true,
            Expression::Variable(name) => {
                // 检查变量是否为int类型
                if let Some(Value::Int(_)) = self.local_env.get(name) {
                    true
                } else if let Some(Value::Int(_)) = self.global_env.get(name) {
                    true
                } else {
                    false
                }
            },
            Expression::BinaryOp(left, _, right) => {
                self.is_pure_int_expression(left) && self.is_pure_int_expression(right)
            },
            Expression::CompareOp(left, _, right) => {
                self.is_pure_int_expression(left) && self.is_pure_int_expression(right)
            },
            Expression::LogicalOp(left, _, right) => {
                self.is_pure_int_expression(left) && self.is_pure_int_expression(right)
            },
            Expression::TernaryOp(cond, true_expr, false_expr) => {
                self.is_pure_int_expression(cond) && self.is_pure_int_expression(true_expr) && self.is_pure_int_expression(false_expr)
            },
            _ => false,
        }
    }
    
}

impl<'a> Interpreter<'a> {
    fn evaluate_compare_operation(&self, left: &Value, op: &CompareOperator, right: &Value) -> Value {
        use super::evaluator::evaluate_compare_operation;
        evaluate_compare_operation(left, op, right)
    }
    
    fn evaluate_logical_operation(&mut self, left: &Expression, op: &LogicalOperator, right: &Expression) -> Value {
        match op {
            LogicalOperator::And => {
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                match (left_val, right_val) {
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(jit::jit_and_bool(a, b)),
                    _ => panic!("逻辑操作符的操作数必须是布尔类型"),
                }
            },
            LogicalOperator::Or => {
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                match (left_val, right_val) {
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(jit::jit_or_bool(a, b)),
                    _ => panic!("逻辑操作符的操作数必须是布尔类型"),
                }
            },
            LogicalOperator::Not => {
                let val = self.evaluate_expression(left);
                match val {
                    Value::Bool(b) => Value::Bool(jit::jit_not_bool(b)),
                    _ => panic!("逻辑操作符的操作数必须是布尔类型"),
                }
            }
        }
    }
    
    fn evaluate_ternary_operation(&mut self, condition: &Expression, true_expr: &Expression, false_expr: &Expression) -> Value {
        // 三元运算符：先计算条件，然后根据条件计算相应的表达式
        let condition_val = self.evaluate_expression(condition);
        
        match condition_val {
            Value::Bool(true) => self.evaluate_expression(true_expr),
            Value::Bool(false) => self.evaluate_expression(false_expr),
            _ => panic!("三元运算符的条件必须是布尔类型"),
        }
    }
    
    fn evaluate_pre_increment(&mut self, name: &str) -> Value {
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
            self.local_env.insert(name.to_string(), new_value.clone());
        } else {
            self.global_env.insert(name.to_string(), new_value.clone());
        }
        
        // 返回新值
        new_value
    }
    
    fn evaluate_pre_decrement(&mut self, name: &str) -> Value {
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
            self.local_env.insert(name.to_string(), new_value.clone());
        } else {
            self.global_env.insert(name.to_string(), new_value.clone());
        }
        
        // 返回新值
        new_value
    }
    
    fn evaluate_post_increment(&mut self, name: &str) -> Value {
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
            self.local_env.insert(name.to_string(), new_value);
        } else {
            self.global_env.insert(name.to_string(), new_value);
        }
        
        // 返回原值
        value
    }
    
    fn evaluate_post_decrement(&mut self, name: &str) -> Value {
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
            self.local_env.insert(name.to_string(), new_value);
        } else {
            self.global_env.insert(name.to_string(), new_value);
        }
        
        // 返回原值
        value
    }
    
    fn handle_method_call(&mut self, obj_expr: &Expression, method_name: &str, args: &[Expression]) -> Value {
        // 计算对象表达式
        let obj_value = self.evaluate_expression(obj_expr);
        
        // 计算参数
        let mut evaluated_args = Vec::new();
        for arg in args {
            let arg_value = self.evaluate_expression(arg);
            evaluated_args.push(arg_value.to_string());
        }
        
        // 根据对象类型调用相应的方法
        match obj_value {
            Value::String(s) => {
                // 字符串方法调用
                self.handle_string_method(&s, method_name, &evaluated_args)
            },
            Value::Array(arr) => {
                // 数组方法调用
                self.handle_array_method(&arr, method_name, &evaluated_args)
            },
            Value::Map(map) => {
                // 映射方法调用
                self.handle_map_method(&map, method_name, &evaluated_args)
            },
            _ => {
                // 不支持的对象类型
                panic!("不支持对类型 {:?} 调用方法 {}", obj_value, method_name)
            }
        }
    }
    
    fn handle_chain_call(&mut self, obj_expr: &Expression, chain_calls: &[(String, Vec<Expression>)]) -> Value {
        // 计算初始对象
        let mut current_value = self.evaluate_expression(obj_expr);
        
        // 依次执行链式调用
        for (method_name, args) in chain_calls {
            // 计算参数
            let mut evaluated_args = Vec::new();
            for arg in args {
                let arg_value = self.evaluate_expression(arg);
                evaluated_args.push(arg_value.to_string());
            }
            
            // 根据当前值类型调用相应的方法
            current_value = match &current_value {
                Value::String(s) => {
                    self.handle_string_method(s, method_name, &evaluated_args)
                },
                Value::Array(arr) => {
                    self.handle_array_method(arr, method_name, &evaluated_args)
                },
                Value::Map(map) => {
                    self.handle_map_method(map, method_name, &evaluated_args)
                },
                _ => {
                    // 不支持的对象类型
                    panic!("不支持对类型 {:?} 调用方法 {}", current_value, method_name)
                }
            };
        }
        
        current_value
    }
    
    fn handle_string_method(&mut self, s: &str, method_name: &str, args: &[String]) -> Value {
        match method_name {
            "length" => {
                if args.is_empty() {
                    Value::Int(s.len() as i32)
                } else {
                    panic!("length方法不接受参数")
                }
            },
            "substring" => {
                if args.len() == 2 {
                    if let (Ok(start), Ok(end)) = (args[0].parse::<usize>(), args[1].parse::<usize>()) {
                        if start < s.len() && end <= s.len() && start < end {
                            Value::String(s[start..end].to_string())
                        } else {
                            Value::String("".to_string())
                        }
                    } else {
                        panic!("substring方法的参数必须是整数")
                    }
                } else {
                    panic!("substring方法需要两个参数")
                }
            },
            "to_upper" => {
                if args.is_empty() {
                    Value::String(s.to_uppercase())
                } else {
                    panic!("to_upper方法不接受参数")
                }
            },
            "to_lower" => {
                if args.is_empty() {
                    Value::String(s.to_lowercase())
                } else {
                    panic!("to_lower方法不接受参数")
                }
            },
            "trim" => {
                if args.is_empty() {
                    Value::String(s.trim().to_string())
                } else {
                    panic!("trim方法不接受参数")
                }
            },
            _ => {
                // 未知的字符串方法
                panic!("未知的字符串方法: {}", method_name)
            }
        }
    }
    
    fn handle_array_method(&mut self, arr: &[Value], method_name: &str, args: &[String]) -> Value {
        match method_name {
            "length" => {
                if args.is_empty() {
                    Value::Int(arr.len() as i32)
                } else {
                    panic!("length方法不接受参数")
                }
            },
            "push" => {
                if args.len() == 1 {
                    let mut new_arr = arr.to_vec();
                    new_arr.push(Value::String(args[0].clone()));
                    Value::Array(new_arr)
                } else {
                    panic!("push方法需要一个参数")
                }
            },
            "pop" => {
                if args.is_empty() {
                    let mut new_arr = arr.to_vec();
                    if let Some(last) = new_arr.pop() {
                        last
                    } else {
                        Value::None
                    }
                } else {
                    panic!("pop方法不接受参数")
                }
            },
            _ => {
                panic!("未知的数组方法: {}", method_name)
            }
        }
    }
    
    fn handle_map_method(&mut self, map: &std::collections::HashMap<String, Value>, method_name: &str, args: &[String]) -> Value {
        match method_name {
            "size" => {
                if args.is_empty() {
                    Value::Int(map.len() as i32)
                } else {
                    panic!("size方法不接受参数")
                }
            },
            "get" => {
                if args.len() == 1 {
                    if let Some(value) = map.get(&args[0]) {
                        value.clone()
                    } else {
                        Value::None
                    }
                } else {
                    panic!("get方法需要一个参数")
                }
            },
            "set" => {
                if args.len() == 2 {
                    let mut new_map = map.clone();
                    new_map.insert(args[0].clone(), Value::String(args[1].clone()));
                    Value::Map(new_map)
                } else {
                    panic!("set方法需要两个参数")
                }
            },
            _ => {
                panic!("未知的映射方法: {}", method_name)
            }
        }
    }
    
    fn contains_method_call(&self, expr: &Expression) -> bool {
        match expr {
            Expression::MethodCall(_, _, _) | Expression::ChainCall(_, _) => true,
            Expression::BinaryOp(left, _, right) => {
                self.contains_method_call(left) || self.contains_method_call(right)
            },
            Expression::CompareOp(left, _, right) => {
                self.contains_method_call(left) || self.contains_method_call(right)
            },
            Expression::LogicalOp(left, _, right) => {
                self.contains_method_call(left) || self.contains_method_call(right)
            },
            Expression::TernaryOp(cond, true_expr, false_expr) => {
                self.contains_method_call(cond) || self.contains_method_call(true_expr) || self.contains_method_call(false_expr)
            },
            Expression::FunctionCall(_, args) => {
                args.iter().any(|arg| self.contains_method_call(arg))
            },
            Expression::NamespacedFunctionCall(_, args) => {
                args.iter().any(|arg| self.contains_method_call(arg))
            },
            Expression::GlobalFunctionCall(_, args) => {
                args.iter().any(|arg| self.contains_method_call(arg))
            },
            Expression::LibraryFunctionCall(_, _, args) => {
                args.iter().any(|arg| self.contains_method_call(arg))
            },
            Expression::ArrayLiteral(elements) => {
                elements.iter().any(|elem| self.contains_method_call(elem))
            },
            Expression::MapLiteral(entries) => {
                entries.iter().any(|(key, value)| {
                    self.contains_method_call(key) || self.contains_method_call(value)
                })
            },
            Expression::Throw(expr) => {
                self.contains_method_call(expr)
            },
            _ => false,
        }
    }
    
    // OOP相关方法
    fn create_object(&mut self, class_name: &str, args: &[Expression]) -> Value {
        // 查找类定义
        let class = match self.classes.get(class_name) {
            Some(class) => *class,
            None => {
                eprintln!("错误: 未找到类 '{}'", class_name);
                return Value::None;
            }
        };
        
        // 计算构造函数参数
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expression(arg));
        }
        
        // 创建对象实例
        let mut fields = HashMap::new();
        
        // 初始化字段为默认值
        for field in &class.fields {
            let default_value = match field.initial_value {
                Some(ref expr) => self.evaluate_expression(expr),
                None => match field.field_type {
                    crate::ast::Type::Int => Value::Int(0),
                    crate::ast::Type::Float => Value::Float(0.0),
                    crate::ast::Type::Bool => Value::Bool(false),
                    crate::ast::Type::String => Value::String(String::new()),
                    crate::ast::Type::Long => Value::Long(0),
                    _ => Value::None,
                }
            };
            fields.insert(field.name.clone(), default_value);
        }
        
        let object = ObjectInstance {
            class_name: class_name.to_string(),
            fields,
        };
        
        // TODO: 调用构造函数
        // 这里需要实现构造函数的调用逻辑
        
        Value::Object(object)
    }
    
    fn access_field(&mut self, obj_expr: &Expression, field_name: &str) -> Value {
        let obj_value = self.evaluate_expression(obj_expr);
        
        match obj_value {
            Value::Object(obj) => {
                match obj.fields.get(field_name) {
                    Some(value) => value.clone(),
                    None => {
                        eprintln!("错误: 对象 '{}' 没有字段 '{}'", obj.class_name, field_name);
                        Value::None
                    }
                }
            },
            _ => {
                eprintln!("错误: 尝试访问非对象的字段");
                Value::None
            }
        }
    }
} 