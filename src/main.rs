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
            
            let ast = parser::parse(&processed_content, debug_parser);
            match ast {
                Ok(program) => {
                    let result = interpreter::interpret(&program);
                    println!("程序执行结果: {}", result);
                },
                Err(errors) => {
                    // 增强错误报告 - 显示所有收集到的错误
                    if errors.is_empty() {
                        println!("解析错误：未知错误");
                    } else {
                        println!("\n发现 {} 个解析错误:", errors.len());
                        println!("==============================================");
                        
                        // 将源代码按行分割
                        let lines: Vec<&str> = processed_content.lines().collect();
                        
                        // 显示每一个错误的详细信息
                        for (i, error) in errors.iter().enumerate() {
                            println!("\n错误 #{}: {}", i + 1, error.message);
                            
                            // 显示当前Token信息
                            if let Some(ref token) = error.token {
                                println!("当前处理的标记: '{}'", token);
                            }
                            
                            println!("Token位置索引: {}", error.token_position);
                            
                            // 从源代码的开始扫描，统计token数量，找到大致的错误行
                            let tokens_per_line = tokenize_lines(&processed_content);
                            let mut token_count = 0;
                            let mut found_token_line = false;
                            
                            // 找到token_position所在的行
                            for (line_idx, tokens_in_line) in tokens_per_line.iter().enumerate() {
                                if token_count + tokens_in_line.len() > error.token_position {
                                    // 找到了错误所在的行
                                    found_token_line = true;
                                    let current_line = line_idx + 1;
                                    
                                    // 估算token在行中的位置
                                    let token_idx_in_line = error.token_position - token_count;
                                    let column = if token_idx_in_line < tokens_in_line.len() {
                                        // 使用token的实际位置
                                        tokens_in_line[token_idx_in_line].1 + 1 // +1 因为列从1开始
                                    } else {
                                        // fallback
                                        1
                                    };
                                    
                                    println!("位置: 第 {} 行, 第 {} 列", current_line, column);
                                    
                                    // 显示错误行的上下文
                                    let start_line = if current_line > 2 { current_line - 2 } else { 1 };
                                    let end_line = if current_line + 1 < lines.len() { current_line + 1 } else { lines.len() };
                                    
                                    println!("代码上下文:");
                                    for l in start_line..=end_line {
                                        if l > 0 && l <= lines.len() {
                                            let line_content = lines[l-1];
                                            if l == current_line {
                                                // 错误所在行，高亮显示
                                                println!("-> {}: {}", l, line_content);
                                                if column > 0 {
                                                    // 在错误位置下方显示指示符
                                                    println!("   {}{}", " ".repeat(column - 1), "^");
                                                }
                                            } else {
                                                println!("   {}: {}", l, line_content);
                                            }
                                        }
                                    }
                                    
                                    break;
                                }
                                
                                token_count += tokens_in_line.len();
                            }
                            
                            if !found_token_line {
                                println!("无法精确定位错误行");
                            }
                            
                            println!("----------------------------------------------");
                        }
                    }
                }
            }
        },
        Err(err) => println!("预处理文件错误: {}", err),
    }
}

// 辅助函数：为每一行标记token和它们在行中的位置
fn tokenize_lines(source: &str) -> Vec<Vec<(String, usize)>> {
    let mut result = Vec::new();
    
    for line in source.lines() {
        let mut line_tokens = Vec::new();
        let mut in_token = false;
        let mut token_start = 0;
        let mut token = String::new();
        
        for (i, c) in line.char_indices() {
            if c.is_whitespace() {
                if in_token {
                    // 结束当前token
                    line_tokens.push((token.clone(), token_start));
                    token.clear();
                    in_token = false;
                }
            } else {
                if !in_token {
                    // 开始新token
                    token_start = i;
                    in_token = true;
                }
                token.push(c);
            }
        }
        
        // 检查最后一个token
        if in_token {
            line_tokens.push((token, token_start));
        }
        
        result.push(line_tokens);
    }
    
    result
}