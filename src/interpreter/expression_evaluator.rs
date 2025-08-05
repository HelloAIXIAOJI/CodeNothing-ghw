use crate::ast::{Expression, BinaryOperator, CompareOperator, LogicalOperator, SwitchCase, CasePattern};
use super::value::{Value, ObjectInstance, EnumInstance, PointerInstance, PointerType, FunctionPointerInstance, LambdaFunctionPointerInstance, PointerError};
use super::memory_manager::{allocate_memory_smart, read_memory, write_memory, is_valid_address, is_null_pointer, validate_pointer, is_dangling_pointer, read_memory_safe, validate_pointer_safe, is_dangling_pointer_by_address, safe_pointer_arithmetic};
use super::interpreter_core::{Interpreter, debug_println, VariableLocation};
use std::collections::HashMap;
use super::function_calls::FunctionCallHandler;
use super::statement_executor::StatementExecutor;
use super::jit;

pub trait ExpressionEvaluator {
    fn evaluate_expression(&mut self, expr: &Expression) -> Value;
    fn perform_binary_operation(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Value;
    fn get_variable(&self, name: &str) -> Option<Value>;
    fn is_pure_int_expression(&self, expr: &Expression) -> bool;
}

impl<'a> Interpreter<'a> {
    /// 快速变量查找，暂时禁用缓存以避免副作用
    pub fn get_variable_fast(&mut self, name: &str) -> Value {
        // 暂时禁用缓存，直接使用原有的查找逻辑
        if let Some(value) = self.constants.get(name) {
            return value.clone();
        }

        if let Some(value) = self.local_env.get(name) {
            return value.clone();
        }

        if let Some(value) = self.global_env.get(name) {
            return value.clone();
        }

        if self.functions.contains_key(name) {
            return self.create_function_pointer(name);
        }

        Value::None
    }

    /// 检查是否应该尝试数学表达式JIT优化
    fn should_try_math_jit_optimization(&self, expr: &Expression) -> bool {
        match expr {
            Expression::BinaryOp(_, op, _) => {
                // v0.7.2更新：包含位运算符的JIT优化
                matches!(op,
                    BinaryOperator::Add | BinaryOperator::Subtract |
                    BinaryOperator::Multiply | BinaryOperator::Divide |
                    BinaryOperator::Modulo |
                    // 新增位运算符
                    BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOr |
                    BinaryOperator::BitwiseXor | BinaryOperator::LeftShift |
                    BinaryOperator::RightShift
                )
            },
            _ => false
        }
    }

    /// 检查是否应该尝试JIT优化
    fn should_try_jit_optimization(&self, expr: &Expression) -> bool {
        // 只对包含变量的算术表达式进行JIT优化
        match expr {
            Expression::BinaryOp(_, _, _) => self.contains_variables(expr),
            Expression::PreIncrement(_) | Expression::PreDecrement(_) |
            Expression::PostIncrement(_) | Expression::PostDecrement(_) => true,
            _ => false
        }
    }

    /// 检查表达式是否包含变量
    fn contains_variables(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Variable(_) => true,
            Expression::BinaryOp(left, _, right) => {
                self.contains_variables(left) || self.contains_variables(right)
            },
            Expression::PreIncrement(_) | Expression::PreDecrement(_) |
            Expression::PostIncrement(_) | Expression::PostDecrement(_) => true,
            _ => false
        }
    }

    /// 尝试使用数学表达式JIT编译
    fn try_math_jit_expression(&self, expr: &Expression) -> Option<Value> {
        // 生成数学表达式的唯一键
        let key = format!("math_expr_{:p}", expr as *const _);

        // 获取JIT编译器并检查是否应该编译
        let jit = jit::get_jit();

        // 首先检查是否应该编译（这会增加热点计数器）
        if jit.should_compile_math_expression(&key) {
            // 尝试编译数学表达式
            match jit.compile_math_expression(expr, key.clone(), true) {
                Ok(_compiled) => {
                    println!("✅ 数学表达式JIT编译成功: {}", key);
                    // 编译成功，记录在统计中
                    // 注意：这里我们暂时返回None，因为实际执行需要更复杂的实现
                    // 但是编译过程已经被记录在统计中
                },
                Err(e) => {
                    println!("❌ 数学表达式JIT编译失败: {} - {}", key, e);
                    // 编译失败，继续使用解释执行
                }
            }
        }

        None
    }

    /// 尝试使用JIT编译执行表达式
    fn try_jit_expression(&self, expr: &Expression) -> Option<Value> {
        // 收集当前环境中的整数变量
        let mut variables = HashMap::new();
        self.collect_int_variables(expr, &mut variables);

        // 如果所有变量都是整数，尝试JIT编译
        if !variables.is_empty() {
            if let Some(result) = jit::jit_compile_and_execute_expression(expr, &variables) {
                return Some(result);
            }
        }

        None
    }

    /// 收集表达式中的整数变量及其值
    fn collect_int_variables(&self, expr: &Expression, variables: &mut HashMap<String, i64>) {
        match expr {
            Expression::Variable(name) => {
                if let Some(value) = self.get_variable_value_as_int(name) {
                    variables.insert(name.clone(), value);
                }
            },
            Expression::BinaryOp(left, _, right) => {
                self.collect_int_variables(left, variables);
                self.collect_int_variables(right, variables);
            },
            Expression::PreIncrement(name) | Expression::PreDecrement(name) |
            Expression::PostIncrement(name) | Expression::PostDecrement(name) => {
                if let Some(value) = self.get_variable_value_as_int(name) {
                    variables.insert(name.clone(), value);
                }
            },
            _ => {}
        }
    }

    /// 获取变量的整数值
    fn get_variable_value_as_int(&self, name: &str) -> Option<i64> {
        let value = if let Some(v) = self.constants.get(name) {
            v
        } else if let Some(v) = self.local_env.get(name) {
            v
        } else if let Some(v) = self.global_env.get(name) {
            v
        } else {
            return None;
        };

        match value {
            Value::Int(i) => Some(*i as i64),
            Value::Long(l) => Some(*l),
            _ => None
        }
    }

    /// 检查是否为纯常量表达式（可以在编译时求值）
    fn is_pure_constant_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::IntLiteral(_) | Expression::FloatLiteral(_) |
            Expression::BoolLiteral(_) | Expression::StringLiteral(_) |
            Expression::LongLiteral(_) => true,
            Expression::BinaryOp(left, _, right) => {
                self.is_pure_constant_expression(left) && self.is_pure_constant_expression(right)
            },
            // 注意：AST中没有UnaryOp，先注释掉
            // Expression::UnaryOp(_, operand) => self.is_pure_constant_expression(operand),
            _ => false,
        }
    }
}

