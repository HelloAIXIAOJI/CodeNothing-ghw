use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::env;
use std::vec::Vec;

mod parser;
mod ast;
mod interpreter;

// 添加调试模式检查函数
fn is_debug_mode() -> bool {
    env::args().any(|arg| arg == "--cn-debug")
}

// 添加全部错误模式检查函数
fn is_all_error_mode() -> bool {
    env::args().any(|arg| arg == "--cn-allerror")
}

// 添加条件打印函数
fn debug_println(msg: &str) {
    if is_debug_mode() {
        println!("{}", msg);
    }
}

// 添加错误处理辅助函数
fn display_error_context(source: &str, error_msg: &str, line: usize, col: usize) {
    // 显示错误位置的上下文（前后各显示1行）
    let lines: Vec<&str> = source.lines().collect();
    let line_count = lines.len();
    
    println!("\n错误详情:");
    println!("-------------------------");
    
    // 显示错误前一行（如果有）
    if line > 1 && line <= line_count {
        println!("{:4} | {}", line-1, lines[line-2]);
    }
    
    // 显示错误行并标记错误位置
    if line > 0 && line <= line_count {
        println!("{:4} | {}", line, lines[line-1]);
        println!("     | {}^", " ".repeat(col.saturating_sub(1)));
    }
    
    // 显示错误消息
    println!("     | \x1b[31m错误: {}\x1b[0m", error_msg);
    
    // 显示错误后一行（如果有）
    if line < line_count {
        println!("{:4} | {}", line+1, lines[line]);
    }
    
    println!("-------------------------");
}

// 添加错误类型分析函数
fn analyze_error(err: &str) -> (&str, Option<String>) {
    // 分析错误类型并提供可能的修复建议
    if err.contains("期望") && err.contains("但得到了") {
        return ("语法错误", Some("检查语法是否正确，可能缺少分号、括号或其他语法元素".to_string()));
    } else if err.contains("未定义的变量") || err.contains("未找到变量") {
        return ("变量错误", Some("确保变量在使用前已声明".to_string()));
    } else if err.contains("类型不匹配") || err.contains("类型必须是") {
        return ("类型错误", Some("检查变量类型是否与操作兼容".to_string()));
    } else if err.contains("检测到循环依赖") {
        return ("导入错误", Some("检查文件导入结构，消除循环依赖".to_string()));
    } else if err.contains("无法读取文件") || err.contains("文件不存在") {
        return ("文件错误", Some("确保文件路径正确且文件存在".to_string()));
    } else if err.contains("未定义的函数") || err.contains("未找到函数") {
        return ("函数错误", Some("确保函数在使用前已定义".to_string()));
    } else if err.contains("参数数量不匹配") || err.contains("参数类型不匹配") {
        return ("参数错误", Some("检查函数调用的参数数量和类型是否正确".to_string()));
    } else if err.contains("无法加载库") || err.contains("调用库函数失败") {
        return ("库错误", Some("确保库文件存在且函数调用正确".to_string()));
    } else {
        return ("一般错误", None);
    }
}

// 从错误信息中提取文件路径
fn extract_file_path_from_error(err: &str) -> Option<&str> {
    if let Some(start) = err.find("'") {
        if let Some(end) = err[start+1..].find("'") {
            return Some(&err[start+1..start+1+end]);
        }
    }
    None
}

// 错误收集结构体
#[derive(Debug)]
struct ErrorCollection {
    parse_errors: Vec<parser::ParseError>,
    interpreter_errors: Vec<interpreter::InterpreterError>,
    preprocessing_errors: Vec<String>,
}

impl ErrorCollection {
    fn new() -> Self {
        ErrorCollection {
            parse_errors: Vec::new(),
            interpreter_errors: Vec::new(),
            preprocessing_errors: Vec::new(),
        }
    }
    
    fn has_errors(&self) -> bool {
        !self.parse_errors.is_empty() || !self.interpreter_errors.is_empty() || !self.preprocessing_errors.is_empty()
    }
    
