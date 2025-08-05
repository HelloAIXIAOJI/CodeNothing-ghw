use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::Instant;

mod ast;
mod parser;
mod interpreter;
mod analyzer;
mod debug_config;
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
        enums: Vec::new(), // 初始化枚举列表
    }
}

// 格式化执行时间
fn format_execution_time(duration_ms: f64) -> String {
    if duration_ms < 1000.0 {
        format!("{:.3} ms", duration_ms)
    } else if duration_ms < 60000.0 {
        let seconds = duration_ms / 1000.0;
        format!("{:.3} ms [{:.1} s]", duration_ms, seconds)
    } else {
        let total_seconds = duration_ms / 1000.0;
        let minutes = (total_seconds / 60.0).floor();
        let seconds = total_seconds % 60.0;
        format!("{:.3} ms [{:.0} min {:.1} s]", duration_ms, minutes, seconds)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("用法: {} <文件路径> [选项]", args[0]);
        println!("");
        println!("传统选项:");
        println!("  --cn-parser     显示详细的解析信息");
        println!("  --cn-lexer      显示词法分析信息");
        println!("  --cn-debug      启用调试模式");
        println!("  --cn-return     显示程序执行结果");
        println!("  --cn-query-jit  显示JIT编译统计信息");
        println!("  --cn-jit-debug  显示JIT编译调试信息");
        println!("  --cn-jit-stats  显示JIT性能统计报告");
        println!("  --cn-time       显示程序执行时间");
        println!("  --cn-rwlock     🚀 v0.6.2 显示读写锁性能统计");
        println!("");
        println!("🆕 v0.7.4 细粒度调试选项:");
        debug_config::print_debug_help();
        println!("");
        println!("示例:");
        println!("  {} hello.cn", args[0]);
        println!("  {} hello.cn --cn-time", args[0]);
        println!("  {} hello.cn --debug-jit", args[0]);
        println!("  {} hello.cn --debug-lifetime --cn-time", args[0]);
        return;
    }

    // v0.7.4新增：初始化调试配置
    debug_config::init_debug_config(&args);

    let file_path = &args[1];
    let debug_parser = args.iter().any(|arg| arg == "--cn-parser");
    let debug_lexer = args.iter().any(|arg| arg == "--cn-lexer");
    let debug_mode = args.iter().any(|arg| arg == "--cn-debug");
    let show_return = args.iter().any(|arg| arg == "--cn-return");
    let query_jit = args.iter().any(|arg| arg == "--cn-query-jit");
    let jit_debug = args.iter().any(|arg| arg == "--cn-jit-debug");
    let jit_stats = args.iter().any(|arg| arg == "--cn-jit-stats");
    let show_time = args.iter().any(|arg| arg == "--cn-time");
    let show_rwlock = args.iter().any(|arg| arg == "--cn-rwlock");
    
    // 初始化JIT编译器
    interpreter::jit::init_jit(jit_debug);

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

    // 开始计时（如果启用了时间显示）
    let start_time = if show_time { Some(Instant::now()) } else { None };

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

                    // 进行类型检查
                    let mut type_checker = analyzer::TypeChecker::new();
                    match type_checker.check_program(&program) {
                        Ok(()) => {
                            if debug_mode {
                                println!("✓ 类型检查通过");
                            }
                        },
                        Err(type_errors) => {
                            println!("发现 {} 个类型错误:", type_errors.len());
                            for (i, error) in type_errors.iter().enumerate() {
                                if let (Some(line), Some(column)) = (error.line, error.column) {
                                    println!("类型错误 {}: {} (行 {}, 列 {})", i+1, error.message, line, column);
                                } else {
                                    println!("类型错误 {}: {}", i+1, error.message);
                                }
                            }
                            println!("");
                            println!("由于存在类型错误，程序无法执行。");

                            // 显示执行时间（如果启用了时间显示）
                            if let Some(start) = start_time {
                                let duration = start.elapsed();
                                let duration_ms = duration.as_secs_f64() * 1000.0;
                                println!("类型检查时间: {}", format_execution_time(duration_ms));
                            }
                            return;
                        }
                    }

                    // 执行程序
                    let result = interpreter::interpret(&program);

                    // 只有当结果不是None且启用了--cn-return参数时才打印
                    if show_return && !matches!(result, Value::None) {
                        println!("程序执行结果: {}", result);
                    }

                    // JIT统计信息显示
                    if query_jit && jit::was_jit_used() {
                        print!("{}", jit::jit_stats());
                    }

                    // 显示JIT性能报告（通过命令行参数控制）
                    if jit_stats {
                        jit::print_jit_performance_report();
                    }

                    // 🚀 v0.6.2 显示读写锁性能统计（如果启用了--cn-rwlock参数）
                    if show_rwlock {
                        interpreter::memory_manager::print_rwlock_performance_stats();
                    }

                    // 显示执行时间（如果启用了时间显示）
                    if let Some(start) = start_time {
                        let duration = start.elapsed();
                        let duration_ms = duration.as_secs_f64() * 1000.0;
                        println!("执行时间: {}", format_execution_time(duration_ms));
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

                    // 显示执行时间（如果启用了时间显示）
                    if let Some(start) = start_time {
                        let duration = start.elapsed();
                        let duration_ms = duration.as_secs_f64() * 1000.0;
                        println!("解析时间: {}", format_execution_time(duration_ms));
                    }
                }
            }
        },
        Err(err) => println!("预处理文件错误: {}", err),
    }
}