impl<'a> ExpressionEvaluator for Interpreter<'a> {
    fn evaluate_expression(&mut self, expr: &Expression) -> Value {
        // 启用常量表达式JIT优化
        if self.is_pure_constant_expression(expr) {
            if let Some(val) = jit::jit_eval_const_expr(expr) {
                return val;
            }
        }

        // 快速路径：直接处理简单表达式，避免递归调用开销
        match expr {
            Expression::IntLiteral(i) => return Value::Int(*i),
            Expression::FloatLiteral(f) => return Value::Float(*f),
            Expression::BoolLiteral(b) => return Value::Bool(*b),
            Expression::StringLiteral(s) => return Value::String(s.clone()),
            Expression::LongLiteral(l) => return Value::Long(*l),
            Expression::Variable(name) => {
                // 优化变量查找：使用更高效的查找顺序
                return self.get_variable_fast(name);
            },
            _ => {} // 继续处理复杂表达式
        }
        //             panic!("JIT表达式变量{}未赋Int值", name);
        //         };
        //         vars.insert(name.clone(), val);
        //     }
        //     let result = jit_expr.call(&vars);
        //     return Value::Int(result as i32);
        //     }
        // }
        match expr {
            Expression::IntLiteral(value) => Value::Int(*value),
            Expression::FloatLiteral(value) => Value::Float(*value),
            Expression::BoolLiteral(value) => Value::Bool(*value),
            Expression::StringLiteral(value) => Value::String(value.clone()),
            Expression::RawStringLiteral(value) => Value::String(value.clone()), // 原始字符串字面量
            Expression::LongLiteral(value) => Value::Long(*value),
            Expression::StringInterpolation(segments) => {
                // 计算字符串插值
                let mut result = String::new();
                
                for segment in segments {
                    match segment {
                        crate::ast::StringInterpolationSegment::Text(text) => {
                            result.push_str(text);
                        },
                        crate::ast::StringInterpolationSegment::Expression(expr) => {
                            // 计算表达式并转换为字符串
                            let value = self.evaluate_expression(expr);
                            result.push_str(&value.to_string());
                        }
                    }
                }
                
                Value::String(result)
            },
            Expression::ArrayLiteral(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate_expression(elem));
                }
                Value::Array(values)
            },
            Expression::ArrayAccess(array_expr, index_expr) => {
                // 🧮 数组访问JIT编译检查
                let array_key = format!("array_access_{:p}", expr as *const _);
                if jit::should_compile_array_operation(&array_key) {
                    if let Ok(_compiled) = jit::compile_array_operation(expr, array_key.clone(), false) {
                        println!("✅ 数组访问JIT编译成功: {}", array_key);
                    }
                }

                let array_value = self.evaluate_expression(array_expr);
                let index_value = self.evaluate_expression(index_expr);

                match (array_value, index_value) {
                    (Value::Array(arr), Value::Int(index)) => {
                        if index < 0 || index as usize >= arr.len() {
                            panic!("数组索引越界: 索引 {} 超出数组长度 {}", index, arr.len());
                        }
                        arr[index as usize].clone()
                    },
                    (Value::Array(_), _) => {
                        panic!("数组索引必须是整数类型");
                    },
                    _ => {
                        panic!("只能对数组进行索引访问");
                    }
                }
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
            Expression::FunctionPointerCall(func_expr, args) => {
                let func_value = self.evaluate_expression(func_expr);
                let arg_values: Vec<Value> = args.iter().map(|arg| self.evaluate_expression(arg)).collect();

                match func_value {
                    Value::FunctionPointer(func_ptr) => {
                        self.call_function_pointer_impl(&func_ptr, arg_values)
                    },
                    Value::LambdaFunctionPointer(lambda_ptr) => {
                        self.call_lambda_function_pointer_impl(&lambda_ptr, arg_values)
                    },
                    _ => {
                        panic!("只能调用函数指针或Lambda函数指针");
                    }
                }
            },
            Expression::GlobalFunctionCall(name, args) => {
                self.handle_global_function_call(name, args)
            },
            Expression::NamespacedFunctionCall(path, args) => {
                self.handle_namespaced_function_call(path, args)
            },
            Expression::Variable(name) => {
                // v0.7.4新增：生命周期优化的变量访问
                if self.can_skip_runtime_check(name) {
                    // 对于安全变量，跳过边界检查，直接访问
                    return self.get_variable_optimized(name);
                }

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

                // 检查是否是函数名，如果是则创建函数指针
                if self.functions.contains_key(name) {
                    return self.create_function_pointer(name);
                }

                // 如果都找不到，返回None
                Value::None
            },
            Expression::BinaryOp(left, op, right) => {
                // 尝试数学表达式JIT编译优化
                if self.should_try_math_jit_optimization(expr) {
                    if let Some(result) = self.try_math_jit_expression(expr) {
                        return result;
                    }
                }

                // 尝试通用JIT编译优化
                if self.should_try_jit_optimization(expr) {
                    if let Some(result) = self.try_jit_expression(expr) {
                        return result;
                    }
                }

                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);

                // 内联简单的整数运算，避免函数调用开销
                match (&left_val, op, &right_val) {
                    (Value::Int(l), BinaryOperator::Add, Value::Int(r)) => Value::Int(l + r),
                    (Value::Int(l), BinaryOperator::Subtract, Value::Int(r)) => Value::Int(l - r),
                    (Value::Int(l), BinaryOperator::Multiply, Value::Int(r)) => Value::Int(l * r),
                    (Value::Int(l), BinaryOperator::Divide, Value::Int(r)) => {
                        if *r == 0 { panic!("除以零错误"); }
                        Value::Int(l / r)
                    },
                    (Value::Int(l), BinaryOperator::Modulo, Value::Int(r)) => {
                        if *r == 0 { panic!("除以零错误"); }
                        Value::Int(l % r)
                    },
                    // v0.7.2新增：内联位运算优化
                    (Value::Int(l), BinaryOperator::BitwiseAnd, Value::Int(r)) => Value::Int(l & r),
                    (Value::Int(l), BinaryOperator::BitwiseOr, Value::Int(r)) => Value::Int(l | r),
                    (Value::Int(l), BinaryOperator::BitwiseXor, Value::Int(r)) => Value::Int(l ^ r),
                    (Value::Int(l), BinaryOperator::LeftShift, Value::Int(r)) => {
                        if *r < 0 || *r >= 32 { panic!("移位操作数超出范围: {}", r); }
                        Value::Int(l << r)
                    },
                    (Value::Int(l), BinaryOperator::RightShift, Value::Int(r)) => {
                        if *r < 0 || *r >= 32 { panic!("移位操作数超出范围: {}", r); }
                        Value::Int(l >> r)
                    },
                    // 对于复杂运算，回退到原有实现
                    _ => self.perform_binary_operation(&left_val, op, &right_val)
                }
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
            Expression::Super => {
                // TODO: 实现super关键字，需要当前类上下文
                Value::None
            },
            Expression::StaticAccess(class_name, member_name) => {
                // 简化的静态访问实现
                if let Some(static_members) = self.static_members.get(class_name) {
                    if let Some(value) = static_members.static_fields.get(member_name) {
                        value.clone()
                    } else {
                        eprintln!("静态成员 {}::{} 不存在", class_name, member_name);
                        Value::None
                    }
                } else {
                    eprintln!("类 {} 不存在", class_name);
                    Value::None
                }
            },
            Expression::StaticMethodCall(class_name, method_name, args) => {
                // 🔧 首先检查是否是库命名空间函数调用
                if self.library_namespaces.contains_key(class_name) {
                    debug_println(&format!("StaticMethodCall被识别为库命名空间函数调用: {}::{}", class_name, method_name));
                    // 转换为命名空间函数调用
                    let path = vec![class_name.clone(), method_name.clone()];
                    return self.handle_namespaced_function_call(&path, args);
                }

                // 🔧 新增：检查是否是代码命名空间函数调用
                let potential_ns_path = format!("{}::{}", class_name, method_name);
                if self.namespaced_functions.contains_key(&potential_ns_path) {
                    debug_println(&format!("StaticMethodCall被识别为代码命名空间函数调用: {}", potential_ns_path));
                    // 转换为命名空间函数调用
                    let path = vec![class_name.clone(), method_name.clone()];
                    return self.handle_namespaced_function_call(&path, args);
                }
                
                // 简化的静态方法调用实现
                if let Some(class) = self.classes.get(class_name) {
                    if let Some(method) = class.methods.iter().find(|m| m.is_static && m.name == *method_name) {
                        // 计算参数
                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.evaluate_expression(arg));
                        }
                        
                        // 创建简单的参数环境
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
                                if let Some(crate::ast::Expression::Variable(var_name)) = expr {
                                    if let Some(value) = method_env.get(var_name) {
                                        return value.clone();
                                    }
                                } else if let Some(crate::ast::Expression::BinaryOp(left, op, right)) = expr {
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
                                    }
                                }
                                if let Some(expr) = expr {
                        return self.evaluate_expression(expr);
                    } else {
                        return Value::None;
                    }
                            }
                        }
                        Value::None
                    } else {
                        eprintln!("错误: 类 '{}' 没有静态方法 '{}'", class_name, method_name);
                        Value::None
                    }
                } else {
                    eprintln!("错误: 未找到类 '{}'", class_name);
                    Value::None
                }
            },
            // Lambda表达式和函数式编程
            Expression::Lambda(params, body) => {
                // 创建Lambda函数指针
                self.create_lambda_expression_pointer(params, body)
            },
            Expression::LambdaBlock(params, statements) => {
                // 创建Lambda块函数指针
                self.create_lambda_block_pointer(params, statements)
            },
            Expression::FunctionValue(func_name) => {
                // 函数值引用
                Value::FunctionReference(func_name.clone())
            },
            Expression::Apply(func_expr, args) => {
                // 函数应用
                let func_value = self.evaluate_expression(func_expr);
                let arg_values: Vec<Value> = args.iter().map(|arg| self.evaluate_expression(arg)).collect();
                self.apply_function(func_value, arg_values)
            },
            Expression::ArrayMap(array_expr, lambda_expr) => {
                // 🧮 数组map操作JIT编译检查
                let map_key = format!("array_map_{:p}", expr as *const _);
                if jit::should_compile_array_operation(&map_key) {
                    if let Ok(_compiled) = jit::compile_array_operation(expr, map_key.clone(), false) {
                        println!("✅ 数组map操作JIT编译成功: {}", map_key);
                    }
                }

                // array.map(lambda)
                let array_value = self.evaluate_expression(array_expr);
                let lambda_value = self.evaluate_expression(lambda_expr);
                self.array_map(array_value, lambda_value)
            },
            Expression::ArrayFilter(array_expr, lambda_expr) => {
                // 🧮 数组filter操作JIT编译检查
                let filter_key = format!("array_filter_{:p}", expr as *const _);
                if jit::should_compile_array_operation(&filter_key) {
                    if let Ok(_compiled) = jit::compile_array_operation(expr, filter_key.clone(), false) {
                        println!("✅ 数组filter操作JIT编译成功: {}", filter_key);
                    }
                }

                // array.filter(lambda)
                let array_value = self.evaluate_expression(array_expr);
                let lambda_value = self.evaluate_expression(lambda_expr);
                self.array_filter(array_value, lambda_value)
            },
            Expression::ArrayReduce(array_expr, lambda_expr, initial_expr) => {
                // 🧮 数组reduce操作JIT编译检查
                let reduce_key = format!("array_reduce_{:p}", expr as *const _);
                if jit::should_compile_array_operation(&reduce_key) {
                    if let Ok(_compiled) = jit::compile_array_operation(expr, reduce_key.clone(), false) {
                        println!("✅ 数组reduce操作JIT编译成功: {}", reduce_key);
                    }
                }

                // array.reduce(lambda, initial)
                let array_value = self.evaluate_expression(array_expr);
                let lambda_value = self.evaluate_expression(lambda_expr);
                let initial_value = self.evaluate_expression(initial_expr);
                self.array_reduce(array_value, lambda_value, initial_value)
            },
            Expression::ArrayForEach(array_expr, lambda_expr) => {
                // 🧮 数组forEach操作JIT编译检查
                let foreach_key = format!("array_foreach_{:p}", expr as *const _);
                if jit::should_compile_array_operation(&foreach_key) {
                    if let Ok(_compiled) = jit::compile_array_operation(expr, foreach_key.clone(), false) {
                        println!("✅ 数组forEach操作JIT编译成功: {}", foreach_key);
                    }
                }

                // array.forEach(lambda)
                let array_value = self.evaluate_expression(array_expr);
                let lambda_value = self.evaluate_expression(lambda_expr);
                self.array_for_each(array_value, lambda_value);
                Value::None
            },
            // Enum 相关表达式
            Expression::EnumVariantCreation(enum_name, variant_name, args) => {
                self.create_enum_variant(enum_name, variant_name, args)
            },
            Expression::EnumVariantAccess(enum_name, variant_name) => {
                self.access_enum_variant(enum_name, variant_name)
            },
            // Pointer 相关表达式
            Expression::AddressOf(expr) => {
                match self.create_pointer_safe(expr) {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("指针创建错误: {}", e);
                        Value::None
                    }
                }
            },
            Expression::Dereference(expr) => {
                match self.dereference_pointer_safe(expr) {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("指针解引用错误: {}", e);
                        Value::None
                    }
                }
            },
            Expression::PointerArithmetic(left, op, right) => {
                match self.evaluate_pointer_arithmetic_safe(left, op, right) {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("指针算术错误: {}", e);
                        Value::None
                    }
                }
            },
            Expression::PointerMemberAccess(ptr_expr, member_name) => {
                match self.evaluate_pointer_member_access_safe(ptr_expr, member_name) {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("指针成员访问错误: {}", e);
                        Value::None
                    }
                }
            },
            Expression::ArrayPointerAccess(array_ptr_expr, index_expr) => {
                match self.evaluate_array_pointer_access_safe(array_ptr_expr, index_expr) {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("数组指针访问错误: {}", e);
                        Value::None
                    }
                }
            },
            Expression::PointerArrayAccess(ptr_array_expr, index_expr) => {
                match self.evaluate_pointer_array_access_safe(ptr_array_expr, index_expr) {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("指针数组访问错误: {}", e);
                        Value::None
                    }
                }
            },
            Expression::FunctionPointerCall(func_expr, args) => {
                self.call_function_pointer(func_expr, args)
            },
            Expression::FunctionReference(func_name) => {
                self.create_function_pointer(func_name)
            },
            Expression::LambdaFunction(params, return_type, body) => {
                self.create_lambda_function_pointer(params, return_type, body)
            },
            Expression::None => {
                Value::None
            },
            Expression::SwitchExpression(switch_expr, cases, default_expr) => {
                let switch_value = self.evaluate_expression(switch_expr);
                for case in cases {
                    if let CasePattern::Value(case_expr) = &case.pattern {
                        let case_value = self.evaluate_expression(case_expr);
                        if match (&switch_value, &case_value) {
                            (Value::Int(a), Value::Int(b)) => a == b,
                            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
                            (Value::Bool(a), Value::Bool(b)) => a == b,
                            (Value::String(a), Value::String(b)) => a == b,
                            (Value::Long(a), Value::Long(b)) => a == b,
                            _ => false,
                        } {
                            if let Some(expr) = &case.expression {
                                return self.evaluate_expression(expr);
                            }
                            return Value::None;
                        }
                    }
                }
                if let Some(default_expr_box) = default_expr {
                    self.evaluate_expression(default_expr_box)
                } else {
                    Value::None
                }
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
                // 对于否定操作，实际的表达式在右操作数位置
                let val = self.evaluate_expression(right);
                match val {
                    Value::Bool(b) => Value::Bool(!b),
                    _ => panic!("逻辑否定操作符的操作数必须是布尔类型"),
                }
            },
        }
    }
    
    // 收集类的所有字段（包括继承的）
    fn collect_all_fields(&self, class: &crate::ast::Class) -> Vec<crate::ast::Field> {
        let mut all_fields = Vec::new();
        
        // 递归收集父类字段
        if let Some(ref super_class_name) = class.super_class {
            if let Some(super_class) = self.classes.get(super_class_name) {
                let parent_fields = self.collect_all_fields(super_class);
                all_fields.extend(parent_fields);
            }
        }
        
        // 添加当前类的字段
        all_fields.extend(class.fields.clone());
        
        all_fields
    }
    
    // 查找方法（支持继承）
    fn find_method(&self, class_name: &str, method_name: &str) -> Option<(&crate::ast::Class, &crate::ast::Method)> {
        if let Some(class) = self.classes.get(class_name) {
            // 首先在当前类中查找
            for method in &class.methods {
                if method.name == method_name && !method.is_static {
                    return Some((class, method));
                }
            }
            
            // 如果没找到，在父类中查找
            if let Some(ref super_class_name) = class.super_class {
                return self.find_method(super_class_name, method_name);
            }
        }
        None
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
                // 🧮 数组方法调用JIT编译检查
                let method_key = format!("array_method_{}_{:p}", method_name, obj_expr as *const _);
                if jit::should_compile_array_operation(&method_key) {
                    if let Ok(_compiled) = jit::compile_array_operation(obj_expr, method_key.clone(), false) {
                        println!("✅ 数组方法{}JIT编译成功: {}", method_name, method_key);
                    }
                }

                // 数组方法调用
                self.handle_array_method(&arr, method_name, &evaluated_args)
            },
            Value::Map(map) => {
                // 映射方法调用
                self.handle_map_method(&map, method_name, &evaluated_args)
            },
            Value::Object(_) => {
                // 对象方法调用
                self.call_method(obj_expr, method_name, args)
            },
            Value::EnumValue(enum_val) => {
                // 枚举值方法调用
                self.handle_enum_method(&enum_val, method_name, &evaluated_args)
            },
            Value::Pointer(ptr) => {
                // 指针值方法调用
                self.handle_pointer_method(&ptr, method_name, &evaluated_args)
            },
            Value::FunctionPointer(func_ptr) => {
                // 函数指针方法调用
                self.handle_function_pointer_method(&func_ptr, method_name, &evaluated_args)
            },
            Value::LambdaFunctionPointer(lambda_ptr) => {
                // Lambda函数指针方法调用
                self.handle_lambda_function_pointer_method(&lambda_ptr, method_name, &evaluated_args)
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
            "startsWith" => {
                if args.len() == 1 {
                    Value::Bool(s.starts_with(&args[0]))
                } else {
                    panic!("startsWith方法需要一个参数")
                }
            },
            "endsWith" => {
                if args.len() == 1 {
                    Value::Bool(s.ends_with(&args[0]))
                } else {
                    panic!("endsWith方法需要一个参数")
                }
            },
            "contains" => {
                if args.len() == 1 {
                    Value::Bool(s.contains(&args[0]))
                } else {
                    panic!("contains方法需要一个参数")
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
            Expression::FunctionPointerCall(func_expr, args) => {
                self.contains_method_call(func_expr) || args.iter().any(|arg| self.contains_method_call(arg))
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
            Expression::ArrayAccess(array_expr, index_expr) => {
                self.contains_method_call(array_expr) || self.contains_method_call(index_expr)
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
        
        // 检查是否为抽象类
        if class.is_abstract {
            eprintln!("错误: 不能实例化抽象类 '{}'", class_name);
            return Value::None;
        }
        
        // 计算构造函数参数
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expression(arg));
        }
        
        // 创建对象实例，包含继承的字段
        let mut fields = HashMap::new();
        
        // 收集所有字段（包括继承的）
        let all_fields = self.collect_all_fields(class);
        
        // 初始化字段为默认值
        for field in &all_fields {
            if !field.is_static { // 只初始化非静态字段
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
        }
        
        // 调用构造函数
        if let Some(constructor) = class.constructors.first() {
            // 创建临时的this上下文
            let mut this_context = ObjectInstance {
                class_name: class_name.to_string(),
                fields: fields.clone(),
            };
            
            // 创建构造函数参数环境
            let mut constructor_env = HashMap::new();
            for (i, param) in constructor.parameters.iter().enumerate() {
                if i < arg_values.len() {
                    constructor_env.insert(param.name.clone(), arg_values[i].clone());
                }
            }
            
            // 执行构造函数体
            for statement in &constructor.body {
                self.execute_constructor_statement(statement, &mut this_context, &constructor_env);
            }
            
            // 使用构造函数执行后的字段
            Value::Object(this_context)
        } else {
            // 没有构造函数，使用默认字段
            let object = ObjectInstance {
                class_name: class_name.to_string(),
                fields,
            };
            Value::Object(object)
        }
    }
    
    fn access_field(&mut self, obj_expr: &Expression, field_name: &str) -> Value {
        let obj_value = self.evaluate_expression(obj_expr);

        match obj_value {
            Value::Object(obj) => {
                // 检查字段访问权限
                if let Some(class) = self.classes.get(&obj.class_name) {
                    // 查找字段定义
                    if let Some(field) = class.fields.iter().find(|f| f.name == field_name) {
                        // 检查访问权限
                        match field.visibility {
                            crate::ast::Visibility::Private => {
                                // 私有字段只能在同一个类内部访问
                                // 这里简化处理：如果是this访问则允许，否则拒绝
                                if let Expression::This = *obj_expr {
                                    // this.field 访问，允许
                                } else {
                                    eprintln!("错误: 字段 '{}' 是私有的，无法从外部访问", field_name);
                                    return Value::None;
                                }
                            },
                            crate::ast::Visibility::Protected => {
                                // 保护字段可以在同一个类或子类中访问
                                // 这里简化处理：暂时允许访问
                                // TODO: 实现完整的继承检查
                            },
                            crate::ast::Visibility::Public => {
                                // 公共字段可以自由访问
                            }
                        }
                    }
                }

                // 执行实际的字段访问
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
    
    fn execute_constructor_statement(&mut self, statement: &crate::ast::Statement, this_obj: &mut ObjectInstance, constructor_env: &HashMap<String, Value>) {
        use crate::ast::Statement;

        match statement {
            Statement::FieldAssignment(obj_expr, field_name, value_expr) => {
                // 检查是否是this.field = value
                if let crate::ast::Expression::This = **obj_expr {
                    let value = self.evaluate_expression_with_constructor_context(value_expr, this_obj, constructor_env);
                    this_obj.fields.insert(field_name.clone(), value);
                }
            },
            _ => {
                // 其他语句暂时跳过
            }
        }
    }
    
    fn evaluate_expression_with_constructor_context(&mut self, expr: &Expression, this_obj: &ObjectInstance, constructor_env: &HashMap<String, Value>) -> Value {
        match expr {
            Expression::Variable(var_name) => {
                // 首先检查构造函数参数
                if let Some(value) = constructor_env.get(var_name) {
                    return value.clone();
                }
                // 然后检查常量
                if let Some(value) = self.constants.get(var_name) {
                    return value.clone();
                }
                // 最后检查全局变量
                if let Some(value) = self.global_env.get(var_name) {
                    return value.clone();
                }
                // 如果都没找到，返回None
                Value::None
            },
            Expression::BinaryOp(left, op, right) => {
                // 递归处理二元操作，保持构造函数上下文
                let left_val = self.evaluate_expression_with_constructor_context(left, this_obj, constructor_env);
                let right_val = self.evaluate_expression_with_constructor_context(right, this_obj, constructor_env);

                match op {
                    crate::ast::BinaryOperator::Add => {
                        match (&left_val, &right_val) {
                            (Value::String(s1), Value::String(s2)) => Value::String(s1.clone() + s2),
                            (Value::String(s), Value::Int(i)) => Value::String(s.clone() + &i.to_string()),
                            (Value::String(s), Value::Float(f)) => Value::String(s.clone() + &f.to_string()),
                            (Value::Int(i), Value::String(s)) => Value::String(i.to_string() + s),
                            (Value::Float(f), Value::String(s)) => Value::String(f.to_string() + s),
                            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1 + i2),
                            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 + f2),
                            (Value::Int(i), Value::Float(f)) => Value::Float(*i as f64 + f),
                            (Value::Float(f), Value::Int(i)) => Value::Float(f + *i as f64),
                            // 处理None值的字符串拼接
                            (Value::String(s), Value::None) => {
                                eprintln!("警告: 字符串拼接中遇到None值");
                                Value::String(s.clone() + "null")
                            },
                            (Value::None, Value::String(s)) => {
                                eprintln!("警告: 字符串拼接中遇到None值");
                                Value::String("null".to_string() + s)
                            },
                            _ => {
                                eprintln!("不支持的二元操作: {:?} Add {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    crate::ast::BinaryOperator::Multiply => {
                        match (&left_val, &right_val) {
                            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1 * i2),
                            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 * f2),
                            (Value::Int(i), Value::Float(f)) => Value::Float(*i as f64 * f),
                            (Value::Float(f), Value::Int(i)) => Value::Float(f * *i as f64),
                            _ => {
                                eprintln!("不支持的乘法操作: {:?} Multiply {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    crate::ast::BinaryOperator::Subtract => {
                        match (&left_val, &right_val) {
                            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1 - i2),
                            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 - f2),
                            (Value::Int(i), Value::Float(f)) => Value::Float(*i as f64 - f),
                            (Value::Float(f), Value::Int(i)) => Value::Float(f - *i as f64),
                            _ => {
                                eprintln!("不支持的减法操作: {:?} Subtract {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    crate::ast::BinaryOperator::Divide => {
                        match (&left_val, &right_val) {
                            (Value::Int(i1), Value::Int(i2)) => {
                                if *i2 == 0 {
                                    eprintln!("错误: 除零");
                                    Value::None
                                } else {
                                    Value::Int(i1 / i2)
                                }
                            },
                            (Value::Float(f1), Value::Float(f2)) => {
                                if *f2 == 0.0 {
                                    eprintln!("错误: 除零");
                                    Value::None
                                } else {
                                    Value::Float(f1 / f2)
                                }
                            },
                            (Value::Int(i), Value::Float(f)) => {
                                if *f == 0.0 {
                                    eprintln!("错误: 除零");
                                    Value::None
                                } else {
                                    Value::Float(*i as f64 / f)
                                }
                            },
                            (Value::Float(f), Value::Int(i)) => {
                                if *i == 0 {
                                    eprintln!("错误: 除零");
                                    Value::None
                                } else {
                                    Value::Float(f / *i as f64)
                                }
                            },
                            _ => {
                                eprintln!("不支持的除法操作: {:?} Divide {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    // v0.7.2新增：位运算操作符支持
                    crate::ast::BinaryOperator::BitwiseAnd => {
                        match (&left_val, &right_val) {
                            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1 & i2),
                            (Value::Long(l1), Value::Long(l2)) => Value::Long(l1 & l2),
                            (Value::Int(i), Value::Long(l)) => Value::Long((*i as i64) & l),
                            (Value::Long(l), Value::Int(i)) => Value::Long(l & (*i as i64)),
                            _ => {
                                eprintln!("不支持的按位与操作: {:?} BitwiseAnd {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    crate::ast::BinaryOperator::BitwiseOr => {
                        match (&left_val, &right_val) {
                            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1 | i2),
                            (Value::Long(l1), Value::Long(l2)) => Value::Long(l1 | l2),
                            (Value::Int(i), Value::Long(l)) => Value::Long((*i as i64) | l),
                            (Value::Long(l), Value::Int(i)) => Value::Long(l | (*i as i64)),
                            _ => {
                                eprintln!("不支持的按位或操作: {:?} BitwiseOr {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    crate::ast::BinaryOperator::BitwiseXor => {
                        match (&left_val, &right_val) {
                            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1 ^ i2),
                            (Value::Long(l1), Value::Long(l2)) => Value::Long(l1 ^ l2),
                            (Value::Int(i), Value::Long(l)) => Value::Long((*i as i64) ^ l),
                            (Value::Long(l), Value::Int(i)) => Value::Long(l ^ (*i as i64)),
                            _ => {
                                eprintln!("不支持的按位异或操作: {:?} BitwiseXor {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    crate::ast::BinaryOperator::LeftShift => {
                        match (&left_val, &right_val) {
                            (Value::Int(i1), Value::Int(i2)) => {
                                if *i2 < 0 || *i2 >= 32 {
                                    eprintln!("错误: 移位操作数超出范围: {}", i2);
                                    Value::None
                                } else {
                                    Value::Int(i1 << i2)
                                }
                            },
                            (Value::Long(l), Value::Int(i)) => {
                                if *i < 0 || *i >= 64 {
                                    eprintln!("错误: 移位操作数超出范围: {}", i);
                                    Value::None
                                } else {
                                    Value::Long(l << i)
                                }
                            },
                            _ => {
                                eprintln!("不支持的左移操作: {:?} LeftShift {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    crate::ast::BinaryOperator::RightShift => {
                        match (&left_val, &right_val) {
                            (Value::Int(i1), Value::Int(i2)) => {
                                if *i2 < 0 || *i2 >= 32 {
                                    eprintln!("错误: 移位操作数超出范围: {}", i2);
                                    Value::None
                                } else {
                                    Value::Int(i1 >> i2)
                                }
                            },
                            (Value::Long(l), Value::Int(i)) => {
                                if *i < 0 || *i >= 64 {
                                    eprintln!("错误: 移位操作数超出范围: {}", i);
                                    Value::None
                                } else {
                                    Value::Long(l >> i)
                                }
                            },
                            _ => {
                                eprintln!("不支持的右移操作: {:?} RightShift {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    _ => {
                        // 其他操作暂时返回None
                        eprintln!("构造函数上下文中不支持的二元操作: {:?}", op);
                        Value::None
                    }
                }
            },
            Expression::This => Value::Object(this_obj.clone()),
            _ => self.evaluate_expression(expr),
        }
    }
    
    fn call_method(&mut self, obj_expr: &Expression, method_name: &str, args: &[Expression]) -> Value {
        let obj_value = self.evaluate_expression(obj_expr);

        match obj_value {
            Value::Object(obj) => {
                // 使用继承支持的方法查找，克隆方法以避免借用冲突
                let (class, method) = match self.find_method(&obj.class_name, method_name) {
                    Some((class, method)) => (class, method),
                    None => {
                        eprintln!("错误: 类 '{}' 没有方法 '{}'", obj.class_name, method_name);
                        return Value::None;
                    }
                };

                // 检查方法访问权限
                match method.visibility {
                    crate::ast::Visibility::Private => {
                        // 私有方法只能在同一个类内部调用
                        // 这里简化处理：如果是this调用则允许，否则拒绝
                        if let Expression::This = *obj_expr {
                            // this.method() 调用，允许
                        } else {
                            eprintln!("错误: 方法 '{}' 是私有的，无法从外部调用", method_name);
                            return Value::None;
                        }
                    },
                    crate::ast::Visibility::Protected => {
                        // 保护方法可以在同一个类或子类中调用
                        // 这里简化处理：暂时允许调用
                        // TODO: 实现完整的继承检查
                    },
                    crate::ast::Visibility::Public => {
                        // 公共方法可以自由调用
                    }
                }

                let method_clone = method.clone();

                // 检查抽象方法
                if method_clone.is_abstract {
                    eprintln!("错误: 不能调用抽象方法 '{}'", method_name);
                    return Value::None;
                }

                // 计算参数
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg));
                }

                // 创建方法参数环境
                let mut method_env = HashMap::new();
                for (i, param) in method_clone.parameters.iter().enumerate() {
                    if i < arg_values.len() {
                        method_env.insert(param.name.clone(), arg_values[i].clone());
                    }
                }

                // 执行方法体，传递this对象和参数环境
                let (result, updated_obj) = self.execute_method_body_with_context(&method_clone.body, &obj, &method_env);

                // 更新原始对象的状态
                match obj_expr {
                    Expression::Variable(var_name) => {
                        // 更新变量环境中的对象
                        if let Some(_) = self.local_env.get(var_name) {
                            self.local_env.insert(var_name.clone(), Value::Object(updated_obj));
                        } else {
                            self.global_env.insert(var_name.clone(), Value::Object(updated_obj));
                        }
                    },
                    _ => {
                        // 其他情况暂时不处理
                    }
                }

                result
            },
            _ => {
                eprintln!("错误: 尝试在非对象上调用方法");
                Value::None
            }
        }
    }
    
    fn execute_method_body_with_context(&mut self, statements: &[crate::ast::Statement], this_obj: &ObjectInstance, method_env: &HashMap<String, Value>) -> (Value, ObjectInstance) {
        use crate::ast::Statement;

        // 创建一个可变的this对象副本，用于跟踪字段变化
        let mut current_this = this_obj.clone();

        // 保存当前的局部环境
        let old_local_env = self.local_env.clone();

        // 设置方法参数环境
        self.local_env.extend(method_env.clone());

        for statement in statements {
            match statement {
                Statement::Return(expr) => {
                    // 在方法执行期间，需要设置this上下文和参数环境
                    if let Some(expr) = expr {
                        let result = self.evaluate_expression_with_method_context(expr, &current_this, method_env);
                        // 恢复环境
                        self.local_env = old_local_env;
                        return (result, current_this);
                    } else {
                        // 恢复环境
                        self.local_env = old_local_env;
                        return (Value::None, current_this);
                    }
                },
                Statement::FieldAssignment(obj_expr, field_name, value_expr) => {
                    // 处理字段赋值
                    if let crate::ast::Expression::This = **obj_expr {
                        // this.field = value
                        let new_value = self.evaluate_expression_with_method_context(value_expr, &current_this, method_env);
                        current_this.fields.insert(field_name.clone(), new_value);
                    }
                },
                Statement::VariableDeclaration(var_name, _, init_expr) => {
                    // 处理局部变量声明
                    let value = self.evaluate_expression_with_method_context(init_expr, &current_this, method_env);
                    self.local_env.insert(var_name.clone(), value);
                },
                Statement::FunctionCallStatement(expr) => {
                    // 处理函数调用语句
                    self.evaluate_expression_with_method_context(expr, &current_this, method_env);
                },
                _ => {
                    // 其他语句类型可能需要进一步处理
                    // 暂时跳过，但记录警告
                    eprintln!("警告: 方法体中的语句类型暂未完全支持: {:?}", statement);
                }
            }
        }

        // 恢复环境
        self.local_env = old_local_env;

        (Value::None, current_this)
    }
    
    fn evaluate_expression_with_method_context(&mut self, expr: &Expression, this_obj: &ObjectInstance, method_env: &HashMap<String, Value>) -> Value {
        match expr {
            Expression::This => Value::Object(this_obj.clone()),
            Expression::FieldAccess(obj_expr, field_name) => {
                if let Expression::This = **obj_expr {
                    // this.field 访问 - 直接从this_obj获取
                    match this_obj.fields.get(field_name) {
                        Some(value) => value.clone(),
                        None => Value::None
                    }
                } else {
                    // 递归处理其他字段访问
                    let obj_value = self.evaluate_expression_with_method_context(obj_expr, this_obj, method_env);
                    match obj_value {
                        Value::Object(obj) => {
                            match obj.fields.get(field_name) {
                                Some(value) => value.clone(),
                                None => Value::None
                            }
                        },
                        _ => Value::None
                    }
                }
            },
            Expression::BinaryOp(left, op, right) => {
                // 处理二元操作，确保this上下文传递
                let left_val = self.evaluate_expression_with_method_context(left, this_obj, method_env);
                let right_val = self.evaluate_expression_with_method_context(right, this_obj, method_env);
                // 使用现有的二元操作评估方法
                self.perform_binary_operation(&left_val, op, &right_val)
            },
            Expression::Variable(var_name) => {
                // 特殊处理this关键字
                if var_name == "this" {
                    return Value::Object(this_obj.clone());
                }
                // 首先检查方法参数
                if let Some(value) = method_env.get(var_name) {
                    return value.clone();
                }
                // 然后检查局部变量
                if let Some(value) = self.local_env.get(var_name) {
                    return value.clone();
                }
                // 然后检查常量
                if let Some(value) = self.constants.get(var_name) {
                    return value.clone();
                }
                // 最后检查全局变量
                if let Some(value) = self.global_env.get(var_name) {
                    return value.clone();
                }
                Value::None
            },
            Expression::StaticMethodCall(class_name, method_name, args) => {
                // 在方法上下文中处理StaticMethodCall

                // 检查是否是库命名空间函数调用
                if self.library_namespaces.contains_key(class_name) {
                    // 在方法上下文中计算参数
                    let mut arg_values = Vec::new();
                    for arg_expr in args {
                        arg_values.push(self.evaluate_expression_with_method_context(arg_expr, this_obj, method_env));
                    }

                    // 调用库函数
                    let string_args = super::library_loader::convert_values_to_string_args(&arg_values);
                    let full_path = format!("{}::{}", class_name, method_name);

                    if let Some(lib_name) = self.library_namespaces.get(class_name) {
                        match super::library_loader::call_library_function(lib_name, &full_path, string_args) {
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
                            Err(_) => return Value::None,
                        }
                    }
                }

                // 如果不是库函数，回退到普通处理
                self.evaluate_expression(expr)
            },
            Expression::MethodCall(obj_expr, method_name, args) => {
                // 在方法上下文中处理MethodCall

                if let Expression::This = **obj_expr {
                    // this.method() 调用 - 在方法上下文中计算参数
                    let mut arg_values = Vec::new();
                    for arg_expr in args {
                        arg_values.push(self.evaluate_expression_with_method_context(arg_expr, this_obj, method_env));
                    }

                    // 查找方法并调用
                    let (_class, method) = match self.find_method(&this_obj.class_name, method_name) {
                        Some(result) => result,
                        None => {
                            eprintln!("错误: 类 '{}' 中未找到方法 '{}'", this_obj.class_name, method_name);
                            return Value::None;
                        }
                    };

                    // 克隆方法以避免借用冲突
                    let method_clone = method.clone();

                    // 检查抽象方法
                    if method_clone.is_abstract {
                        eprintln!("错误: 不能调用抽象方法 '{}'", method_name);
                        return Value::None;
                    }

                    // 创建方法参数环境
                    let mut method_env_new = HashMap::new();
                    for (i, param) in method_clone.parameters.iter().enumerate() {
                        if i < arg_values.len() {
                            method_env_new.insert(param.name.clone(), arg_values[i].clone());
                        }
                    }

                    // 执行方法体，传递this对象和参数环境
                    let (result, _updated_obj) = self.execute_method_body_with_context(&method_clone.body, this_obj, &method_env_new);
                    return result;
                } else {
                    // 其他对象的方法调用，递归处理
                    let obj_value = self.evaluate_expression_with_method_context(obj_expr, this_obj, method_env);
                    match obj_value {
                        Value::Object(obj) => {
                            let mut arg_values = Vec::new();
                            for arg_expr in args {
                                arg_values.push(self.evaluate_expression_with_method_context(arg_expr, this_obj, method_env));
                            }

                            let (_class, method) = match self.find_method(&obj.class_name, method_name) {
                                Some(result) => result,
                                None => {
                                    eprintln!("错误: 类 '{}' 中未找到方法 '{}'", obj.class_name, method_name);
                                    return Value::None;
                                }
                            };

                            // 克隆方法以避免借用冲突
                            let method_clone = method.clone();

                            // 检查抽象方法
                            if method_clone.is_abstract {
                                eprintln!("错误: 不能调用抽象方法 '{}'", method_name);
                                return Value::None;
                            }

                            // 创建方法参数环境
                            let mut method_env_new = HashMap::new();
                            for (i, param) in method_clone.parameters.iter().enumerate() {
                                if i < arg_values.len() {
                                    method_env_new.insert(param.name.clone(), arg_values[i].clone());
                                }
                            }

                            // 执行方法体，传递this对象和参数环境
                            let (result, _updated_obj) = self.execute_method_body_with_context(&method_clone.body, &obj, &method_env_new);
                            return result;
                        },
                        _ => {
                            eprintln!("错误: 尝试在非对象上调用方法");
                            return Value::None;
                        }
                    }
                }
            },
            Expression::FieldAccess(obj_expr, field_name) => {
                // 在方法上下文中处理FieldAccess

                if let Expression::This = **obj_expr {
                    if let Some(value) = this_obj.fields.get(field_name) {
                        return value.clone();
                    } else {
                        return Value::None;
                    }
                } else {
                    // 其他对象的字段访问，递归处理
                    let obj_value = self.evaluate_expression_with_method_context(obj_expr, this_obj, method_env);
                    match obj_value {
                        Value::Object(obj) => {
                            eprintln!("调试: 访问{}.{}, 对象类型: {}", obj.class_name, field_name, obj.class_name);
                            eprintln!("调试: 对象字段: {:?}", obj.fields.keys().collect::<Vec<_>>());

                            if let Some(value) = obj.fields.get(field_name) {
                                eprintln!("调试: 找到字段 '{}', 值: {:?}", field_name, value);
                                return value.clone();
                            } else {
                                eprintln!("调试: 字段 '{}' 不存在", field_name);
                                return Value::None;
                            }
                        },
                        _ => {
                            eprintln!("调试: 尝试在非对象上访问字段");
                            return Value::None;
                        }
                    }
                }
            },
            _ => self.evaluate_expression(expr),
        }
    }

    fn evaluate_expression_with_full_context(&mut self, expr: &Expression, this_obj: &ObjectInstance, method_env: &HashMap<String, Value>) -> Value {
        match expr {
            Expression::This => Value::Object(this_obj.clone()),
            Expression::FieldAccess(obj_expr, field_name) => {
                if let Expression::This = **obj_expr {
                    // this.field 访问 - 直接从this_obj获取
                    match this_obj.fields.get(field_name) {
                        Some(value) => value.clone(),
                        None => {
                            eprintln!("错误: 对象 '{}' 没有字段 '{}'", this_obj.class_name, field_name);
                            // 列出所有可用字段用于调试
                            eprintln!("可用字段: {:?}", this_obj.fields.keys().collect::<Vec<_>>());
                            Value::None
                        }
                    }
                } else {
                    // 递归处理其他字段访问
                    let obj_value = self.evaluate_expression_with_full_context(obj_expr, this_obj, method_env);
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
                            eprintln!("错误: 尝试访问非对象的字段，对象值: {:?}", obj_value);
                            eprintln!("调试: obj_expr = {:?}", obj_expr);
                            Value::None
                        }
                    }
                }
            },
            Expression::BinaryOp(left, op, right) => {
                // 处理二元操作，确保this上下文传递
                let left_val = self.evaluate_expression_with_full_context(left, this_obj, method_env);
                let right_val = self.evaluate_expression_with_full_context(right, this_obj, method_env);
                // 使用现有的二元操作评估方法
                match op {
                    crate::ast::BinaryOperator::Add => {
                        match (&left_val, &right_val) {
                            (Value::String(s1), Value::String(s2)) => Value::String(s1.clone() + s2),
                            (Value::String(s), Value::Int(i)) => Value::String(s.clone() + &i.to_string()),
                            (Value::String(s), Value::Float(f)) => Value::String(s.clone() + &f.to_string()),
                            (Value::Int(i), Value::String(s)) => Value::String(i.to_string() + s),
                            (Value::Float(f), Value::String(s)) => Value::String(f.to_string() + s),
                            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1 + i2),
                            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 + f2),
                            (Value::Int(i), Value::Float(f)) => Value::Float(*i as f64 + f),
                            (Value::Float(f), Value::Int(i)) => Value::Float(f + *i as f64),
                            // 处理None值的字符串拼接
                            (Value::String(s), Value::None) => {
                                eprintln!("警告: 字符串拼接中遇到None值");
                                Value::String(s.clone() + "null")
                            },
                            (Value::None, Value::String(s)) => {
                                eprintln!("警告: 字符串拼接中遇到None值");
                                Value::String("null".to_string() + s)
                            },
                            _ => {
                                eprintln!("不支持的二元操作: {:?} Add {:?}", left_val, right_val);
                                Value::None
                            },
                        }
                    },
                    _ => Value::None, // 其他操作暂时返回None
                }
            },
            Expression::Variable(var_name) => {
                // 特殊处理this关键字
                if var_name == "this" {
                    return Value::Object(this_obj.clone());
                }
                // 首先检查方法参数
                if let Some(value) = method_env.get(var_name) {
                    return value.clone();
                }
                // 然后检查局部变量
                if let Some(value) = self.local_env.get(var_name) {
                    return value.clone();
                }
                // 然后检查常量
                if let Some(value) = self.constants.get(var_name) {
                    return value.clone();
                }
                // 最后检查全局变量
                if let Some(value) = self.global_env.get(var_name) {
                    return value.clone();
                }
                Value::None
            },
            _ => self.evaluate_expression(expr),
        }
    }
    
    // Lambda表达式和函数式编程的辅助方法
    fn apply_function(&mut self, func_value: Value, arg_values: Vec<Value>) -> Value {
        match func_value {
            Value::Lambda(params, body) => {
                // 创建Lambda执行环境
                let mut lambda_env = HashMap::new();
                for (i, param) in params.iter().enumerate() {
                    if i < arg_values.len() {
                        lambda_env.insert(param.name.clone(), arg_values[i].clone());
                    }
                }
                
                // 保存当前环境
                let old_local_env = self.local_env.clone();
                
                // 设置Lambda环境
                self.local_env.extend(lambda_env);
                
                // 执行Lambda体
                let result = self.evaluate_expression(&body);
                
                // 恢复环境
                self.local_env = old_local_env;
                
                result
            },
            Value::LambdaBlock(params, statements) => {
                // 创建Lambda块执行环境
                let mut lambda_env = HashMap::new();
                for (i, param) in params.iter().enumerate() {
                    if i < arg_values.len() {
                        lambda_env.insert(param.name.clone(), arg_values[i].clone());
                    }
                }
                
                // 保存当前环境
                let old_local_env = self.local_env.clone();
                
                // 设置Lambda环境
                self.local_env.extend(lambda_env);
                
                // 执行Lambda块
                let mut result = Value::None;
                for statement in &statements {
                    if let crate::ast::Statement::Return(expr) = statement {
                        if let Some(expr) = expr {
                            result = self.evaluate_expression(expr);
                        } else {
                            result = Value::None;
                        }
                        break;
                    }
                    // 这里需要执行其他语句，但为了简化暂时跳过
                }
                
                // 恢复环境
                self.local_env = old_local_env;
                
                result
            },
            Value::FunctionReference(func_name) => {
                // 调用已定义的函数
                if let Some(func) = self.functions.get(&func_name) {
                    let func_clone = func.clone();
                    let args_as_expressions: Vec<crate::ast::Expression> = arg_values.iter().map(|v| {
                        match v {
                            Value::Int(i) => crate::ast::Expression::IntLiteral(*i),
                            Value::Float(f) => crate::ast::Expression::FloatLiteral(*f),
                            Value::Bool(b) => crate::ast::Expression::BoolLiteral(*b),
                            Value::String(s) => crate::ast::Expression::StringLiteral(s.clone()),
                            Value::Long(l) => crate::ast::Expression::LongLiteral(*l),
                            _ => crate::ast::Expression::StringLiteral(v.to_string()),
                        }
                    }).collect();
                    
                    self.handle_function_call(&func_name, &args_as_expressions)
                } else {
                    eprintln!("错误: 未找到函数 '{}'", func_name);
                    Value::None
                }
            },
            _ => {
                eprintln!("错误: 尝试应用非函数值");
                Value::None
            }
        }
    }
    
    fn array_map(&mut self, array_value: Value, lambda_value: Value) -> Value {
        match array_value {
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    let mapped_value = self.apply_function(lambda_value.clone(), vec![item]);
                    result.push(mapped_value);
                }
                Value::Array(result)
            },
            _ => {
                eprintln!("错误: map操作只能应用于数组");
                Value::None
            }
        }
    }
    
    fn array_filter(&mut self, array_value: Value, lambda_value: Value) -> Value {
        match array_value {
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    let filter_result = self.apply_function(lambda_value.clone(), vec![item.clone()]);
                    if let Value::Bool(true) = filter_result {
                        result.push(item);
                    }
                }
                Value::Array(result)
            },
            _ => {
                eprintln!("错误: filter操作只能应用于数组");
                Value::None
            }
        }
    }
    
    fn array_reduce(&mut self, array_value: Value, lambda_value: Value, initial_value: Value) -> Value {
        match array_value {
            Value::Array(arr) => {
                let mut accumulator = initial_value;
                for item in arr {
                    accumulator = self.apply_function(lambda_value.clone(), vec![accumulator, item]);
                }
                accumulator
            },
            _ => {
                eprintln!("错误: reduce操作只能应用于数组");
                Value::None
            }
        }
    }
    
    fn array_for_each(&mut self, array_value: Value, lambda_value: Value) {
        match array_value {
            Value::Array(arr) => {
                for item in arr {
                    self.apply_function(lambda_value.clone(), vec![item]);
                }
            },
            _ => {
                eprintln!("错误: forEach操作只能应用于数组");
            }
        }
    }

    // Enum 相关方法
    fn create_enum_variant(&mut self, enum_name: &str, variant_name: &str, args: &[Expression]) -> Value {
        debug_println(&format!("创建枚举变体: {}::{}", enum_name, variant_name));

        // 检查枚举是否存在
        if let Some(enum_def) = self.enums.get(enum_name) {
            // 查找对应的变体
            for variant in &enum_def.variants {
                if variant.name == variant_name {
                    // 计算参数值
                    let mut field_values = Vec::new();
                    for arg in args {
                        let value = self.evaluate_expression(arg);
                        field_values.push(value);
                    }

                    // 检查参数数量是否匹配
                    if field_values.len() != variant.fields.len() {
                        eprintln!("错误: 枚举变体 {}::{} 期望 {} 个参数，但得到了 {} 个",
                                enum_name, variant_name, variant.fields.len(), field_values.len());
                        return Value::None;
                    }

                    debug_println(&format!("成功创建枚举变体: {}::{}({} 个字段)",
                                enum_name, variant_name, field_values.len()));

                    return Value::EnumValue(EnumInstance {
                        enum_name: enum_name.to_string(),
                        variant_name: variant_name.to_string(),
                        fields: field_values,
                    });
                }
            }

            eprintln!("错误: 枚举 {} 中不存在变体 {}", enum_name, variant_name);
            Value::None
        } else {
            eprintln!("错误: 未找到枚举定义: {}", enum_name);
            Value::None
        }
    }

    fn access_enum_variant(&self, enum_name: &str, variant_name: &str) -> Value {
        debug_println(&format!("访问枚举变体: {}::{}", enum_name, variant_name));

        // 检查枚举是否存在
        if let Some(enum_def) = self.enums.get(enum_name) {
            // 查找对应的变体
            for variant in &enum_def.variants {
                if variant.name == variant_name {
                    // 如果变体没有字段，直接返回枚举实例
                    if variant.fields.is_empty() {
                        debug_println(&format!("访问无参数枚举变体: {}::{}", enum_name, variant_name));
                        return Value::EnumValue(EnumInstance {
                            enum_name: enum_name.to_string(),
                            variant_name: variant_name.to_string(),
                            fields: Vec::new(),
                        });
                    } else {
                        // 有字段的变体需要通过函数调用创建
                        eprintln!("错误: 枚举变体 {}::{} 需要参数，请使用 {}::{}(...) 语法",
                                enum_name, variant_name, enum_name, variant_name);
                        return Value::None;
                    }
                }
            }

            eprintln!("错误: 枚举 {} 中不存在变体 {}", enum_name, variant_name);
            Value::None
        } else {
            eprintln!("错误: 未找到枚举定义: {}", enum_name);
            Value::None
        }
    }

    fn handle_enum_method(&self, enum_val: &super::value::EnumInstance, method_name: &str, args: &[String]) -> Value {
        match method_name {
            "toString" => {
                // 返回枚举值的字符串表示
                if enum_val.fields.is_empty() {
                    Value::String(format!("{}::{}", enum_val.enum_name, enum_val.variant_name))
                } else {
                    let field_strs: Vec<String> = enum_val.fields.iter().map(|f| f.to_string()).collect();
                    Value::String(format!("{}::{}({})", enum_val.enum_name, enum_val.variant_name, field_strs.join(", ")))
                }
            },
            "length" => {
                // 返回枚举字段的数量
                Value::Int(enum_val.fields.len() as i32)
            },
            "getVariantName" => {
                // 返回枚举变体名称
                Value::String(enum_val.variant_name.clone())
            },
            "getEnumName" => {
                // 返回枚举类型名称
                Value::String(enum_val.enum_name.clone())
            },
            _ => {
                panic!("枚举类型不支持方法: {}", method_name);
            }
        }
    }

    fn handle_pointer_method(&self, ptr: &super::value::PointerInstance, method_name: &str, args: &[String]) -> Value {
        match method_name {
            "toString" => {
                // 返回指针的字符串表示
                if ptr.is_null {
                    Value::String("null".to_string())
                } else {
                    let stars = "*".repeat(ptr.level);
                    Value::String(format!("{}0x{:x}", stars, ptr.address))
                }
            },
            "getAddress" => {
                // 返回指针地址
                Value::Long(ptr.address as i64)
            },
            "getLevel" => {
                // 返回指针级别
                Value::Int(ptr.level as i32)
            },
            "isNull" => {
                // 返回是否为空指针
                Value::Bool(ptr.is_null)
            },
            _ => {
                panic!("指针类型不支持方法: {}", method_name);
            }
        }
    }

    fn handle_function_pointer_method(&self, func_ptr: &FunctionPointerInstance, method_name: &str, args: &[String]) -> Value {
        match method_name {
            "toString" => {
                // 返回函数指针的字符串表示
                if func_ptr.is_null {
                    Value::String("null".to_string())
                } else if func_ptr.is_lambda {
                    Value::String("*fn(lambda)".to_string())
                } else {
                    Value::String(format!("*fn({})", func_ptr.function_name))
                }
            },
            "getName" => {
                // 返回函数名
                if func_ptr.is_lambda {
                    Value::String("lambda".to_string())
                } else {
                    Value::String(func_ptr.function_name.clone())
                }
            },
            "getParamCount" => {
                // 返回参数数量
                Value::Int(func_ptr.param_types.len() as i32)
            },
            "getReturnType" => {
                // 返回返回类型的字符串表示
                Value::String(Value::type_to_string(&func_ptr.return_type))
            },
            "isNull" => {
                // 返回是否为空
                Value::Bool(func_ptr.is_null)
            },
            "isLambda" => {
                // 返回是否为Lambda
                Value::Bool(func_ptr.is_lambda)
            },
            _ => {
                panic!("函数指针类型不支持方法: {}", method_name);
            }
        }
    }

    fn handle_lambda_function_pointer_method(&self, lambda_ptr: &LambdaFunctionPointerInstance, method_name: &str, args: &[String]) -> Value {
        match method_name {
            "toString" => {
                // 返回Lambda函数指针的字符串表示
                if lambda_ptr.is_null {
                    Value::String("null".to_string())
                } else {
                    let param_strs: Vec<String> = lambda_ptr.param_types.iter()
                        .map(|t| Value::type_to_string(t))
                        .collect();
                    Value::String(format!("*fn({}) : {}", param_strs.join(", "), Value::type_to_string(&lambda_ptr.return_type)))
                }
            },
            "getName" => {
                // 返回函数名
                Value::String("lambda".to_string())
            },
            "getParamCount" => {
                // 返回参数数量
                Value::Int(lambda_ptr.param_types.len() as i32)
            },
            "getReturnType" => {
                // 返回返回类型的字符串表示
                Value::String(Value::type_to_string(&lambda_ptr.return_type))
            },
            "isNull" => {
                // 返回是否为空
                Value::Bool(lambda_ptr.is_null)
            },
            "isLambda" => {
                // 返回是否为Lambda
                Value::Bool(true) // Lambda函数指针总是Lambda
            },
            "getParamNames" => {
                // 返回参数名列表（Lambda特有的方法）
                let param_names: Vec<String> = lambda_ptr.lambda_params.iter()
                    .map(|p| p.name.clone())
                    .collect();
                Value::String(format!("[{}]", param_names.join(", ")))
            },
            _ => {
                panic!("Lambda函数指针类型不支持方法: {}", method_name);
            }
        }
    }

    // 指针操作方法
    fn create_pointer(&mut self, expr: &Expression) -> Value {
        debug_println("创建指针");

        match expr {
            // 对变量取地址：直接获取变量的内存地址
            Expression::Variable(var_name) => {
                if let Some(existing_address) = self.get_variable_address(var_name) {
                    // 变量已有地址，直接返回指针
                    let target_value = self.get_variable_value(var_name).unwrap_or(Value::None);
                    let target_type = self.value_to_pointer_type(&target_value);

                    let pointer = PointerInstance {
                        address: existing_address,
                        target_type,
                        is_null: false,
                        level: 1,
                        tag_id: None, // 变量地址不需要标记
                    };

                    debug_println(&format!("获取变量地址: {} -> 0x{:x}", var_name, existing_address));
                    Value::Pointer(pointer)
                } else {
                    // 变量不存在，分配新地址
                    let target_value = self.evaluate_expression(expr);
                    self.allocate_and_create_pointer(target_value)
                }
            },
            // 对其他表达式取地址：需要分配临时内存
            _ => {
                let target_value = self.evaluate_expression(expr);
                self.allocate_and_create_pointer(target_value)
            }
        }
    }

    fn allocate_and_create_pointer(&mut self, target_value: Value) -> Value {
        match allocate_memory_smart(target_value.clone()) {
            Ok((address, tag_id)) => {
                let target_type = self.value_to_pointer_type(&target_value);
                let pointer = PointerInstance {
                    address,
                    target_type,
                    is_null: false,
                    level: 1,
                    tag_id: Some(tag_id),
                };

                debug_println(&format!("分配内存并创建指针，地址: 0x{:x}, 标记: {}", address, tag_id));
                Value::Pointer(pointer)
            },
            Err(e) => {
                panic!("内存分配失败: {}", e);
            }
        }
    }

    // 安全版本的指针创建
    fn create_pointer_safe(&mut self, expr: &Expression) -> Result<Value, PointerError> {
        debug_println("安全创建指针");

        match expr {
            // 对变量取地址：直接获取变量的内存地址
            Expression::Variable(var_name) => {
                if let Some(existing_address) = self.get_variable_address(var_name) {
                    // 变量已有地址，直接返回指针
                    let target_value = self.get_variable_value(var_name).unwrap_or(Value::None);
                    let target_type = self.value_to_pointer_type(&target_value);

                    let pointer = PointerInstance {
                        address: existing_address,
                        target_type,
                        is_null: false,
                        level: 1,
                        tag_id: None, // 变量地址不需要标记
                    };

                    debug_println(&format!("获取变量地址: {} -> 0x{:x}", var_name, existing_address));
                    Ok(Value::Pointer(pointer))
                } else {
                    // 变量不存在，分配新地址
                    let target_value = self.evaluate_expression(expr);
                    self.allocate_and_create_pointer_safe(target_value)
                }
            },
            // 对其他表达式取地址：需要分配临时内存
            _ => {
                let target_value = self.evaluate_expression(expr);
                self.allocate_and_create_pointer_safe(target_value)
            }
        }
    }

    fn allocate_and_create_pointer_safe(&mut self, target_value: Value) -> Result<Value, PointerError> {
        match allocate_memory_smart(target_value.clone()) {
            Ok((address, tag_id)) => {
                let target_type = self.value_to_pointer_type(&target_value);
                let pointer = PointerInstance {
                    address,
                    target_type,
                    is_null: false,
                    level: 1,
                    tag_id: Some(tag_id),
                };

                debug_println(&format!("安全分配内存并创建指针，地址: 0x{:x}, 标记: {}", address, tag_id));
                Ok(Value::Pointer(pointer))
            },
            Err(e) => {
                Err(PointerError::MemoryAllocationFailed(e))
            }
        }
    }

    // 获取变量的内存地址（如果已分配）
    fn get_variable_address(&self, var_name: &str) -> Option<usize> {
        // 这里需要实现变量地址映射
        // 暂时返回None，表示需要分配新地址
        None
    }

    // 获取变量的值
    fn get_variable_value(&self, var_name: &str) -> Option<Value> {
        self.local_env.get(var_name)
            .or_else(|| self.global_env.get(var_name))
            .cloned()
    }

    // 安全版本的指针解引用
    fn dereference_pointer_safe(&mut self, expr: &Expression) -> Result<Value, PointerError> {
        debug_println("安全解引用指针");

        // 计算指针表达式
        let pointer_value = self.evaluate_expression(expr);

        match pointer_value {
            Value::Pointer(ptr) => {
                if ptr.is_null {
                    return Err(PointerError::NullPointerAccess);
                }

                // 检查指针操作的有效性
                self.check_pointer_operation_validity(&ptr, "解引用")?;

                // 使用增强的安全检查
                let validation_result = if let Some(tag_id) = ptr.tag_id {
                    validate_pointer_safe(ptr.address, tag_id)
                } else {
                    validate_pointer(ptr.address)
                };

                if let Err(e) = validation_result {
                    return Err(PointerError::InvalidAddress(ptr.address));
                }

                // 检查悬空指针
                let is_dangling = if let Some(tag_id) = ptr.tag_id {
                    is_dangling_pointer(tag_id)
                } else {
                    is_dangling_pointer_by_address(ptr.address)
                };

                if is_dangling {
                    return Err(PointerError::DanglingPointerAccess(ptr.address));
                }

                // 安全读取内存
                let read_result = if let Some(tag_id) = ptr.tag_id {
                    read_memory_safe(ptr.address, tag_id)
                } else {
                    read_memory(ptr.address)
                };

                match read_result {
                    Ok(value) => {
                        debug_println(&format!("安全解引用指针，地址: 0x{:x}", ptr.address));

                        // 如果是多级指针，需要正确处理级别
                        if ptr.level > 1 {
                            match value {
                                Value::Pointer(mut inner_ptr) => {
                                    // 正确减少指针级别
                                    inner_ptr.level = ptr.level - 1;

                                    // 更新目标类型
                                    if let PointerType::Pointer(inner_type) = &ptr.target_type {
                                        inner_ptr.target_type = (**inner_type).clone();
                                    }

                                    Ok(Value::Pointer(inner_ptr))
                                },
                                _ => {
                                    Err(PointerError::InvalidPointerLevel)
                                }
                            }
                        } else {
                            Ok(value)
                        }
                    },
                    Err(e) => {
                        Err(PointerError::MemoryReadFailed(e))
                    }
                }
            },
            _ => {
                Err(PointerError::InvalidAddress(0)) // 非指针值
            }
        }
    }

    fn dereference_pointer(&mut self, expr: &Expression) -> Value {
        debug_println("解引用指针");

        // 计算指针表达式
        let pointer_value = self.evaluate_expression(expr);

        match pointer_value {
            Value::Pointer(ptr) => {
                if ptr.is_null {
                    panic!("尝试解引用空指针");
                }

                // 使用增强的安全检查
                let validation_result = if let Some(tag_id) = ptr.tag_id {
                    validate_pointer_safe(ptr.address, tag_id)
                } else {
                    validate_pointer(ptr.address)
                };

                if let Err(e) = validation_result {
                    panic!("指针验证失败: {}", e);
                }

                // 检查悬空指针
                let is_dangling = if let Some(tag_id) = ptr.tag_id {
                    is_dangling_pointer(tag_id)
                } else {
                    is_dangling_pointer_by_address(ptr.address)
                };

                if is_dangling {
                    panic!("尝试解引用悬空指针: 0x{:x}", ptr.address);
                }

                // 安全读取内存
                let read_result = if let Some(tag_id) = ptr.tag_id {
                    read_memory_safe(ptr.address, tag_id)
                } else {
                    read_memory(ptr.address)
                };

                match read_result {
                    Ok(value) => {
                        debug_println(&format!("解引用指针，地址: 0x{:x}", ptr.address));

                        // 如果是多级指针，需要正确处理级别
                        if ptr.level > 1 {
                            match value {
                                Value::Pointer(mut inner_ptr) => {
                                    // 正确减少指针级别
                                    inner_ptr.level = ptr.level - 1;

                                    // 更新目标类型
                                    if let PointerType::Pointer(inner_type) = &ptr.target_type {
                                        inner_ptr.target_type = (**inner_type).clone();
                                    }

                                    Value::Pointer(inner_ptr)
                                },
                                _ => {
                                    panic!("多级指针解引用错误：期望指针值，但得到: {:?}", value);
                                }
                            }
                        } else {
                            value
                        }
                    },
                    Err(e) => {
                        panic!("内存读取失败: {}", e);
                    }
                }
            },
            _ => {
                panic!("尝试解引用非指针值: {:?}", pointer_value);
            }
        }
    }

    // 安全版本的指针算术运算
    fn evaluate_pointer_arithmetic_safe(&mut self, left: &Expression, op: &crate::ast::PointerArithmeticOp, right: &Expression) -> Result<Value, PointerError> {
        debug_println("执行安全指针算术运算");

        let left_val = self.evaluate_expression(left);
        let right_val = self.evaluate_expression(right);

        match (&left_val, op, &right_val) {
            (Value::Pointer(ptr), crate::ast::PointerArithmeticOp::Add, Value::Int(offset)) => {
                // 检查指针操作的有效性
                self.check_pointer_operation_validity(ptr, "算术运算")?;

                let element_size = self.get_pointer_element_size(&ptr.target_type);

                // 使用安全的指针算术
                match safe_pointer_arithmetic(ptr.address, *offset as isize, element_size, ptr.tag_id) {
                    Ok(new_address) => {
                        let new_ptr = PointerInstance {
                            address: new_address,
                            target_type: ptr.target_type.clone(),
                            is_null: false,
                            level: ptr.level,
                            tag_id: None, // 算术结果不继承标记
                        };

                        debug_println(&format!("安全指针算术: 0x{:x} + {} = 0x{:x}", ptr.address, offset, new_address));
                        Ok(Value::Pointer(new_ptr))
                    },
                    Err(e) => {
                        if e.contains("溢出") {
                            Err(PointerError::PointerArithmeticOverflow)
                        } else if e.contains("下溢") {
                            Err(PointerError::PointerArithmeticUnderflow)
                        } else if e.contains("范围") {
                            Err(PointerError::AddressOutOfRange(ptr.address))
                        } else {
                            Err(PointerError::InvalidAddress(ptr.address))
                        }
                    }
                }
            },
            (Value::Pointer(ptr), crate::ast::PointerArithmeticOp::Sub, Value::Int(offset)) => {
                // 检查指针操作的有效性
                self.check_pointer_operation_validity(ptr, "算术运算")?;

                let element_size = self.get_pointer_element_size(&ptr.target_type);

                // 使用安全的指针算术
                match safe_pointer_arithmetic(ptr.address, -(*offset as isize), element_size, ptr.tag_id) {
                    Ok(new_address) => {
                        let new_ptr = PointerInstance {
                            address: new_address,
                            target_type: ptr.target_type.clone(),
                            is_null: false,
                            level: ptr.level,
                            tag_id: None, // 算术结果不继承标记
                        };

                        debug_println(&format!("安全指针算术: 0x{:x} - {} = 0x{:x}", ptr.address, offset, new_address));
                        Ok(Value::Pointer(new_ptr))
                    },
                    Err(e) => {
                        if e.contains("溢出") {
                            Err(PointerError::PointerArithmeticOverflow)
                        } else if e.contains("下溢") {
                            Err(PointerError::PointerArithmeticUnderflow)
                        } else if e.contains("范围") {
                            Err(PointerError::AddressOutOfRange(ptr.address))
                        } else {
                            Err(PointerError::InvalidAddress(ptr.address))
                        }
                    }
                }
            },
            (Value::Pointer(ptr1), crate::ast::PointerArithmeticOp::Diff, Value::Pointer(ptr2)) => {
                // 检查指针类型是否兼容
                if !self.are_pointer_types_compatible(&ptr1.target_type, &ptr2.target_type) {
                    return Err(PointerError::IncompatiblePointerTypes);
                }

                // 检查指针操作的有效性
                self.check_pointer_operation_validity(ptr1, "算术运算")?;
                self.check_pointer_operation_validity(ptr2, "算术运算")?;

                let element_size = self.get_pointer_element_size(&ptr1.target_type);

                // 检查除零
                if element_size == 0 {
                    return Err(PointerError::InvalidPointerLevel);
                }

                let diff = (ptr1.address as isize - ptr2.address as isize) / element_size as isize;

                debug_println(&format!("安全指针差值: 0x{:x} - 0x{:x} = {}", ptr1.address, ptr2.address, diff));
                Ok(Value::Int(diff as i32))
            },
            _ => {
                Err(PointerError::IncompatiblePointerTypes)
            }
        }
    }

    // 指针算术运算（带安全检查）
    fn evaluate_pointer_arithmetic(&mut self, left: &Expression, op: &crate::ast::PointerArithmeticOp, right: &Expression) -> Value {
        debug_println("执行指针算术运算");

        let left_val = self.evaluate_expression(left);
        let right_val = self.evaluate_expression(right);

        match (&left_val, op, &right_val) {
            (Value::Pointer(ptr), crate::ast::PointerArithmeticOp::Add, Value::Int(offset)) => {
                // 检查是否为函数指针（不允许算术运算）
                if matches!(ptr.target_type, PointerType::Function(_, _)) {
                    panic!("不允许对函数指针进行算术运算");
                }

                let element_size = self.get_pointer_element_size(&ptr.target_type);

                // 使用安全的指针算术
                match safe_pointer_arithmetic(ptr.address, *offset as isize, element_size, ptr.tag_id) {
                    Ok(new_address) => {
                        let new_ptr = PointerInstance {
                            address: new_address,
                            target_type: ptr.target_type.clone(),
                            is_null: false,
                            level: ptr.level,
                            tag_id: None, // 算术结果不继承标记
                        };

                        debug_println(&format!("安全指针算术: 0x{:x} + {} = 0x{:x}", ptr.address, offset, new_address));
                        Value::Pointer(new_ptr)
                    },
                    Err(e) => {
                        panic!("指针算术失败: {}", e);
                    }
                }
            },
            (Value::Pointer(ptr), crate::ast::PointerArithmeticOp::Sub, Value::Int(offset)) => {
                // 检查是否为函数指针
                if matches!(ptr.target_type, PointerType::Function(_, _)) {
                    panic!("不允许对函数指针进行算术运算");
                }

                let element_size = self.get_pointer_element_size(&ptr.target_type);

                // 使用安全的指针算术
                match safe_pointer_arithmetic(ptr.address, -(*offset as isize), element_size, ptr.tag_id) {
                    Ok(new_address) => {
                        let new_ptr = PointerInstance {
                            address: new_address,
                            target_type: ptr.target_type.clone(),
                            is_null: false,
                            level: ptr.level,
                            tag_id: None, // 算术结果不继承标记
                        };

                        debug_println(&format!("安全指针算术: 0x{:x} - {} = 0x{:x}", ptr.address, offset, new_address));
                        Value::Pointer(new_ptr)
                    },
                    Err(e) => {
                        panic!("指针算术失败: {}", e);
                    }
                }
            },
            (Value::Pointer(ptr1), crate::ast::PointerArithmeticOp::Diff, Value::Pointer(ptr2)) => {
                // 检查指针类型是否兼容
                if !self.are_pointer_types_compatible(&ptr1.target_type, &ptr2.target_type) {
                    panic!("不兼容的指针类型无法计算差值");
                }

                // 检查是否为函数指针
                if matches!(ptr1.target_type, PointerType::Function(_, _)) {
                    panic!("不允许对函数指针进行算术运算");
                }

                let element_size = self.get_pointer_element_size(&ptr1.target_type);

                // 检查除零
                if element_size == 0 {
                    panic!("指针元素大小为零，无法计算差值");
                }

                let diff = (ptr1.address as isize - ptr2.address as isize) / element_size as isize;

                debug_println(&format!("指针差值: 0x{:x} - 0x{:x} = {}", ptr1.address, ptr2.address, diff));
                Value::Int(diff as i32)
            },
            _ => {
                panic!("不支持的指针算术运算: {:?} {:?} {:?}", left_val, op, right_val);
            }
        }
    }

    // 检查指针类型是否兼容
    fn are_pointer_types_compatible(&self, type1: &PointerType, type2: &PointerType) -> bool {
        match (type1, type2) {
            (PointerType::Int, PointerType::Int) => true,
            (PointerType::Float, PointerType::Float) => true,
            (PointerType::Bool, PointerType::Bool) => true,
            (PointerType::String, PointerType::String) => true,
            (PointerType::Long, PointerType::Long) => true,
            (PointerType::Enum(name1), PointerType::Enum(name2)) => name1 == name2,
            (PointerType::Class(name1), PointerType::Class(name2)) => name1 == name2,
            (PointerType::Pointer(inner1), PointerType::Pointer(inner2)) => {
                self.are_pointer_types_compatible(inner1, inner2)
            },
            _ => false,
        }
    }

    // 函数指针调用
    fn call_function_pointer(&mut self, func_expr: &Expression, args: &[Expression]) -> Value {
        debug_println("调用函数指针");

        let func_val = self.evaluate_expression(func_expr);

        match func_val {
            Value::FunctionPointer(func_ptr) => {
                if func_ptr.is_null {
                    panic!("尝试调用空函数指针");
                }

                // 求值参数
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.evaluate_expression(arg));
                }

                if func_ptr.is_lambda {
                    // 调用Lambda函数
                    self.call_lambda_function(&func_ptr, evaluated_args)
                } else {
                    // 调用普通函数
                    self.call_named_function(&func_ptr.function_name, evaluated_args)
                }
            },
            Value::LambdaFunctionPointer(lambda_ptr) => {
                if lambda_ptr.is_null {
                    panic!("尝试调用空Lambda函数指针");
                }

                // 求值参数
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.evaluate_expression(arg));
                }

                // 调用Lambda函数
                self.call_lambda_function_with_params(&lambda_ptr, evaluated_args)
            },
            _ => {
                panic!("尝试调用非函数指针: {:?}", func_val);
            }
        }
    }

    // 创建函数指针
    fn create_function_pointer(&mut self, func_name: &str) -> Value {
        debug_println(&format!("创建函数指针: {}", func_name));

        // 检查函数是否存在
        if !self.functions.contains_key(func_name) {
            panic!("函数 '{}' 不存在", func_name);
        }

        let function = &self.functions[func_name];

        // 提取参数类型
        let param_types: Vec<crate::ast::Type> = function.parameters.iter()
            .map(|p| p.param_type.clone())
            .collect();

        let func_ptr = FunctionPointerInstance {
            function_name: func_name.to_string(),
            param_types,
            return_type: Box::new(function.return_type.clone()),
            is_null: false,
            is_lambda: false,
            lambda_body: None,
        };

        debug_println(&format!("创建函数指针成功: {}", func_name));
        Value::FunctionPointer(func_ptr)
    }

    // 创建Lambda函数指针
    fn create_lambda_function_pointer(&mut self, params: &[crate::ast::Parameter], return_type: &crate::ast::Type, body: &crate::ast::Statement) -> Value {
        debug_println("创建Lambda函数指针");

        // 提取参数类型
        let param_types: Vec<crate::ast::Type> = params.iter()
            .map(|p| p.param_type.clone())
            .collect();

        let func_ptr = FunctionPointerInstance {
            function_name: "lambda".to_string(),
            param_types,
            return_type: Box::new(return_type.clone()),
            is_null: false,
            is_lambda: true,
            lambda_body: Some(Box::new(body.clone())),
        };

        debug_println("创建Lambda函数指针成功");
        Value::FunctionPointer(func_ptr)
    }

    // 调用Lambda函数
    fn call_lambda_function(&mut self, func_ptr: &FunctionPointerInstance, args: Vec<Value>) -> Value {
        debug_println("调用Lambda函数");

        if let Some(body) = &func_ptr.lambda_body {
            // 保存当前局部环境
            let saved_local_env = self.local_env.clone();

            // 创建Lambda执行环境
            let mut lambda_env = HashMap::new();

            // 绑定参数
            for (i, arg) in args.iter().enumerate() {
                if i < func_ptr.param_types.len() {
                    let param_name = format!("param_{}", i); // 简化的参数名
                    lambda_env.insert(param_name, arg.clone());
                }
            }

            // 设置Lambda环境
            self.local_env.extend(lambda_env);

            // 执行Lambda体
            let result = match body.as_ref() {
                crate::ast::Statement::Return(expr) => {
                    if let Some(expr) = expr {
                        self.evaluate_expression(expr)
                    } else {
                        Value::None
                    }
                },
                crate::ast::Statement::FunctionCallStatement(expr) => {
                    self.evaluate_expression(expr)
                },
                _ => {
                    // 对于其他类型的语句，暂时返回None
                    Value::None
                }
            };

            // 恢复环境
            self.local_env = saved_local_env;

            result
        } else {
            panic!("Lambda函数体为空");
        }
    }

    // 创建Lambda表达式函数指针
    fn create_lambda_expression_pointer(&mut self, params: &[crate::ast::Parameter], body: &crate::ast::Expression) -> Value {
        debug_println("创建Lambda表达式函数指针");

        // 提取参数类型
        let param_types: Vec<crate::ast::Type> = params.iter()
            .map(|p| p.param_type.clone())
            .collect();

        // 推断返回类型（简化实现，使用Auto）
        let return_type = crate::ast::Type::Auto;

        // 将表达式包装为Return语句
        let lambda_body = crate::ast::Statement::Return(Some(body.clone()));

        // 捕获当前环境作为闭包环境
        let mut closure_env = std::collections::HashMap::new();

        // 分析Lambda体中使用的变量，捕获外部作用域的变量
        let used_vars = self.analyze_lambda_variables(body, params);
        for var_name in used_vars {
            if let Some(value) = self.local_env.get(&var_name).or_else(|| self.global_env.get(&var_name)) {
                closure_env.insert(var_name, value.clone());
            }
        }

        // 创建扩展的函数指针实例，包含参数信息
        let func_ptr = LambdaFunctionPointerInstance {
            function_name: "lambda".to_string(),
            param_types,
            return_type: Box::new(return_type),
            is_null: false,
            is_lambda: true,
            lambda_body: Some(Box::new(lambda_body)),
            lambda_params: params.to_vec(), // 保存完整的参数信息
            closure_env,
        };

        debug_println("创建Lambda表达式函数指针成功");
        Value::LambdaFunctionPointer(func_ptr)
    }

    // 创建Lambda块函数指针
    fn create_lambda_block_pointer(&mut self, params: &[crate::ast::Parameter], statements: &[crate::ast::Statement]) -> Value {
        debug_println("创建Lambda块函数指针");

        // 提取参数类型
        let param_types: Vec<crate::ast::Type> = params.iter()
            .map(|p| p.param_type.clone())
            .collect();

        // 推断返回类型（简化实现，使用Auto）
        let return_type = crate::ast::Type::Auto;

        // 暂时简化：只支持单个return语句的Lambda块
        let lambda_body = if let Some(first_stmt) = statements.first() {
            first_stmt.clone()
        } else {
            crate::ast::Statement::Return(Some(crate::ast::Expression::None))
        };

        // 为Lambda块创建空的闭包环境（Lambda块通常不需要闭包）
        let closure_env = std::collections::HashMap::new();

        let func_ptr = LambdaFunctionPointerInstance {
            function_name: "lambda".to_string(),
            param_types,
            return_type: Box::new(return_type),
            is_null: false,
            is_lambda: true,
            lambda_body: Some(Box::new(lambda_body)),
            lambda_params: params.to_vec(), // 保存完整的参数信息
            closure_env,
        };

        debug_println("创建Lambda块函数指针成功");
        Value::LambdaFunctionPointer(func_ptr)
    }

    // 分析Lambda表达式中使用的变量，用于闭包捕获
    fn analyze_lambda_variables(&self, expr: &Expression, params: &[crate::ast::Parameter]) -> Vec<String> {
        let mut used_vars = Vec::new();
        let param_names: std::collections::HashSet<String> = params.iter().map(|p| p.name.clone()).collect();

        self.collect_variables_from_expression(expr, &mut used_vars, &param_names);

        // 去重
        used_vars.sort();
        used_vars.dedup();

        debug_println(&format!("Lambda闭包捕获变量: {:?}", used_vars));
        used_vars
    }

    // 递归收集表达式中使用的变量
    fn collect_variables_from_expression(&self, expr: &Expression, used_vars: &mut Vec<String>, param_names: &std::collections::HashSet<String>) {
        match expr {
            Expression::Variable(name) => {
                // 如果不是参数，则是外部变量
                if !param_names.contains(name) {
                    used_vars.push(name.clone());
                }
            },
            Expression::BinaryOp(left, _, right) => {
                self.collect_variables_from_expression(left, used_vars, param_names);
                self.collect_variables_from_expression(right, used_vars, param_names);
            },
            Expression::PreIncrement(var_name) | Expression::PreDecrement(var_name) |
            Expression::PostIncrement(var_name) | Expression::PostDecrement(var_name) => {
                if !param_names.contains(var_name) {
                    used_vars.push(var_name.clone());
                }
            },
            Expression::FunctionCall(_, args) => {
                for arg in args {
                    self.collect_variables_from_expression(arg, used_vars, param_names);
                }
            },
            Expression::FunctionPointerCall(func_expr, args) => {
                self.collect_variables_from_expression(func_expr, used_vars, param_names);
                for arg in args {
                    self.collect_variables_from_expression(arg, used_vars, param_names);
                }
            },
            Expression::ArrayAccess(array_expr, index_expr) => {
                self.collect_variables_from_expression(array_expr, used_vars, param_names);
                self.collect_variables_from_expression(index_expr, used_vars, param_names);
            },
            Expression::ArrayLiteral(elements) => {
                for elem in elements {
                    self.collect_variables_from_expression(elem, used_vars, param_names);
                }
            },
            Expression::FieldAccess(obj_expr, _) => {
                self.collect_variables_from_expression(obj_expr, used_vars, param_names);
            },
            Expression::MethodCall(obj_expr, _, args) => {
                self.collect_variables_from_expression(obj_expr, used_vars, param_names);
                for arg in args {
                    self.collect_variables_from_expression(arg, used_vars, param_names);
                }
            },
            // 其他表达式类型不包含变量引用
            _ => {}
        }
    }

    // 调用带完整参数信息的Lambda函数
    fn call_lambda_function_with_params(&mut self, lambda_ptr: &LambdaFunctionPointerInstance, args: Vec<Value>) -> Value {
        debug_println("调用Lambda函数（带参数信息）");

        if let Some(body) = &lambda_ptr.lambda_body {
            // 检查参数数量
            if args.len() != lambda_ptr.lambda_params.len() {
                panic!("Lambda函数期望 {} 个参数，但得到 {} 个",
                       lambda_ptr.lambda_params.len(), args.len());
            }

            // 保存当前局部环境
            let saved_local_env = self.local_env.clone();

            // 创建Lambda执行环境
            let mut lambda_env = HashMap::new();

            // 绑定参数（使用实际的参数名）
            for (i, (param, arg)) in lambda_ptr.lambda_params.iter().zip(args.iter()).enumerate() {
                lambda_env.insert(param.name.clone(), arg.clone());
                debug_println(&format!("绑定参数: {} = {:?}", param.name, arg));
            }

            // 设置Lambda环境
            self.local_env.extend(lambda_env);

            // 执行Lambda体
            let result = match body.as_ref() {
                crate::ast::Statement::Return(expr) => {
                    if let Some(expr) = expr {
                        self.evaluate_expression(expr)
                    } else {
                        Value::None
                    }
                },
                crate::ast::Statement::FunctionCallStatement(expr) => {
                    self.evaluate_expression(expr)
                },
                // 暂时不支持Block语句，因为AST中没有定义
                // 如果需要支持多语句Lambda，需要在AST中添加Block语句类型
                _ => {
                    // 对于其他类型的语句，暂时返回None
                    Value::None
                }
            };

            // 恢复环境
            self.local_env = saved_local_env;

            debug_println(&format!("Lambda函数执行完成，结果: {:?}", result));
            result
        } else {
            panic!("Lambda函数体为空");
        }
    }

    // 调用命名函数
    fn call_named_function(&mut self, func_name: &str, args: Vec<Value>) -> Value {
        debug_println(&format!("通过函数指针调用函数: {}", func_name));

        // 检查函数是否存在
        if !self.functions.contains_key(func_name) {
            panic!("函数 '{}' 不存在", func_name);
        }

        let function = self.functions[func_name].clone();

        // 检查参数数量
        if args.len() != function.parameters.len() {
            panic!("函数 '{}' 期望 {} 个参数，但得到 {} 个",
                   func_name, function.parameters.len(), args.len());
        }

        // 保存当前局部环境
        let saved_local_env = self.local_env.clone();

        // 清空局部环境，为函数调用创建新的作用域
        self.local_env.clear();

        // 绑定参数
        for (i, param) in function.parameters.iter().enumerate() {
            if i < args.len() {
                self.local_env.insert(param.name.clone(), args[i].clone());
            }
        }

        // 执行函数体（简化实现）
        let mut result = Value::None;

        // 暂时简化：只处理简单的return语句
        for statement in &function.body {
            if let crate::ast::Statement::Return(expr) = statement {
                if let Some(expr) = expr {
                    result = self.evaluate_expression(expr);
                } else {
                    result = Value::None;
                }
                break;
            }
            // 其他语句暂时跳过
        }

        // 恢复局部环境
        self.local_env = saved_local_env;

        // 如果没有显式返回值，根据返回类型返回默认值
        if matches!(result, Value::None) {
            match function.return_type {
                crate::ast::Type::Int => Value::Int(0),
                crate::ast::Type::Float => Value::Float(0.0),
                crate::ast::Type::Bool => Value::Bool(false),
                crate::ast::Type::String => Value::String("".to_string()),
                crate::ast::Type::Long => Value::Long(0),
                crate::ast::Type::Void => Value::None,
                _ => Value::None,
            }
        } else {
            result
        }
    }

    // 将值转换为指针类型信息
    fn value_to_pointer_type(&self, value: &Value) -> PointerType {
        match value {
            Value::Int(_) => PointerType::Int,
            Value::Float(_) => PointerType::Float,
            Value::Bool(_) => PointerType::Bool,
            Value::String(_) => PointerType::String,
            Value::Long(_) => PointerType::Long,
            Value::EnumValue(enum_val) => PointerType::Enum(enum_val.enum_name.clone()),
            Value::Object(_) => PointerType::Class("Object".to_string()),
            Value::Pointer(ptr) => PointerType::Pointer(Box::new(ptr.target_type.clone())),
            _ => PointerType::Int, // 默认类型
        }
    }

    // 获取指针元素大小（平台无关且类型安全）
    fn get_pointer_element_size(&self, ptr_type: &PointerType) -> usize {
        match ptr_type {
            PointerType::Int => std::mem::size_of::<i32>(),
            PointerType::Float => std::mem::size_of::<f64>(),
            PointerType::Bool => std::mem::size_of::<bool>(),
            PointerType::String => std::mem::size_of::<usize>(), // 字符串指针大小
            PointerType::Long => std::mem::size_of::<i64>(),
            PointerType::Enum(_) => std::mem::size_of::<usize>() * 4, // 枚举基础大小
            PointerType::Class(_) => std::mem::size_of::<usize>() * 8, // 对象基础大小
            PointerType::Function(_, _) => {
                // 函数指针不应该进行算术运算，但为了类型完整性提供大小
                std::mem::size_of::<usize>()
            },
            PointerType::Pointer(_) => std::mem::size_of::<usize>(), // 指针大小
            PointerType::Array(element_type, size) => {
                // 数组大小 = 元素大小 * 元素数量
                self.get_pointer_element_size(element_type) * size
            },
        }
    }

    // 严格的指针类型验证
    fn validate_pointer_type(&self, ptr_type: &PointerType, operation: &str) -> Result<(), String> {
        match ptr_type {
            PointerType::Function(_, _) => {
                if operation == "arithmetic" {
                    Err("函数指针不支持算术运算".to_string())
                } else {
                    Ok(())
                }
            },
            PointerType::Pointer(inner_type) => {
                // 递归验证内层类型
                self.validate_pointer_type(inner_type, operation)
            },
            _ => Ok(()),
        }
    }

    // 增强的类型检查
    fn check_pointer_operation_validity(&self, ptr: &PointerInstance, operation: &str) -> Result<(), PointerError> {
        // 检查空指针
        if ptr.is_null {
            return Err(PointerError::NullPointerAccess);
        }

        // 检查指针类型
        match self.validate_pointer_type(&ptr.target_type, operation) {
            Ok(()) => {},
            Err(_) => {
                if operation == "算术运算" && matches!(ptr.target_type, PointerType::Function(_, _)) {
                    return Err(PointerError::FunctionPointerArithmetic);
                } else {
                    return Err(PointerError::IncompatiblePointerTypes);
                }
            }
        }

        // 检查指针级别
        if ptr.level == 0 {
            return Err(PointerError::InvalidPointerLevel);
        }

        Ok(())
    }

    // 安全版本的指针成员访问
    fn evaluate_pointer_member_access_safe(&mut self, ptr_expr: &Expression, member_name: &str) -> Result<Value, PointerError> {
        debug_println("执行安全指针成员访问");

        // 计算指针表达式
        let pointer_value = self.evaluate_expression(ptr_expr);

        match pointer_value {
            Value::Pointer(ptr) => {
                if ptr.is_null {
                    return Err(PointerError::NullPointerAccess);
                }

                // 检查指针操作的有效性
                self.check_pointer_operation_validity(&ptr, "成员访问")?;

                // 使用增强的安全检查
                let validation_result = if let Some(tag_id) = ptr.tag_id {
                    validate_pointer_safe(ptr.address, tag_id)
                } else {
                    validate_pointer(ptr.address)
                };

                if let Err(_) = validation_result {
                    return Err(PointerError::InvalidAddress(ptr.address));
                }

                // 检查悬空指针
                let is_dangling = if let Some(tag_id) = ptr.tag_id {
                    is_dangling_pointer(tag_id)
                } else {
                    is_dangling_pointer_by_address(ptr.address)
                };

                if is_dangling {
                    return Err(PointerError::DanglingPointerAccess(ptr.address));
                }

                // 安全读取内存中的对象
                let read_result = if let Some(tag_id) = ptr.tag_id {
                    read_memory_safe(ptr.address, tag_id)
                } else {
                    read_memory(ptr.address)
                };

                match read_result {
                    Ok(object_value) => {
                        // 根据对象类型访问成员
                        match object_value {
                            Value::Object(obj) => {
                                // 访问对象成员
                                if let Some(member_value) = obj.fields.get(member_name) {
                                    debug_println(&format!("安全指针成员访问: 0x{:x}->{} = {:?}", ptr.address, member_name, member_value));
                                    Ok(member_value.clone())
                                } else {
                                    Err(PointerError::InvalidAddress(ptr.address)) // 成员不存在
                                }
                            },
                            Value::EnumValue(enum_val) => {
                                // 访问枚举成员（如果有的话）
                                match member_name {
                                    "variant" => Ok(Value::String(enum_val.variant_name.clone())),
                                    "name" => Ok(Value::String(enum_val.enum_name.clone())),
                                    _ => {
                                        // 尝试访问枚举的字段（通过索引）
                                        match member_name.parse::<usize>() {
                                            Ok(index) => {
                                                if index < enum_val.fields.len() {
                                                    Ok(enum_val.fields[index].clone())
                                                } else {
                                                    Err(PointerError::InvalidAddress(ptr.address))
                                                }
                                            },
                                            Err(_) => {
                                                // 如果不是数字索引，返回错误
                                                Err(PointerError::InvalidAddress(ptr.address))
                                            }
                                        }
                                    }
                                }
                            },
                            Value::String(s) => {
                                // 字符串的内置方法
                                match member_name {
                                    "length" => Ok(Value::Int(s.len() as i32)),
                                    _ => Err(PointerError::InvalidAddress(ptr.address))
                                }
                            },
                            Value::Array(arr) => {
                                // 数组的内置方法
                                match member_name {
                                    "length" => Ok(Value::Int(arr.len() as i32)),
                                    _ => Err(PointerError::InvalidAddress(ptr.address))
                                }
                            },
                            _ => {
                                // 其他类型暂不支持成员访问
                                Err(PointerError::InvalidAddress(ptr.address))
                            }
                        }
                    },
                    Err(e) => {
                        Err(PointerError::MemoryReadFailed(e))
                    }
                }
            },
            _ => {
                Err(PointerError::InvalidAddress(0)) // 非指针值
            }
        }
    }

    // 安全版本的数组指针访问
    fn evaluate_array_pointer_access_safe(&mut self, array_ptr_expr: &Expression, index_expr: &Expression) -> Result<Value, PointerError> {
        debug_println("执行安全数组指针访问");

        // 计算数组指针表达式
        let array_pointer_value = self.evaluate_expression(array_ptr_expr);
        let index_value = self.evaluate_expression(index_expr);

        // 获取索引值
        let index = match index_value {
            Value::Int(i) => i as usize,
            _ => return Err(PointerError::InvalidAddress(0)),
        };

        match array_pointer_value {
            Value::ArrayPointer(array_ptr) => {
                if array_ptr.is_null {
                    return Err(PointerError::NullPointerAccess);
                }

                // 检查索引边界
                if index >= array_ptr.array_size {
                    return Err(PointerError::AddressOutOfRange(array_ptr.address + index));
                }

                // 计算元素地址
                let element_size = self.get_pointer_type_size(&array_ptr.element_type);
                let element_address = array_ptr.address + (index * element_size);

                // 验证元素地址
                let validation_result = if let Some(tag_id) = array_ptr.tag_id {
                    validate_pointer_safe(element_address, tag_id)
                } else {
                    validate_pointer(element_address)
                };

                if let Err(_) = validation_result {
                    return Err(PointerError::InvalidAddress(element_address));
                }

                // 读取元素值
                let read_result = if let Some(tag_id) = array_ptr.tag_id {
                    read_memory_safe(element_address, tag_id)
                } else {
                    read_memory(element_address)
                };

                match read_result {
                    Ok(element_value) => {
                        debug_println(&format!("安全数组指针访问: 0x{:x}[{}] = {:?}", array_ptr.address, index, element_value));
                        Ok(element_value)
                    },
                    Err(e) => {
                        Err(PointerError::MemoryReadFailed(e))
                    }
                }
            },
            Value::Pointer(ptr) => {
                // 如果是普通指针，尝试作为数组访问
                if ptr.is_null {
                    return Err(PointerError::NullPointerAccess);
                }

                // 计算元素地址（假设指针指向数组的第一个元素）
                let element_size = self.get_pointer_element_size(&ptr.target_type);
                let element_address = ptr.address + (index * element_size);

                // 使用安全的指针算术
                match safe_pointer_arithmetic(ptr.address, index as isize, element_size, ptr.tag_id) {
                    Ok(safe_address) => {
                        // 读取元素值
                        let read_result = if let Some(tag_id) = ptr.tag_id {
                            read_memory_safe(safe_address, tag_id)
                        } else {
                            read_memory(safe_address)
                        };

                        match read_result {
                            Ok(element_value) => {
                                debug_println(&format!("安全指针数组访问: 0x{:x}[{}] = {:?}", ptr.address, index, element_value));
                                Ok(element_value)
                            },
                            Err(e) => {
                                Err(PointerError::MemoryReadFailed(e))
                            }
                        }
                    },
                    Err(e) => {
                        if e.contains("溢出") {
                            Err(PointerError::PointerArithmeticOverflow)
                        } else if e.contains("范围") {
                            Err(PointerError::AddressOutOfRange(ptr.address))
                        } else {
                            Err(PointerError::InvalidAddress(ptr.address))
                        }
                    }
                }
            },
            _ => {
                Err(PointerError::InvalidAddress(0)) // 非指针值
            }
        }
    }

    // 获取指针类型的大小
    fn get_pointer_type_size(&self, ptr_type: &PointerType) -> usize {
        match ptr_type {
            PointerType::Int => std::mem::size_of::<i32>(),
            PointerType::Float => std::mem::size_of::<f64>(),
            PointerType::Bool => std::mem::size_of::<bool>(),
            PointerType::String => std::mem::size_of::<usize>(),
            PointerType::Long => std::mem::size_of::<i64>(),
            PointerType::Enum(_) => std::mem::size_of::<usize>() * 4,
            PointerType::Class(_) => std::mem::size_of::<usize>() * 8,
            PointerType::Function(_, _) => std::mem::size_of::<usize>(),
            PointerType::Pointer(_) => std::mem::size_of::<usize>(),
            PointerType::Array(element_type, size) => {
                self.get_pointer_type_size(element_type) * size
            },
        }
    }

    // 安全版本的指针数组访问
    fn evaluate_pointer_array_access_safe(&mut self, ptr_array_expr: &Expression, index_expr: &Expression) -> Result<Value, PointerError> {
        debug_println("执行安全指针数组访问");

        // 计算指针数组表达式
        let pointer_array_value = self.evaluate_expression(ptr_array_expr);
        let index_value = self.evaluate_expression(index_expr);

        // 获取索引值
        let index = match index_value {
            Value::Int(i) => {
                if i < 0 {
                    return Err(PointerError::InvalidAddress(0));
                }
                i as usize
            },
            _ => return Err(PointerError::InvalidAddress(0)),
        };

        match pointer_array_value {
            Value::PointerArray(ptr_array) => {
                // 检查索引边界
                if index >= ptr_array.array_size {
                    return Err(PointerError::AddressOutOfRange(index));
                }

                // 检查指针数组是否有足够的元素
                if index >= ptr_array.pointers.len() {
                    return Err(PointerError::AddressOutOfRange(index));
                }

                // 获取指定索引的指针
                let pointer = &ptr_array.pointers[index];

                // 返回指针值（不是解引用）
                debug_println(&format!("安全指针数组访问: ptrArray[{}] = 0x{:x}", index, pointer.address));
                Ok(Value::Pointer(pointer.clone()))
            },
            Value::Array(array) => {
                // 如果是普通数组，检查是否包含指针
                if index >= array.len() {
                    return Err(PointerError::AddressOutOfRange(index));
                }

                match &array[index] {
                    Value::Pointer(ptr) => {
                        debug_println(&format!("安全数组指针访问: array[{}] = 0x{:x}", index, ptr.address));
                        Ok(Value::Pointer(ptr.clone()))
                    },
                    _ => {
                        Err(PointerError::InvalidAddress(0)) // 数组元素不是指针
                    }
                }
            },
            _ => {
                Err(PointerError::InvalidAddress(0)) // 非指针数组值
            }
        }
    }

    /// v0.7.4新增：优化的变量访问（跳过运行时检查）
    fn get_variable_optimized(&self, name: &str) -> Value {
        // 对于安全变量，使用最快的访问路径
        // 直接按优先级顺序访问，不进行额外的安全检查

        // 1. 常量（最快）
        if let Some(value) = self.constants.get(name) {
            return value.clone();
        }

        // 2. 局部变量
        if let Some(value) = self.local_env.get(name) {
            return value.clone();
        }

        // 3. 全局变量
        if let Some(value) = self.global_env.get(name) {
            return value.clone();
        }

        // 如果到这里还没找到，说明分析有误，回退到普通处理
        Value::None
    }


}