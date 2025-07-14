use ::std::collections::HashMap;

// 定义库函数类型
pub type LibraryFunction = fn(Vec<String>) -> String;

/// 命名空间构建器，用于简化库函数的命名空间注册
pub struct NamespaceBuilder {
    namespace: String,
    functions: HashMap<String, LibraryFunction>,
}

impl NamespaceBuilder {
    /// 创建一个新的命名空间构建器
    /// 
    /// # 参数
    /// * `namespace` - 命名空间名称
    pub fn new(namespace: &str) -> Self {
        NamespaceBuilder {
            namespace: namespace.to_string(),
            functions: HashMap::new(),
        }
    }
    
    /// 向命名空间中添加函数
    /// 
    /// # 参数
    /// * `name` - 函数名称（不含命名空间前缀）
    /// * `func` - 函数指针
    /// 
    /// # 返回
    /// 返回自身引用，支持链式调用
    pub fn add_function(&mut self, name: &str, func: LibraryFunction) -> &mut Self {
        let full_name = if self.namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.namespace, name)
        };
        self.functions.insert(full_name, func);
        self
    }
    
    /// 将命名空间中的所有函数注册到目标HashMap
    /// 
    /// # 参数
    /// * `target` - 目标函数映射
    pub fn register_all(&self, target: &mut HashMap<String, LibraryFunction>) {
        for (name, func) in &self.functions {
            target.insert(name.clone(), *func);
        }
    }
    
    /// 获取命名空间名称
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
    
    /// 获取已注册函数数量
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }
    
    /// 检查函数是否已注册
    pub fn has_function(&self, name: &str) -> bool {
        let full_name = if self.namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.namespace, name)
        };
        self.functions.contains_key(&full_name)
    }
}

/// 创建并初始化多个命名空间
/// 
/// # 参数
/// * `namespaces` - 命名空间名称和函数的映射
/// 
/// # 返回
/// 返回合并后的函数映射
pub fn register_namespaces(namespaces: Vec<(&str, Vec<(&str, LibraryFunction)>)>) -> HashMap<String, LibraryFunction> {
    let mut all_functions = HashMap::new();
    
    for (namespace, functions) in namespaces {
        let mut ns_builder = NamespaceBuilder::new(namespace);
        for (name, func) in functions {
            ns_builder.add_function(name, func);
        }
        ns_builder.register_all(&mut all_functions);
    }
    
    all_functions
}

/// 创建指向函数映射的原始指针，用于库初始化
/// 
/// # 参数
/// * `functions` - 函数映射
/// 
/// # 返回
/// 返回函数映射的原始指针
pub fn create_library_pointer(functions: HashMap<String, LibraryFunction>) -> *mut HashMap<String, LibraryFunction> {
    Box::into_raw(Box::new(functions))
} 