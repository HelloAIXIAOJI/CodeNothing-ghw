// CodeNothing v0.7.5 内存池感知的值类型
// 为解释器提供高效的内存管理

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// 内存池感知的值类型
#[derive(Debug, Clone)]
pub enum PoolValue {
    Int(i32),
    Long(i64),
    Float(f64),
    String(PoolString),
    Bool(bool),
    Array(PoolArray),
    Object(PoolObject),
    None,
}

impl PoolValue {
    /// 从标准Value转换为PoolValue
    pub fn from_value(value: Value) -> Self {
        match value {
            Value::Int(i) => PoolValue::Int(i),
            Value::Long(l) => PoolValue::Long(l),
            Value::Float(f) => PoolValue::Float(f),
            Value::String(s) => PoolValue::String(PoolString::new(s)),
            Value::Bool(b) => PoolValue::Bool(b),
            Value::Array(arr) => {
                let pool_arr = arr.into_iter()
                    .map(|v| PoolValue::from_value(v))
                    .collect();
                PoolValue::Array(PoolArray::new(pool_arr))
            },
            // 暂时简化Object处理
            Value::Object(_obj) => {
                PoolValue::Object(PoolObject::new(HashMap::new()))
            },
            Value::None => PoolValue::None,
            // 其他类型暂时转换为None
            _ => PoolValue::None,
        }
    }

    /// 转换为标准Value
    pub fn to_value(&self) -> Value {
        match self {
            PoolValue::Int(i) => Value::Int(*i),
            PoolValue::Long(l) => Value::Long(*l),
            PoolValue::Float(f) => Value::Float(*f),
            PoolValue::String(s) => Value::String(s.to_string()),
            PoolValue::Bool(b) => Value::Bool(*b),
            PoolValue::Array(arr) => {
                let std_arr = arr.iter()
                    .map(|v| v.to_value())
                    .collect();
                Value::Array(std_arr)
            },
            PoolValue::Object(_obj) => {
                // 暂时返回空的Map
                Value::Map(HashMap::new())
            },
            PoolValue::None => Value::None,
        }
    }

    /// 获取值的类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            PoolValue::Int(_) => "int",
            PoolValue::Long(_) => "long",
            PoolValue::Float(_) => "float",
            PoolValue::String(_) => "string",
            PoolValue::Bool(_) => "bool",
            PoolValue::Array(_) => "array",
            PoolValue::Object(_) => "object",
            PoolValue::None => "none",
        }
    }

    /// 检查是否为真值
    pub fn is_truthy(&self) -> bool {
        match self {
            PoolValue::Bool(b) => *b,
            PoolValue::Int(i) => *i != 0,
            PoolValue::Long(l) => *l != 0,
            PoolValue::Float(f) => *f != 0.0,
            PoolValue::String(s) => !s.is_empty(),
            PoolValue::Array(arr) => !arr.is_empty(),
            PoolValue::Object(obj) => !obj.is_empty(),
            PoolValue::None => false,
        }
    }
}

/// 内存池管理的字符串
#[derive(Debug, Clone)]
pub struct PoolString {
    data: String, // 暂时使用标准String，后续可以优化为池分配
}

impl PoolString {
    pub fn new(s: String) -> Self {
        Self { data: s }
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }

    pub fn to_string(&self) -> String {
        self.data.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

/// 内存池管理的数组
#[derive(Debug, Clone)]
pub struct PoolArray {
    data: Vec<PoolValue>, // 暂时使用标准Vec，后续可以优化为池分配
}

impl PoolArray {
    pub fn new(data: Vec<PoolValue>) -> Self {
        Self { data }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, value: PoolValue) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<PoolValue> {
        self.data.pop()
    }

    pub fn get(&self, index: usize) -> Option<&PoolValue> {
        self.data.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut PoolValue> {
        self.data.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<PoolValue> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<PoolValue> {
        self.data.iter_mut()
    }
}

/// 内存池管理的对象
#[derive(Debug, Clone)]
pub struct PoolObject {
    data: HashMap<String, PoolValue>, // 暂时使用标准HashMap，后续可以优化为池分配
}

impl PoolObject {
    pub fn new(data: HashMap<String, PoolValue>) -> Self {
        Self { data }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, key: String, value: PoolValue) -> Option<PoolValue> {
        self.data.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&PoolValue> {
        self.data.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut PoolValue> {
        self.data.get_mut(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<PoolValue> {
        self.data.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, PoolValue> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<String, PoolValue> {
        self.data.iter_mut()
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<String, PoolValue> {
        self.data.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<String, PoolValue> {
        self.data.values()
    }
}

/// 内存池感知的变量存储
pub struct PoolVariableStorage {
    variables: HashMap<String, PoolValue>,
    allocation_count: usize,
}

impl PoolVariableStorage {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            allocation_count: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            variables: HashMap::with_capacity(capacity),
            allocation_count: 0,
        }
    }

    pub fn set(&mut self, name: String, value: PoolValue) {
        if !self.variables.contains_key(&name) {
            self.allocation_count += 1;
        }
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&PoolValue> {
        self.variables.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut PoolValue> {
        self.variables.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<PoolValue> {
        self.variables.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.variables.len()
    }

    pub fn allocation_count(&self) -> usize {
        self.allocation_count
    }

    pub fn clear(&mut self) {
        self.variables.clear();
        self.allocation_count = 0;
    }
}

impl Default for PoolVariableStorage {
    fn default() -> Self {
        Self::new()
    }
}
