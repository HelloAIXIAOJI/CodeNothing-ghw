use std::collections::HashMap;

// 定义库函数类型
type LibraryFunction = fn(Vec<String>) -> String;

// 命名空间辅助结构体和函数
struct NamespaceBuilder {
    namespace: String,
    functions: HashMap<String, LibraryFunction>,
}

impl NamespaceBuilder {
    fn new(namespace: &str) -> Self {
        NamespaceBuilder {
            namespace: namespace.to_string(),
            functions: HashMap::new(),
        }
    }
    
    fn add_function(&mut self, name: &str, func: LibraryFunction) -> &mut Self {
        let full_name = if self.namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.namespace, name)
        };
        self.functions.insert(full_name, func);
        self
    }
    
    fn register_all(&self, target: &mut HashMap<String, LibraryFunction>) {
        for (name, func) in &self.functions {
            target.insert(name.clone(), *func);
        }
    }
}

// 根命名空间函数
// 示例函数：将输入字符串反转并返回
fn cn_reverse(args: Vec<String>) -> String {
    if args.is_empty() {
        return String::new();
    }
    
    let input = &args[0];
    input.chars().rev().collect()
}

// 字符串操作命名空间
mod string {
    use super::*;
    
    // 示例函数：计算字符串长度
    pub fn cn_length(args: Vec<String>) -> String {
        if args.is_empty() {
            return "0".to_string();
        }
        
        args[0].len().to_string()
    }
    
    // 示例函数：将字符串转换为大写
    pub fn cn_to_upper(args: Vec<String>) -> String {
        if args.is_empty() {
            return String::new();
        }
        
        args[0].to_uppercase()
    }
    
    // 示例函数：将字符串转换为小写
    pub fn cn_to_lower(args: Vec<String>) -> String {
        if args.is_empty() {
            return String::new();
        }
        
        args[0].to_lowercase()
    }
}

// 初始化函数，返回函数映射
#[no_mangle]
pub extern "C" fn cn_init() -> *mut HashMap<String, LibraryFunction> {
    let mut functions = HashMap::new();
    
    // 注册根命名空间函数
    functions.insert("reverse".to_string(), cn_reverse as LibraryFunction);
    
    // 使用命名空间构建器注册string命名空间下的函数
    let mut string_ns = NamespaceBuilder::new("string");
    string_ns.add_function("length", string::cn_length)
             .add_function("to_upper", string::cn_to_upper)
             .add_function("to_lower", string::cn_to_lower);
    
    // 注册所有函数到主函数映射
    string_ns.register_all(&mut functions);
    
    // 将HashMap装箱并转换为原始指针
    Box::into_raw(Box::new(functions))
}

/* 
 * CodeNothing 动态库模板
 * 
 * 要创建自己的库，请遵循以下步骤：
 * 
 * 1. 定义你的函数，函数签名必须是 fn(Vec<String>) -> String
 *    例如：
 *    fn my_function(args: Vec<String>) -> String {
 *        // 处理参数并返回结果
 *        "结果".to_string()
 *    }
 * 
 * 2. 创建命名空间（可选）：
 *    mod my_namespace {
 *        use super::*;
 *        
 *        pub fn my_ns_function(args: Vec<String>) -> String {
 *            // 函数实现
 *            "命名空间函数结果".to_string()
 *        }
 *    }
 *
 * 3. 在 cn_init 函数中注册你的函数：
 *    // 直接注册根命名空间函数
 *    functions.insert("function_name".to_string(), my_function as LibraryFunction);
 *    
 *    // 使用命名空间构建器注册命名空间函数
 *    let mut ns = NamespaceBuilder::new("my_namespace");
 *    ns.add_function("my_function", my_namespace::my_ns_function);
 *    ns.register_all(&mut functions);
 * 
 * 4. 编译库：
 *    cargo build --release
 * 
 * 5. 在 CodeNothing 中使用：
 *    using lib <your_library_name>;
 *    
 *    fn main() : int {
 *        // 使用根命名空间函数
 *        result1 : string = function_name("参数");
 *        
 *        // 使用命名空间函数
 *        result2 : string = my_namespace::my_function("参数");
 *        
 *        // 或者导入命名空间后使用
 *        using ns my_namespace;
 *        result3 : string = my_function("参数");
 *        
 *        return 0;
 *    };
 */ 