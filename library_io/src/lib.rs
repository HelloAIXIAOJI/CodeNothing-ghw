use std::collections::HashMap;
use std::io::{self, Write};
use std::fmt::Write as FmtWrite;

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

// 处理转义字符
fn process_escape_chars(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek().is_some() {
            match chars.next().unwrap() {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                c => {
                    result.push('\\');
                    result.push(c);
                }
            }
        } else {
            result.push(c);
        }
    }
    
    result
}

// 命名空间函数
mod std {
    use super::*;
    
    // 打印字符串到标准输出
    pub fn cn_print(args: Vec<String>) -> String {
        let mut output = String::new();
        for arg in args {
            let processed = process_escape_chars(&arg);
            print!("{}", processed);
            output.push_str(&processed);
        }
        io::stdout().flush().unwrap();
        output
    }
    
    // 打印字符串到标准输出，并添加换行符
    pub fn cn_println(args: Vec<String>) -> String {
        let mut output = String::new();
        for arg in args {
            let processed = process_escape_chars(&arg);
            println!("{}", processed);
            output.push_str(&processed);
        }
        output.push('\n');
        output
    }
    
    // 从标准输入读取一行
    pub fn cn_read_line(_args: Vec<String>) -> String {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // 移除末尾的换行符
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                input
            },
            Err(_) => String::new(),
        }
    }
    
    // 格式化打印，类似C语言的printf
    pub fn cn_printf(args: Vec<String>) -> String {
        if args.is_empty() {
            return String::new();
        }
        
        let format_str = process_escape_chars(&args[0]);
        let mut result = String::new();
        let mut format_args = args.iter().skip(1);
        let mut chars = format_str.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '%' {
                if let Some(&next_c) = chars.peek() {
                    match next_c {
                        's' => {
                            // 字符串格式化
                            if let Some(arg) = format_args.next() {
                                let processed = process_escape_chars(arg);
                                result.push_str(&processed);
                            } else {
                                result.push_str("%s");
                            }
                            chars.next(); // 消费's'
                        },
                        'd' | 'i' => {
                            // 整数格式化
                            if let Some(arg) = format_args.next() {
                                if let Ok(num) = arg.parse::<i32>() {
                                    let _ = write!(result, "{}", num);
                                } else {
                                    result.push_str("%d");
                                }
                            } else {
                                result.push_str("%d");
                            }
                            chars.next(); // 消费'd'或'i'
                        },
                        'f' => {
                            // 浮点数格式化
                            if let Some(arg) = format_args.next() {
                                if let Ok(num) = arg.parse::<f64>() {
                                    let _ = write!(result, "{}", num);
                                } else {
                                    result.push_str("%f");
                                }
                            } else {
                                result.push_str("%f");
                            }
                            chars.next(); // 消费'f'
                        },
                        '%' => {
                            // 转义的百分号
                            result.push('%');
                            chars.next(); // 消费第二个'%'
                        },
                        _ => {
                            // 未知格式说明符，保留原样
                            result.push('%');
                            result.push(next_c);
                            chars.next(); // 消费下一个字符
                        }
                    }
                } else {
                    // 格式字符串以%结尾，保留原样
                    result.push('%');
                }
            } else {
                // 普通字符，直接添加
                result.push(c);
            }
        }
        
        // 打印结果
        print!("{}", result);
        io::stdout().flush().unwrap();
        
        result
    }
}

// 初始化函数，返回函数映射
#[no_mangle]
pub extern "C" fn cn_init() -> *mut HashMap<String, LibraryFunction> {
    let mut functions = HashMap::new();
    
    // 使用命名空间构建器注册std命名空间下的函数
    let mut std_ns = NamespaceBuilder::new("std");
    std_ns.add_function("print", std::cn_print)
         .add_function("println", std::cn_println)
         .add_function("read_line", std::cn_read_line)
         .add_function("printf", std::cn_printf);
    
    // 注册所有函数到主函数映射
    std_ns.register_all(&mut functions);
    
    // 将HashMap装箱并转换为原始指针
    Box::into_raw(Box::new(functions))
} 