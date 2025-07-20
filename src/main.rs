use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

mod ast;
mod parser;
mod interpreter;
use interpreter::jit;

use ast::Program;
use interpreter::value::Value;

// 文件预处理器，处理文件导入
struct FilePreprocessor {
    processed_files: HashMap<String, String>,
    file_stack: Vec<String>,
}

impl FilePreprocessor {
    fn new() -> Self {
        FilePreprocessor {
            processed_files: HashMap::new(),
            file_stack: Vec::new(),
        }
    }
    
    // 处理文件，包括导入处理
    fn process_file(&mut self, file_path: &str, current_dir: Option<&Path>) -> Result<String, String> {
        // 规范化文件路径
        let full_path = if Path::new(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else if let Some(dir) = current_dir {
            dir.join(file_path)
        } else {
            PathBuf::from(file_path)
        };
        
        let canonical_path = match full_path.canonicalize() {
            Ok(path) => path,
            Err(_) => return Err(format!("无法找到文件: {}", file_path)),
        };
        
        let canonical_path_str = canonical_path.to_string_lossy().to_string();
        
        // 检查是否已处理过该文件
        if let Some(content) = self.processed_files.get(&canonical_path_str) {
            return Ok(content.clone());
        }
        
        // 检查循环导入
        if self.file_stack.contains(&canonical_path_str) {
            return Err(format!("检测到循环导入: {}", file_path));
        }
        
        // 读取文件内容
        let content = read_file(&canonical_path_str)?;
        
        // 将当前文件添加到处理栈中
        self.file_stack.push(canonical_path_str.clone());
        
        // 处理文件内容
        let processed_content = content;
        
        // 将处理结果存储到缓存中
        self.processed_files.insert(canonical_path_str.clone(), processed_content.clone());
        
        // 从处理栈中移除当前文件
        self.file_stack.pop();
        
        Ok(processed_content)
    }
}

// 读取文件内容
fn read_file(file_path: &str) -> Result<String, String> {
    let mut file = File::open(file_path)
        .map_err(|err| format!("无法打开文件: {}", err))?;
    
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|err| format!("无法读取文件: {}", err))?;
    
    Ok(content)
}

// 添加调试打印函数
fn debug_println(msg: &str) {
    if env::args().any(|arg| arg == "--cn-debug") {
        println!("{}", msg);
    }
}

fn init_program() -> Program {
    Program {
        functions: Vec::new(),
        namespaces: Vec::new(),
        imported_namespaces: Vec::new(),
        file_imports: Vec::new(),
        constants: Vec::new(), // 初始化常量列表
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("用法: {} <文件路径> [--cn-parser] [--cn-debug]", args[0]);
        return;
    }
    
    let file_path = &args[1];
    let debug_parser = args.iter().any(|arg| arg == "--cn-parser");
    let debug_lexer = args.iter().any(|arg| arg == "--cn-lexer");
    let debug_mode = args.iter().any(|arg| arg == "--cn-debug");
    let show_return = args.iter().any(|arg| arg == "--cn-return");
    let query_jit = args.iter().any(|arg| arg == "--cn-query-jit");
    
    // 如果是调试模式，先调试io库中的函数
    if debug_mode {
        match interpreter::library_loader::debug_library_functions("io") {
            Ok(_) => {},
            Err(err) => {
                println!("调试io库函数失败: {}", err);
            }
        }
    }
    
    // 创建文件预处理器
    let mut preprocessor = FilePreprocessor::new();
    
    // 获取文件所在目录
    let file_dir = Path::new(file_path).parent();
    
    // 预处理文件，处理所有导入
    match preprocessor.process_file(file_path, file_dir) {
        Ok(processed_content) => {
            debug_println(&format!("预处理后的文件内容:\n{}", processed_content));
            
            // 添加调试信息，查看注释移除后的代码
            if debug_lexer {
                let content_without_comments = parser::lexer::remove_comments(&processed_content);
                println!("移除注释后的代码:\n{}", content_without_comments);
            }
            
            // 输出所有的tokens，帮助调试
            if debug_parser {
                let tokens = parser::lexer::tokenize(&parser::lexer::remove_comments(&processed_content), true);
                println!("\n所有tokens:");
                for (i, token) in tokens.iter().enumerate() {
                    println!("{}: '{}'", i, token);
                }
                println!("");
            }
            
            // 修改为收集所有错误
            let parse_result = parser::parse_all_errors(&processed_content, debug_parser);
            match parse_result {
                Ok((program, warnings)) => {
                    // 显示警告信息
                    if !warnings.is_empty() {
                        println!("解析警告:");
                        for (i, warning) in warnings.iter().enumerate() {
                            println!("警告 {}: {}", i+1, warning);
                        }
                        println!("");
                    }
                    
                    // 执行程序
                    let result = interpreter::interpret(&program);
                    
                    // 只有当结果不是None且启用了--cn-return参数时才打印
                    if show_return && !matches!(result, Value::None) {
                        println!("程序执行结果: {}", result);
                    }
                    if query_jit && jit::was_jit_used() {
                        print!("{}", jit::jit_stats());
                    }
                },
                Err(errors) => {
                    // 显示所有错误信息
                    println!("发现 {} 个解析错误:", errors.len());
                    
                    // 简单直接地显示错误
                    for (i, error) in errors.iter().enumerate() {
                        // 提取错误消息，忽略位置信息
                        let error_msg = if let Some(pos_start) = error.find("(位置:") {
                            error[0..pos_start].trim()
                        } else {
                            error.as_str()
                        };
                        
                        println!("错误 {}: {}", i+1, error_msg);
                    }
                    
                    println!("\n可以使用 --cn-parser 选项查看更详细的解析信息。");
                    println!("由于存在解析错误，程序无法执行。");
                }
            }
        },
        Err(err) => println!("预处理文件错误: {}", err),
    }
}