use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::env;

mod parser;
mod ast;
mod interpreter;

// 添加调试模式检查函数
fn is_debug_mode() -> bool {
    env::args().any(|arg| arg == "--cn-debug")
}

// 添加条件打印函数
fn debug_println(msg: &str) {
    if is_debug_mode() {
        println!("{}", msg);
    }
}

// 文件预处理器，处理文件导入
struct FilePreprocessor {
    // 已处理的文件路径集合，用于检测循环依赖
    processed_files: HashSet<PathBuf>,
    // 当前工作目录，用于解析相对路径
    current_dir: PathBuf,
}

impl FilePreprocessor {
    fn new() -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            processed_files: HashSet::new(),
            current_dir,
        }
    }
    
    // 解析文件路径（支持相对路径）
    fn resolve_path(&self, file_path: &str, base_dir: Option<&Path>) -> PathBuf {
        let path = Path::new(file_path);
        
        // 如果是绝对路径，直接返回
        if path.is_absolute() {
            return path.to_path_buf();
        }
        
        // 如果是相对路径，先尝试相对于基目录解析
        if let Some(base) = base_dir {
            let resolved = base.join(path);
            if resolved.exists() {
                return resolved;
            }
        }
        
        // 如果是examples目录下的文件，尝试从examples目录解析
        let examples_path = self.current_dir.join("examples").join(file_path);
        if examples_path.exists() {
            return examples_path;
        }
        
        // 最后尝试相对于当前工作目录解析
        self.current_dir.join(path)
    }
    
    // 处理文件，提取并处理所有导入语句
    fn process_file(&mut self, file_path: &str, base_dir: Option<&Path>) -> Result<String, String> {
        // 解析文件路径
        let resolved_path = self.resolve_path(file_path, base_dir);
        
        debug_println(&format!("处理文件: {}", resolved_path.display()));
        
        // 检查文件是否存在
        if !resolved_path.exists() {
            return Err(format!("文件不存在: {}", resolved_path.display()));
        }
        
        // 获取文件所在目录，用于后续导入相对路径
        let file_dir = resolved_path.parent();
        
        // 检查循环依赖
        let canonical_path = resolved_path.canonicalize()
            .map_err(|e| format!("无法获取文件 '{}' 的规范路径: {}", resolved_path.display(), e))?;
        
        if self.processed_files.contains(&canonical_path) {
            // 文件已经处理过，检测到循环依赖
            return Err(format!("检测到循环依赖: 文件 '{}' 已经被导入", resolved_path.display()));
        }
        
        // 添加到已处理文件集合
        self.processed_files.insert(canonical_path);
        
        // 读取文件内容
        let content = fs::read_to_string(&resolved_path)
            .map_err(|e| format!("无法读取文件 '{}': {}", resolved_path.display(), e))?;
        
        debug_println(&format!("成功读取文件: {}", resolved_path.display()));
        
        // 处理文件内容，提取导入语句
        let mut processed_content = String::new();
        let mut lines = content.lines();
        
        while let Some(line) = lines.next() {
            // 检查是否是文件导入语句
            if line.trim().starts_with("using file") {
                // 提取文件路径
                let start = line.find('"');
                let end = line.rfind('"');
                
                if let (Some(start), Some(end)) = (start, end) {
                    let imported_file = &line[start + 1..end];
                    
                    // 递归处理导入的文件
                    match self.process_file(imported_file, file_dir) {
                        Ok(imported_content) => {
                            // 添加导入的文件内容
                            processed_content.push_str(&imported_content);
                            processed_content.push_str("\n");
                        },
                        Err(err) => {
                            return Err(format!("处理导入文件 '{}' 时出错: {}", imported_file, err));
                        }
                    }
                }
            } else {
                // 保留非导入语句
                processed_content.push_str(line);
                processed_content.push_str("\n");
            }
        }
        
        Ok(processed_content)
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
                    println!("程序执行结果: {}", result);
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