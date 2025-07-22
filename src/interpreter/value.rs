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
    Object(ObjectInstance), // 新增：对象实例
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
            Value::None => write!(f, "null"),
        }
    }
} 