    fn display_all_errors(&self, source: &str) {
        if !self.parse_errors.is_empty() {
            println!("\n\x1b[31m发现 {} 个解析错误:\x1b[0m", self.parse_errors.len());
            for (i, err) in self.parse_errors.iter().enumerate() {
                println!("\n错误 #{}: {}", i+1, err.message);
                display_error_context(source, &err.message, err.line, err.column);
                
                let (error_type, suggestion) = analyze_error(&err.message);
                println!("\n错误类型: {}", error_type);
                if let Some(suggest) = suggestion {
                    println!("修复建议: {}", suggest);
                }
            }
        }
        
        if !self.interpreter_errors.is_empty() {
            println!("\n\x1b[31m发现 {} 个解释器错误:\x1b[0m", self.interpreter_errors.len());
            for (i, err) in self.interpreter_errors.iter().enumerate() {
                println!("\n错误 #{}: {}", i+1, err.message);
                
                if let Some(pos) = &err.position {
                    display_error_context(source, &err.message, pos.line, pos.column);
                }
                
                let (error_type, suggestion) = analyze_error(&err.message);
                println!("\n错误类型: {}", error_type);
                if let Some(suggest) = suggestion {
                    println!("修复建议: {}", suggest);
                }
            }
        }
        
        if !self.preprocessing_errors.is_empty() {
            println!("\n\x1b[31m发现 {} 个预处理错误:\x1b[0m", self.preprocessing_errors.len());
            for (i, err) in self.preprocessing_errors.iter().enumerate() {
                println!("\n错误 #{}: {}", i+1, err);
                
                if let Some(file_path) = extract_file_path_from_error(err) {
                    println!("出错的文件: {}", file_path);
                }
                
                let (error_type, suggestion) = analyze_error(err);
                println!("\n错误类型: {}", error_type);
                if let Some(suggest) = suggestion {
                    println!("修复建议: {}", suggest);
                }
            }
        }
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
        println!("用法: {} <文件路径> [--cn-parser] [--cn-debug] [--cn-allerror]", args[0]);
        return;
    }
    
    let file_path = &args[1];
    let debug_parser = args.iter().any(|arg| arg == "--cn-parser");
    let debug_lexer = args.iter().any(|arg| arg == "--cn-lexer");
    let all_error_mode = is_all_error_mode();
    
    // 创建文件预处理器
    let mut preprocessor = FilePreprocessor::new();
    
    // 获取文件所在目录
    let file_dir = Path::new(file_path).parent();
    
    // 创建错误收集器
    let mut error_collection = ErrorCollection::new();
    
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
            
            let ast_result = parser::parse(&processed_content, debug_parser);
            
            // 全部错误模式下，即使解析失败也尝试收集其他错误
            if all_error_mode && ast_result.is_err() {
                if let Err(err) = ast_result {
                    error_collection.parse_errors.push(err);
                }
                
                // 尝试解析可能的其他错误
                // 这里可以添加更多的错误检查逻辑
                
                // 显示所有收集到的错误
                error_collection.display_all_errors(&processed_content);
                return;
            }
            
            match ast_result {
                Ok(program) => {
                    let interpret_result = interpreter::interpret(&program);
                    
                    if all_error_mode && interpret_result.is_err() {
                        if let Err(err) = interpret_result {
                            error_collection.interpreter_errors.push(err);
                            
                            // 尝试收集更多解释器错误
                            // 这里可以添加更多的错误检查逻辑
                            
                            // 显示所有收集到的错误
                            error_collection.display_all_errors(&processed_content);
                            return;
                        }
                    }
                    
                    match interpret_result {
                        Ok(result) => println!("程序执行结果: {}", result),
                        Err(err) => {
                            // 增强解释器错误报告
                            println!("\n\x1b[31m解释器错误:\x1b[0m {}", err.message);
                            
                            // 显示错误位置信息
                            if let Some(pos) = err.position {
                                display_error_context(&processed_content, &err.message, pos.line, pos.column);
                                
                                // 提供修复建议
                                let (error_type, suggestion) = analyze_error(&err.message);
                                println!("\n错误类型: {}", error_type);
                                if let Some(suggest) = suggestion {
                                    println!("修复建议: {}", suggest);
                                }
                            }
                        }
                    }
                },
                Err(err) => {
                    // 增强解析错误报告
                    println!("\n\x1b[31m解析错误:\x1b[0m {}", err.message);
                    
                    // 显示错误位置信息
                    display_error_context(&processed_content, &err.message, err.line, err.column);
                    
                    // 提供修复建议
                    let (error_type, suggestion) = analyze_error(&err.message);
                    println!("\n错误类型: {}", error_type);
                    if let Some(suggest) = suggestion {
                        println!("修复建议: {}", suggest);
                    }
                }
            }
        },
        Err(err) => {
            if all_error_mode {
                error_collection.preprocessing_errors.push(err.clone());
                error_collection.display_all_errors("");
            } else {
                println!("\n\x1b[31m预处理文件错误:\x1b[0m {}", err);
                
                // 尝试从错误信息中提取文件路径
                if let Some(file_path) = extract_file_path_from_error(&err) {
                    println!("出错的文件: {}", file_path);
                    
                    // 提供修复建议
                    let (error_type, suggestion) = analyze_error(&err);
                    println!("\n错误类型: {}", error_type);
                    if let Some(suggest) = suggestion {
                        println!("修复建议: {}", suggest);
                    }
                }
            }
        },
    }
}