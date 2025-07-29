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
    Pointer(PointerInstance), // 新增：指针实例
    FunctionPointer(FunctionPointerInstance), // 新增：函数指针实例
    LambdaFunctionPointer(LambdaFunctionPointerInstance), // 新增：Lambda函数指针实例
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

// 指针实例
#[derive(Debug, Clone)]
pub struct PointerInstance {
    pub address: usize, // 真实内存地址
    pub target_type: PointerType, // 指向的类型
    pub is_null: bool, // 是否为空指针
    pub level: usize, // 指针级别（1=*int, 2=**int, 等）
}

// 指针类型信息
#[derive(Debug, Clone)]
pub enum PointerType {
    Int,
    Float,
    Bool,
    String,
    Long,
    Enum(String),
    Class(String),
    Function(Vec<crate::ast::Type>, Box<crate::ast::Type>), // 函数指针
    Pointer(Box<PointerType>), // 多级指针
}

// 函数指针实例
#[derive(Debug, Clone)]
pub struct FunctionPointerInstance {
    pub function_name: String, // 函数名
    pub param_types: Vec<crate::ast::Type>, // 参数类型
    pub return_type: Box<crate::ast::Type>, // 返回类型
    pub is_null: bool, // 是否为空
    pub is_lambda: bool, // 是否为Lambda表达式
    pub lambda_body: Option<Box<crate::ast::Statement>>, // Lambda函数体
}

// Lambda函数指针实例（包含完整参数信息）
#[derive(Debug, Clone)]
pub struct LambdaFunctionPointerInstance {
    pub function_name: String, // 函数名
    pub param_types: Vec<crate::ast::Type>, // 参数类型
    pub return_type: Box<crate::ast::Type>, // 返回类型
    pub is_null: bool, // 是否为空
    pub is_lambda: bool, // 是否为Lambda表达式
    pub lambda_body: Option<Box<crate::ast::Statement>>, // Lambda函数体
    pub lambda_params: Vec<crate::ast::Parameter>, // 完整的参数信息（包含名称）
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
            Value::Pointer(ptr) => {
                if ptr.is_null {
                    "null".to_string()
                } else {
                    let stars = "*".repeat(ptr.level);
                    format!("{}0x{:x}", stars, ptr.address)
                }
            },
            Value::FunctionPointer(func_ptr) => {
                if func_ptr.is_null {
                    "null".to_string()
                } else if func_ptr.is_lambda {
                    format!("*fn(lambda) : {}", Value::type_to_string(&func_ptr.return_type))
                } else {
                    format!("*fn({}) : {}", func_ptr.function_name, Value::type_to_string(&func_ptr.return_type))
                }
            },
            Value::LambdaFunctionPointer(lambda_ptr) => {
                if lambda_ptr.is_null {
                    "null".to_string()
                } else {
                    let param_strs: Vec<String> = lambda_ptr.param_types.iter()
                        .map(|t| Value::type_to_string(t))
                        .collect();
                    format!("*fn({}) : {}", param_strs.join(", "), Value::type_to_string(&lambda_ptr.return_type))
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
            Value::Pointer(ptr) => {
                if ptr.is_null {
                    write!(f, "null")
                } else {
                    let stars = "*".repeat(ptr.level);
                    write!(f, "{}0x{:x}", stars, ptr.address)
                }
            },
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
            Value::FunctionPointer(func_ptr) => {
                if func_ptr.is_null {
                    write!(f, "null")
                } else if func_ptr.is_lambda {
                    write!(f, "*fn(lambda)")
                } else {
                    write!(f, "*fn({})", func_ptr.function_name)
                }
            },
            Value::LambdaFunctionPointer(lambda_ptr) => {
                if lambda_ptr.is_null {
                    write!(f, "null")
                } else {
                    write!(f, "*fn(lambda)")
                }
            },
            Value::None => write!(f, "null"),
        }
    }
}

impl Value {
    // 辅助方法：将类型转换为字符串
    pub fn type_to_string(type_ref: &crate::ast::Type) -> String {
        match type_ref {
            crate::ast::Type::Int => "int".to_string(),
            crate::ast::Type::Float => "float".to_string(),
            crate::ast::Type::Bool => "bool".to_string(),
            crate::ast::Type::String => "string".to_string(),
            crate::ast::Type::Long => "long".to_string(),
            crate::ast::Type::Void => "void".to_string(),
            crate::ast::Type::Class(name) => name.clone(),
            crate::ast::Type::Array(inner) => format!("[]{}", Self::type_to_string(inner)),
            crate::ast::Type::Pointer(inner) => format!("*{}", Self::type_to_string(inner)),
            crate::ast::Type::OptionalPointer(inner) => format!("?*{}", Self::type_to_string(inner)),
            crate::ast::Type::FunctionPointer(params, ret) => {
                let param_strs: Vec<String> = params.iter().map(|p| Self::type_to_string(p)).collect();
                format!("*fn({}) : {}", param_strs.join(", "), Self::type_to_string(ret))
            },
            _ => "unknown".to_string(),
        }
    }
}