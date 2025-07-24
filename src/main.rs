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
            // 相对于当前工作目录
            std::env::current_dir()
                .map_err(|_| "无法获取当前工作目录".to_string())?
                .join(file_path)
        };
        
        // 检查文件是否存在
        if !full_path.exists() {
            return Err(format!("无法找到文件: {} (完整路径: {})", file_path, full_path.display()));
        }
        
        let canonical_path = match full_path.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                // 如果canonicalize失败，直接使用full_path
                full_path
            }
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
        
        // 递归处理导入的文件
        let processed_content = self.process_imports_in_content(&content, canonical_path.parent())?;
        
        // 将处理结果存储到缓存中
        self.processed_files.insert(canonical_path_str.clone(), processed_content.clone());
        
        // 从处理栈中移除当前文件
        self.file_stack.pop();
        
        Ok(processed_content)
    }
    
    // 处理内容中的导入语句
    fn process_imports_in_content(&mut self, content: &str, current_dir: Option<&Path>) -> Result<String, String> {
        let mut result = String::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            let trimmed = line.trim();
            
            // 检查是否是 using file 语句
            if trimmed.starts_with("using file") && trimmed.ends_with(";") {
                // 提取文件路径
                let start = trimmed.find('"').or_else(|| trimmed.find('\''));
                let end = trimmed.rfind('"').or_else(|| trimmed.rfind('\''));
                
                if let (Some(start), Some(end)) = (start, end) {
                    if start < end {
                        let import_path = &trimmed[start + 1..end];
                        
                        // 递归处理导入的文件
                        match self.process_file(import_path, current_dir) {
                            Ok(imported_content) => {
                                // 将导入的内容添加到结果中
                                result.push_str(&format!("// === 导入文件: {} ===\n", import_path));
                                result.push_str(&imported_content);
                                result.push_str(&format!("\n// === 结束导入: {} ===\n", import_path));
                            },
                            Err(err) => {
                                return Err(format!("导入文件 '{}' 失败: {}", import_path, err));
                            }
                        }
                    }
                }
                // 不将 using file 语句本身添加到结果中
            } else {
                // 保留其他所有行
                result.push_str(line);
                result.push('\n');
            }
        }
        
        Ok(result)
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
        classes: Vec::new(), // 初始化类列表
        interfaces: Vec::new(), // 初始化接口列表
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
    
    // 预处理文件，处理所有导入（不传递父目录，让process_file自己处理相对路径）
    match preprocessor.process_file(file_path, None) {
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