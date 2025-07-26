use std::collections::HashMap;
use std::fmt;
use crate::ast::{Parameter, Expression, Statement};

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
    Object(ObjectInstance), // 新增：对象实例
    Lambda(Vec<Parameter>, Expression), // Lambda表达式
    LambdaBlock(Vec<Parameter>, Vec<Statement>), // Lambda块
    FunctionReference(String), // 函数引用
    EnumValue(EnumInstance), // 新增：枚举实例
    None, // 表示空值或未定义的值
}

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
}

// 静态成员存储
#[derive(Debug, Clone)]
pub struct StaticMembers {
    pub static_fields: HashMap<String, Value>,
}

// 枚举实例
#[derive(Debug, Clone)]
pub struct EnumInstance {
    pub enum_name: String,
    pub variant_name: String,
    pub fields: Vec<Value>, // 枚举变体的字段值
}

impl Value {
    // 将Value转换为String，用于传递给库函数
    pub fn to_string(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Long(l) => l.to_string(),
            Value::Array(arr) => {
                let mut result = String::from("[");
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&val.to_string());
                }
                result.push(']');
                result
            },
            Value::Map(map) => {
                let mut result = String::from("{");
                for (i, (key, val)) in map.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&format!("\"{}\": {}", key, val.to_string()));
                }
                result.push('}');
                result
            },
            Value::Object(obj) => {
                format!("{}@{:p}", obj.class_name, obj)
            },
            Value::EnumValue(enum_val) => {
                if enum_val.fields.is_empty() {
                    format!("{}::{}", enum_val.enum_name, enum_val.variant_name)
                } else {
                    let field_strs: Vec<String> = enum_val.fields.iter().map(|f| f.to_string()).collect();
                    format!("{}::{}({})", enum_val.enum_name, enum_val.variant_name, field_strs.join(", "))
                }
            },
            Value::Lambda(params, _) => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                format!("lambda({})", param_names.join(", "))
            },
            Value::LambdaBlock(params, _) => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                format!("lambda_block({})", param_names.join(", "))
            },
            Value::FunctionReference(name) => {
                format!("function_ref({})", name)
            },
            Value::EnumValue(enum_val) => {
                if enum_val.fields.is_empty() {
                    format!("{}::{}", enum_val.enum_name, enum_val.variant_name)
                } else {
                    let field_strs: Vec<String> = enum_val.fields.iter().map(|f| f.to_string()).collect();
                    format!("{}::{}({})", enum_val.enum_name, enum_val.variant_name, field_strs.join(", "))
                }
            },
            Value::None => "null".to_string(),
        }
    }
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
            },
            Value::Object(obj) => write!(f, "{}@{:p}", obj.class_name, obj),
            Value::Lambda(params, _) => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                write!(f, "lambda({})", param_names.join(", "))
            },
            Value::LambdaBlock(params, _) => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                write!(f, "lambda_block({})", param_names.join(", "))
            },
            Value::FunctionReference(name) => write!(f, "function_ref({})", name),
            Value::EnumValue(enum_val) => {
                if enum_val.fields.is_empty() {
                    write!(f, "{}::{}", enum_val.enum_name, enum_val.variant_name)
                } else {
                    write!(f, "{}::{}(", enum_val.enum_name, enum_val.variant_name)?;
                    for (i, field) in enum_val.fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", field)?;
                    }
                    write!(f, ")")
                }
            },
            Value::None => write!(f, "null"),
        }
    }
} 