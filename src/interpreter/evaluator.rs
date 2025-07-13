use crate::ast::{Expression, BinaryOperator, CompareOperator, LogicalOperator};
use crate::interpreter::value::Value;
use std::collections::HashMap;

pub trait Evaluator {
    fn evaluate_expression(&mut self, expr: &Expression) -> Value;
    fn perform_binary_operation(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Value;
    fn get_variable(&self, name: &str) -> Option<Value>;
    fn call_function(&mut self, function_name: &str, args: Vec<Value>) -> Value;
}

pub fn perform_binary_operation(left: &Value, op: &BinaryOperator, right: &Value) -> Value {
    match (left, op, right) {
        // 整数运算
        (Value::Int(l), BinaryOperator::Add, Value::Int(r)) => Value::Int(l + r),
        (Value::Int(l), BinaryOperator::Subtract, Value::Int(r)) => Value::Int(l - r),
        (Value::Int(l), BinaryOperator::Multiply, Value::Int(r)) => Value::Int(l * r),
        (Value::Int(l), BinaryOperator::Divide, Value::Int(r)) => {
            if *r == 0 {
                panic!("除以零错误");
            }
            Value::Int(l / r)
        },
        (Value::Int(l), BinaryOperator::Modulo, Value::Int(r)) => {
            if *r == 0 {
                panic!("除以零错误");
            }
            Value::Int(l % r)
        },
        
        // 浮点数运算
        (Value::Float(l), BinaryOperator::Add, Value::Float(r)) => Value::Float(l + r),
        (Value::Float(l), BinaryOperator::Subtract, Value::Float(r)) => Value::Float(l - r),
        (Value::Float(l), BinaryOperator::Multiply, Value::Float(r)) => Value::Float(l * r),
        (Value::Float(l), BinaryOperator::Divide, Value::Float(r)) => {
            if *r == 0.0 {
                panic!("除以零错误");
            }
            Value::Float(l / r)
        },
        
        // 整数和浮点数混合运算
        (Value::Int(l), BinaryOperator::Add, Value::Float(r)) => Value::Float(*l as f64 + r),
        (Value::Float(l), BinaryOperator::Add, Value::Int(r)) => Value::Float(l + *r as f64),
        (Value::Int(l), BinaryOperator::Subtract, Value::Float(r)) => Value::Float(*l as f64 - r),
        (Value::Float(l), BinaryOperator::Subtract, Value::Int(r)) => Value::Float(l - *r as f64),
        (Value::Int(l), BinaryOperator::Multiply, Value::Float(r)) => Value::Float(*l as f64 * r),
        (Value::Float(l), BinaryOperator::Multiply, Value::Int(r)) => Value::Float(l * *r as f64),
        (Value::Int(l), BinaryOperator::Divide, Value::Float(r)) => {
            if *r == 0.0 {
                panic!("除以零错误");
            }
            Value::Float(*l as f64 / r)
        },
        (Value::Float(l), BinaryOperator::Divide, Value::Int(r)) => {
            if *r == 0 {
                panic!("除以零错误");
            }
            Value::Float(l / *r as f64)
        },
        
        // 长整型运算
        (Value::Long(l), BinaryOperator::Add, Value::Long(r)) => Value::Long(l + r),
        (Value::Long(l), BinaryOperator::Subtract, Value::Long(r)) => Value::Long(l - r),
        (Value::Long(l), BinaryOperator::Multiply, Value::Long(r)) => Value::Long(l * r),
        (Value::Long(l), BinaryOperator::Divide, Value::Long(r)) => {
            if *r == 0 {
                panic!("除以零错误");
            }
            Value::Long(l / r)
        },
        (Value::Long(l), BinaryOperator::Modulo, Value::Long(r)) => {
            if *r == 0 {
                panic!("除以零错误");
            }
            Value::Long(l % r)
        },
        
        // 整数和长整型混合运算
        (Value::Int(l), BinaryOperator::Add, Value::Long(r)) => Value::Long(*l as i64 + r),
        (Value::Long(l), BinaryOperator::Add, Value::Int(r)) => Value::Long(l + *r as i64),
        (Value::Int(l), BinaryOperator::Subtract, Value::Long(r)) => Value::Long(*l as i64 - r),
        (Value::Long(l), BinaryOperator::Subtract, Value::Int(r)) => Value::Long(l - *r as i64),
        (Value::Int(l), BinaryOperator::Multiply, Value::Long(r)) => Value::Long(*l as i64 * r),
        (Value::Long(l), BinaryOperator::Multiply, Value::Int(r)) => Value::Long(l * *r as i64),
        (Value::Int(l), BinaryOperator::Divide, Value::Long(r)) => {
            if *r == 0 {
                panic!("除以零错误");
            }
            Value::Long(*l as i64 / r)
        },
        (Value::Long(l), BinaryOperator::Divide, Value::Int(r)) => {
            if *r == 0 {
                panic!("除以零错误");
            }
            Value::Long(l / *r as i64)
        },
        
        // 字符串连接
        (Value::String(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.clone() + r),
        
        // 字符串和其他类型的连接
        (Value::String(l), BinaryOperator::Add, Value::Int(r)) => Value::String(l.clone() + &r.to_string()),
        (Value::String(l), BinaryOperator::Add, Value::Float(r)) => Value::String(l.clone() + &r.to_string()),
        (Value::String(l), BinaryOperator::Add, Value::Bool(r)) => Value::String(l.clone() + &r.to_string()),
        (Value::String(l), BinaryOperator::Add, Value::Long(r)) => Value::String(l.clone() + &r.to_string()),
        
        // 其他类型和字符串的连接
        (Value::Int(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.to_string() + r),
        (Value::Float(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.to_string() + r),
        (Value::Bool(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.to_string() + r),
        (Value::Long(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.to_string() + r),
        
        // 不支持的操作
        _ => panic!("不支持的二元操作: {:?} {:?} {:?}", left, op, right),
    }
}

pub fn evaluate_compare_operation(left: &Value, op: &CompareOperator, right: &Value) -> Value {
    match (op, left, right) {
        // 整数比较
        (CompareOperator::Equal, Value::Int(l), Value::Int(r)) => Value::Bool(l == r),
        (CompareOperator::NotEqual, Value::Int(l), Value::Int(r)) => Value::Bool(l != r),
        (CompareOperator::Greater, Value::Int(l), Value::Int(r)) => Value::Bool(l > r),
        (CompareOperator::Less, Value::Int(l), Value::Int(r)) => Value::Bool(l < r),
        (CompareOperator::GreaterEqual, Value::Int(l), Value::Int(r)) => Value::Bool(l >= r),
        (CompareOperator::LessEqual, Value::Int(l), Value::Int(r)) => Value::Bool(l <= r),
        
        // 浮点数比较
        (CompareOperator::Equal, Value::Float(l), Value::Float(r)) => Value::Bool(l == r),
        (CompareOperator::NotEqual, Value::Float(l), Value::Float(r)) => Value::Bool(l != r),
        (CompareOperator::Greater, Value::Float(l), Value::Float(r)) => Value::Bool(l > r),
        (CompareOperator::Less, Value::Float(l), Value::Float(r)) => Value::Bool(l < r),
        (CompareOperator::GreaterEqual, Value::Float(l), Value::Float(r)) => Value::Bool(l >= r),
        (CompareOperator::LessEqual, Value::Float(l), Value::Float(r)) => Value::Bool(l <= r),
        
        // 长整型比较
        (CompareOperator::Equal, Value::Long(l), Value::Long(r)) => Value::Bool(l == r),
        (CompareOperator::NotEqual, Value::Long(l), Value::Long(r)) => Value::Bool(l != r),
        (CompareOperator::Greater, Value::Long(l), Value::Long(r)) => Value::Bool(l > r),
        (CompareOperator::Less, Value::Long(l), Value::Long(r)) => Value::Bool(l < r),
        (CompareOperator::GreaterEqual, Value::Long(l), Value::Long(r)) => Value::Bool(l >= r),
        (CompareOperator::LessEqual, Value::Long(l), Value::Long(r)) => Value::Bool(l <= r),
        
        // 字符串比较
        (CompareOperator::Equal, Value::String(l), Value::String(r)) => Value::Bool(l == r),
        (CompareOperator::NotEqual, Value::String(l), Value::String(r)) => Value::Bool(l != r),
        
        // 布尔值比较
        (CompareOperator::Equal, Value::Bool(l), Value::Bool(r)) => Value::Bool(l == r),
        (CompareOperator::NotEqual, Value::Bool(l), Value::Bool(r)) => Value::Bool(l != r),
        
        // 混合类型比较
        (CompareOperator::Equal, _, _) => Value::Bool(false), // 不同类型永远不相等
        (CompareOperator::NotEqual, _, _) => Value::Bool(true), // 不同类型永远不相等
        
        // 不支持的比较
        _ => panic!("不支持的比较操作: {:?} {:?} {:?}", left, op, right),
    }
} 