use crate::ast::{Expression, BinaryOperator, CompareOperator, LogicalOperator};
use super::value::Value;
use super::interpreter_core::{Interpreter, debug_println};
use super::function_calls::FunctionCallHandler;

pub trait ExpressionEvaluator {
    fn evaluate_expression(&mut self, expr: &Expression) -> Value;
    fn perform_binary_operation(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Value;
    fn get_variable(&self, name: &str) -> Option<Value>;
}

impl<'a> ExpressionEvaluator for Interpreter<'a> {
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
            Expression::Throw(exception_expr) => {
                // 计算异常表达式并抛出
                let exception_value = self.evaluate_expression(exception_expr);
                // 注意：这里我们返回异常值，但实际的抛出逻辑在语句执行器中处理
                exception_value
            }
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
}

impl<'a> Interpreter<'a> {
    fn evaluate_compare_operation(&self, left: &Value, op: &CompareOperator, right: &Value) -> Value {
        use super::evaluator::evaluate_compare_operation;
        evaluate_compare_operation(left, op, right)
    }
    
    fn evaluate_logical_operation(&mut self, left: &Expression, op: &LogicalOperator, right: &Expression) -> Value {
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
} 