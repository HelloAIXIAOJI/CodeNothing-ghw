use crate::ast::{Program, Expression, Statement, BinaryOperator, Type};
use std::collections::HashMap;
use std::fmt;

// 定义值类型，用于存储不同类型的值
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Long(i64),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Long(l) => write!(f, "{}", l),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            },
            Value::Map(map) => {
                write!(f, "{{")?;
                for (i, (key, val)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, val)?;
                }
                write!(f, "}}")
            }
        }
    }
}

pub fn interpret(program: &Program) -> Value {
    let mut interpreter = Interpreter::new(program);
    interpreter.run()
}

struct Interpreter<'a> {
    program: &'a Program,
    functions: HashMap<String, &'a crate::ast::Function>,
    // 全局变量环境
    global_env: HashMap<String, Value>,
    // 局部变量环境（函数内）
    local_env: HashMap<String, Value>,
}

impl<'a> Interpreter<'a> {
    fn new(program: &'a Program) -> Self {
        let mut functions = HashMap::new();
        for function in &program.functions {
            functions.insert(function.name.clone(), function);
        }
        
        Interpreter {
            program,
            functions,
            global_env: HashMap::new(),
            local_env: HashMap::new(),
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
    
    fn execute_function(&mut self, function: &'a crate::ast::Function) -> Value {
        // 注意：局部环境的设置已经在函数调用时完成
        // 不再需要在这里清空局部环境
        
        for statement in &function.body {
            match statement {
                Statement::Return(expr) => {
                    return self.evaluate_expression(expr);
                },
                Statement::VariableDeclaration(name, _type, expr) => {
                    let value = self.evaluate_expression(expr);
                    self.local_env.insert(name.clone(), value);
                },
                Statement::VariableAssignment(name, expr) => {
                    let value = self.evaluate_expression(expr);
                    // 先检查局部变量，再检查全局变量
                    if self.local_env.contains_key(name) {
                        self.local_env.insert(name.clone(), value);
                    } else if self.global_env.contains_key(name) {
                        self.global_env.insert(name.clone(), value);
                    } else {
                        panic!("未定义的变量: {}", name);
                    }
                }
            }
        }
        
        // 如果没有返回语句，默认返回0
        Value::Int(0)
    }
    
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
                
                // 然后查找函数并执行
                if let Some(function) = self.functions.get(name) {
                    // 检查参数数量是否匹配
                    if arg_values.len() != function.parameters.len() {
                        panic!("函数 '{}' 需要 {} 个参数，但提供了 {} 个", 
                            name, function.parameters.len(), arg_values.len());
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
                } else {
                    panic!("未定义的函数: {}", name);
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
                
                match (left_val, op, right_val) {
                    // 整数运算
                    (Value::Int(l), BinaryOperator::Add, Value::Int(r)) => Value::Int(l + r),
                    (Value::Int(l), BinaryOperator::Subtract, Value::Int(r)) => Value::Int(l - r),
                    (Value::Int(l), BinaryOperator::Multiply, Value::Int(r)) => Value::Int(l * r),
                    (Value::Int(l), BinaryOperator::Divide, Value::Int(r)) => {
                        if r == 0 {
                            panic!("除以零错误");
                        }
                        Value::Int(l / r)
                    },
                    (Value::Int(l), BinaryOperator::Modulo, Value::Int(r)) => {
                        if r == 0 {
                            panic!("除以零错误");
                        }
                        Value::Int(l % r)
                    },
                    
                    // 浮点数运算
                    (Value::Float(l), BinaryOperator::Add, Value::Float(r)) => Value::Float(l + r),
                    (Value::Float(l), BinaryOperator::Subtract, Value::Float(r)) => Value::Float(l - r),
                    (Value::Float(l), BinaryOperator::Multiply, Value::Float(r)) => Value::Float(l * r),
                    (Value::Float(l), BinaryOperator::Divide, Value::Float(r)) => {
                        if r == 0.0 {
                            panic!("除以零错误");
                        }
                        Value::Float(l / r)
                    },
                    
                    // 整数和浮点数混合运算
                    (Value::Int(l), BinaryOperator::Add, Value::Float(r)) => Value::Float(l as f64 + r),
                    (Value::Float(l), BinaryOperator::Add, Value::Int(r)) => Value::Float(l + r as f64),
                    (Value::Int(l), BinaryOperator::Subtract, Value::Float(r)) => Value::Float(l as f64 - r),
                    (Value::Float(l), BinaryOperator::Subtract, Value::Int(r)) => Value::Float(l - r as f64),
                    (Value::Int(l), BinaryOperator::Multiply, Value::Float(r)) => Value::Float(l as f64 * r),
                    (Value::Float(l), BinaryOperator::Multiply, Value::Int(r)) => Value::Float(l * r as f64),
                    (Value::Int(l), BinaryOperator::Divide, Value::Float(r)) => {
                        if r == 0.0 {
                            panic!("除以零错误");
                        }
                        Value::Float(l as f64 / r)
                    },
                    (Value::Float(l), BinaryOperator::Divide, Value::Int(r)) => {
                        if r == 0 {
                            panic!("除以零错误");
                        }
                        Value::Float(l / r as f64)
                    },
                    
                    // 长整型运算
                    (Value::Long(l), BinaryOperator::Add, Value::Long(r)) => Value::Long(l + r),
                    (Value::Long(l), BinaryOperator::Subtract, Value::Long(r)) => Value::Long(l - r),
                    (Value::Long(l), BinaryOperator::Multiply, Value::Long(r)) => Value::Long(l * r),
                    (Value::Long(l), BinaryOperator::Divide, Value::Long(r)) => {
                        if r == 0 {
                            panic!("除以零错误");
                        }
                        Value::Long(l / r)
                    },
                    (Value::Long(l), BinaryOperator::Modulo, Value::Long(r)) => {
                        if r == 0 {
                            panic!("除以零错误");
                        }
                        Value::Long(l % r)
                    },
                    
                    // 整数和长整型混合运算
                    (Value::Int(l), BinaryOperator::Add, Value::Long(r)) => Value::Long(l as i64 + r),
                    (Value::Long(l), BinaryOperator::Add, Value::Int(r)) => Value::Long(l + r as i64),
                    (Value::Int(l), BinaryOperator::Subtract, Value::Long(r)) => Value::Long(l as i64 - r),
                    (Value::Long(l), BinaryOperator::Subtract, Value::Int(r)) => Value::Long(l - r as i64),
                    (Value::Int(l), BinaryOperator::Multiply, Value::Long(r)) => Value::Long(l as i64 * r),
                    (Value::Long(l), BinaryOperator::Multiply, Value::Int(r)) => Value::Long(l * r as i64),
                    (Value::Int(l), BinaryOperator::Divide, Value::Long(r)) => {
                        if r == 0 {
                            panic!("除以零错误");
                        }
                        Value::Long(l as i64 / r)
                    },
                    (Value::Long(l), BinaryOperator::Divide, Value::Int(r)) => {
                        if r == 0 {
                            panic!("除以零错误");
                        }
                        Value::Long(l / r as i64)
                    },
                    
                    // 字符串连接
                    (Value::String(l), BinaryOperator::Add, Value::String(r)) => Value::String(l + &r),
                    
                    // 字符串和其他类型的连接
                    (Value::String(l), BinaryOperator::Add, Value::Int(r)) => Value::String(l + &r.to_string()),
                    (Value::String(l), BinaryOperator::Add, Value::Float(r)) => Value::String(l + &r.to_string()),
                    (Value::String(l), BinaryOperator::Add, Value::Bool(r)) => Value::String(l + &r.to_string()),
                    (Value::String(l), BinaryOperator::Add, Value::Long(r)) => Value::String(l + &r.to_string()),
                    
                    // 其他类型和字符串的连接
                    (Value::Int(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.to_string() + &r),
                    (Value::Float(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.to_string() + &r),
                    (Value::Bool(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.to_string() + &r),
                    (Value::Long(l), BinaryOperator::Add, Value::String(r)) => Value::String(l.to_string() + &r),
                    
                    // 不支持的操作
                    (l, op, r) => panic!("不支持的操作: {:?} {:?} {:?}", l, op, r),
                }
            }
        }
    }
} 