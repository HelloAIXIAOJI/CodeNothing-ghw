pub mod lexer;
pub mod parser_base;
pub mod expression_parser;
pub mod statement_parser;

use crate::ast::{Program, Function, Statement, Expression, Type, BinaryOperator, Parameter, Namespace};
use lexer::{remove_comments, tokenize};
use parser_base::ParserBase;
use expression_parser::ExpressionParser;
use statement_parser::StatementParser;
use std::env;

// 添加调试模式检查函数
fn is_all_error_mode() -> bool {
    env::args().any(|arg| arg == "--cn-allerror")
}

// 添加解析错误结构体
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub position: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (行: {}, 列: {}, 位置: {})", 
            self.message, self.line, self.column, self.position)
    }
}

// 添加错误收集器结构体
#[derive(Debug, Default)]
pub struct ErrorCollector {
    pub errors: Vec<ParseError>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        ErrorCollector {
            errors: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

// 修改解析函数返回类型
pub fn parse(source: &str, debug: bool) -> Result<Program, ParseError> {
    // 预处理：移除注释
    let source_without_comments = remove_comments(source);
    
    // 词法分析：将源代码转换为词法单元
    let tokens = tokenize(&source_without_comments, debug);
    
    // 创建解析器
    let mut parser = ParserBase::new(&source_without_comments, tokens, debug);
    
    // 创建错误收集器
    let mut error_collector = ErrorCollector::new();
    
    // 解析程序
    let all_error_mode = is_all_error_mode();
    let program_result = parse_program_with_error_collection(&mut parser, &mut error_collector, all_error_mode);
    
    if all_error_mode && error_collector.has_errors() {
        // 在全部错误模式下，如果有错误，返回第一个错误
        Err(error_collector.errors[0].clone())
    } else {
        // 在普通模式下，直接返回解析结果
        program_result.map_err(|err_msg| {
            let (line, column) = parser.get_line_column();
            ParseError {
                message: err_msg,
                line,
                column,
                position: parser.position,
            }
        })
    }
}

fn parse_program_with_error_collection(
    parser: &mut ParserBase, 
    error_collector: &mut ErrorCollector,
    all_error_mode: bool
) -> Result<Program, String> {
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    let mut library_imports = Vec::new();
    let mut file_imports = Vec::new();
    
    while parser.position < parser.tokens.len() {
        if parser.peek() == Some(&"ns".to_string()) {
            match parse_namespace_with_error_collection(parser, error_collector, all_error_mode) {
                Ok(namespace) => namespaces.push(namespace),
                Err(err) if all_error_mode => {
                    // 在全部错误模式下，记录错误并继续解析
                    let (line, column) = parser.get_line_column();
                    error_collector.add_error(ParseError {
                        message: err,
                        line,
                        column,
                        position: parser.position,
                    });
                    
                    // 尝试恢复到下一个顶层声明
                    recover_to_next_top_level_declaration(parser);
                },
                Err(err) => return Err(err),
            }
        } else if parser.peek() == Some(&"fn".to_string()) {
            match parse_function_with_error_collection(parser, error_collector, all_error_mode) {
                Ok(function) => functions.push(function),
                Err(err) if all_error_mode => {
                    // 在全部错误模式下，记录错误并继续解析
                    let (line, column) = parser.get_line_column();
                    error_collector.add_error(ParseError {
                        message: err,
                        line,
                        column,
                        position: parser.position,
                    });
                    
                    // 尝试恢复到下一个顶层声明
                    recover_to_next_top_level_declaration(parser);
                },
                Err(err) => return Err(err),
            }
        } else if parser.peek() == Some(&"using".to_string()) {
            // 解析using语句
            parser.consume(); // 消费 "using"
            
            if parser.peek() == Some(&"lib_once".to_string()) || parser.peek() == Some(&"lib".to_string()) {
                let lib_keyword = parser.consume().unwrap(); // 消费 "lib_once" 或 "lib"
                
                // 期望 "<" 符号
                if let Err(err) = parser.expect("<") {
                    if all_error_mode {
                        let (line, column) = parser.get_line_column();
                        error_collector.add_error(ParseError {
                            message: err,
                            line,
                            column,
                            position: parser.position,
                        });
                        recover_to_next_top_level_declaration(parser);
                        continue;
                    } else {
                        return Err(err);
                    }
                }
                
                // 获取库名
                let lib_name = match parser.consume() {
                    Some(name) => name,
                    None => {
                        let err = parser.create_error("期望库名");
                        if all_error_mode {
                            let (line, column) = parser.get_line_column();
                            error_collector.add_error(ParseError {
                                message: err,
                                line,
                                column,
                                position: parser.position,
                            });
                            recover_to_next_top_level_declaration(parser);
                            continue;
                        } else {
                            return Err(err);
                        }
                    }
                };
                
                // 期望 ">" 符号
                if let Err(err) = parser.expect(">") {
                    if all_error_mode {
                        let (line, column) = parser.get_line_column();
                        error_collector.add_error(ParseError {
                            message: err,
                            line,
                            column,
                            position: parser.position,
                        });
                        recover_to_next_top_level_declaration(parser);
                        continue;
                    } else {
                        return Err(err);
                    }
                }
                
                // 期望 ";" 符号
                if let Err(err) = parser.expect(";") {
                    if all_error_mode {
                        let (line, column) = parser.get_line_column();
                        error_collector.add_error(ParseError {
                            message: err,
                            line,
                            column,
                            position: parser.position,
                        });
                        recover_to_next_top_level_declaration(parser);
                        continue;
                    } else {
                        return Err(err);
                    }
                }
                
                // 添加到库导入列表
                library_imports.push(lib_name);
            } else if parser.peek() == Some(&"file".to_string()) {
                // 解析文件导入
                parser.consume(); // 消费 "file"
                
                // 获取文件路径（可能被引号包裹）
                let file_path = match parser.consume() {
                    Some(path) => path,
                    None => {
                        let err = parser.create_error("期望文件路径");
                        if all_error_mode {
                            let (line, column) = parser.get_line_column();
                            error_collector.add_error(ParseError {
                                message: err,
                                line,
                                column,
                                position: parser.position,
                            });
                            recover_to_next_top_level_declaration(parser);
                            continue;
                        } else {
                            return Err(err);
                        }
                    }
                };
                
                // 移除可能存在的引号
                let file_path = if file_path.starts_with("\"") && file_path.ends_with("\"") {
                    file_path[1..file_path.len()-1].to_string()
                } else if file_path.starts_with("'") && file_path.ends_with("'") {
                    file_path[1..file_path.len()-1].to_string()
                } else {
                    file_path
                };
                
                // 期望 ";" 符号
                if let Err(err) = parser.expect(";") {
                    if all_error_mode {
                        let (line, column) = parser.get_line_column();
                        error_collector.add_error(ParseError {
                            message: err,
                            line,
                            column,
                            position: parser.position,
                        });
                        recover_to_next_top_level_declaration(parser);
                        continue;
                    } else {
                        return Err(err);
                    }
                }
                
                // 添加到文件导入列表
                file_imports.push(file_path);
            } else if parser.peek() == Some(&"ns".to_string()) || parser.peek() == Some(&"namespace".to_string()) {
                // 解析命名空间导入，但在顶层不做任何处理，因为命名空间导入只在函数内部有效
                parser.consume(); // 消费 "ns" 或 "namespace"
                
                // 解析命名空间路径
                while parser.peek().is_some() && parser.peek() != Some(&";".to_string()) {
                    parser.consume();
                }
                
                if let Err(err) = parser.expect(";") {
                    if all_error_mode {
                        let (line, column) = parser.get_line_column();
                        error_collector.add_error(ParseError {
                            message: err,
                            line,
                            column,
                            position: parser.position,
                        });
                        recover_to_next_top_level_declaration(parser);
                        continue;
                    } else {
                        return Err(err);
                    }
                }
            } else {
                let err = parser.create_error("期望 'lib_once'、'lib'、'file'、'ns' 或 'namespace' 关键字");
                if all_error_mode {
                    let (line, column) = parser.get_line_column();
                    error_collector.add_error(ParseError {
                        message: err,
                        line,
                        column,
                        position: parser.position,
                    });
                    recover_to_next_top_level_declaration(parser);
                    continue;
                } else {
                    return Err(err);
                }
            }
        } else {
            let err = parser.create_error(&format!("期望 'fn', 'ns', 或 'using', 但得到了 '{:?}'", parser.peek()));
            if all_error_mode {
                let (line, column) = parser.get_line_column();
                error_collector.add_error(ParseError {
                    message: err,
                    line,
                    column,
                    position: parser.position,
                });
                recover_to_next_top_level_declaration(parser);
                continue;
            } else {
                return Err(err);
            }
        }
    }
    
    Ok(Program { 
        functions, 
        namespaces,
        library_imports,
        file_imports,
    })
}

// 辅助函数：尝试恢复到下一个顶层声明
fn recover_to_next_top_level_declaration(parser: &mut ParserBase) {
    // 跳过当前错误的声明，直到找到下一个顶层声明（fn、ns或using）
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if token == "fn" || token == "ns" || token == "using" {
                break;
            }
            parser.consume();
        } else {
            break;
        }
    }
}

fn parse_namespace_with_error_collection(
    parser: &mut ParserBase,
    error_collector: &mut ErrorCollector,
    all_error_mode: bool
) -> Result<Namespace, String> {
    parser.expect("ns")?;
    
    let name = match parser.consume() {
        Some(name) => name,
        None => return Err(parser.create_error("期望命名空间名")),
    };
    
    if parser.debug {
        println!("开始解析命名空间: {}", name);
    }
    
    if let Err(err) = parser.expect("{") {
        return Err(err);
    }
    
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    
    while let Some(token) = parser.peek() {
        if parser.debug {
            println!("命名空间 {} 内部解析: 当前token = {:?}, 位置 = {}", 
                name, token, parser.position);
        }
        
        if token == "}" {
            break;
        } else if token == "fn" {
            match parse_function_with_error_collection(parser, error_collector, all_error_mode) {
                Ok(function) => functions.push(function),
                Err(err) if all_error_mode => {
                    // 在全部错误模式下，记录错误并继续解析
                    let (line, column) = parser.get_line_column();
                    error_collector.add_error(ParseError {
                        message: err,
                        line,
                        column,
                        position: parser.position,
                    });
                    
                    // 尝试恢复到命名空间内的下一个声明
                    recover_to_next_namespace_declaration(parser);
                },
                Err(err) => return Err(err),
            }
        } else if token == "ns" {
            match parse_namespace_with_error_collection(parser, error_collector, all_error_mode) {
                Ok(namespace) => namespaces.push(namespace),
                Err(err) if all_error_mode => {
                    // 在全部错误模式下，记录错误并继续解析
                    let (line, column) = parser.get_line_column();
                    error_collector.add_error(ParseError {
                        message: err,
                        line,
                        column,
                        position: parser.position,
                    });
                    
                    // 尝试恢复到命名空间内的下一个声明
                    recover_to_next_namespace_declaration(parser);
                },
                Err(err) => return Err(err),
            }
        } else {
            let (line, column) = parser.get_line_column();
            let err = format!("期望 'fn', 'ns' 或 '}}', 但得到了 '{}' (行: {}, 列: {}, 位置: {})", 
                token, line, column, parser.position);
            
            if all_error_mode {
                error_collector.add_error(ParseError {
                    message: err,
                    line,
                    column,
                    position: parser.position,
                });
                
                // 尝试恢复到命名空间内的下一个声明
                recover_to_next_namespace_declaration(parser);
                
                // 消费当前token
                parser.consume();
                continue;
            } else {
                return Err(err);
            }
        }
    }
    
    if parser.debug {
        println!("命名空间 {} 解析完成, 期望 '}}', 当前token = {:?}, 位置 = {}", 
            name, parser.peek(), parser.position);
    }
    
    if let Err(err) = parser.expect("}") {
        return Err(err);
    }
    
    if parser.debug {
        println!("命名空间 {} 的 '}}' 已消费, 期望 ';', 当前token = {:?}, 位置 = {}", 
            name, parser.peek(), parser.position);
    }
    
    if let Err(err) = parser.expect(";") {
        return Err(err);
    }
    
    if parser.debug {
        println!("命名空间 {} 解析成功", name);
    }
    
    Ok(Namespace { name, functions, namespaces })
}

// 辅助函数：尝试恢复到命名空间内的下一个声明
fn recover_to_next_namespace_declaration(parser: &mut ParserBase) {
    // 跳过当前错误的声明，直到找到下一个命名空间内的声明（fn、ns）或命名空间结束
    let mut brace_count = 0;
    
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if token == "fn" || token == "ns" {
                if brace_count == 0 {
                    break;
                }
            } else if token == "{" {
                brace_count += 1;
            } else if token == "}" {
                if brace_count == 0 {
                    break;
                }
                brace_count -= 1;
            }
            parser.consume();
        } else {
            break;
        }
    }
}

fn parse_function_with_error_collection(
    parser: &mut ParserBase,
    error_collector: &mut ErrorCollector,
    all_error_mode: bool
) -> Result<Function, String> {
    parser.expect("fn")?;
    
    let name = match parser.consume() {
        Some(name) => name,
        None => return Err("期望函数名".to_string()),
    };
    
    if let Err(err) = parser.expect("(") {
        return Err(err);
    }
    
    // 解析函数参数
    let mut parameters = Vec::new();
    if parser.peek() != Some(&")".to_string()) {
        // 至少有一个参数
        let param_name = match parser.consume() {
            Some(name) => name,
            None => return Err("期望参数名".to_string()),
        };
        
        if let Err(err) = parser.expect(":") {
            return Err(err);
        }
        
        let param_type = parser.parse_type()?;
        parameters.push(Parameter {
            name: param_name,
            param_type,
        });
        
        // 解析剩余参数
        while parser.peek() == Some(&",".to_string()) {
            parser.consume(); // 消费逗号
            let param_name = match parser.consume() {
                Some(name) => name,
                None => return Err("期望参数名".to_string()),
            };
            
            if let Err(err) = parser.expect(":") {
                return Err(err);
            }
            
            let param_type = parser.parse_type()?;
            parameters.push(Parameter {
                name: param_name,
                param_type,
            });
        }
    }
    
    if let Err(err) = parser.expect(")") {
        return Err(err);
    }
    
    if let Err(err) = parser.expect(":") {
        return Err(err);
    }
    
    let return_type = parser.parse_type()?;
    
    if let Err(err) = parser.expect("{") {
        return Err(err);
    }
    
    let mut body = Vec::new();
    while let Some(token) = parser.peek() {
        if token == "}" {
            break;
        }
        
        match parser.parse_statement() {
            Ok(stmt) => body.push(stmt),
            Err(err) if all_error_mode => {
                // 在全部错误模式下，记录错误并继续解析
                let (line, column) = parser.get_line_column();
                error_collector.add_error(ParseError {
                    message: err,
                    line,
                    column,
                    position: parser.position,
                });
                
                // 尝试恢复到函数体内的下一个语句
                recover_to_next_statement(parser);
                continue;
            },
            Err(err) => return Err(err),
        }
    }
    
    if parser.peek() != Some(&"}".to_string()) {
        return Err(format!("期望 '}}', 但得到了 {:?}", parser.peek()));
    }
    parser.consume(); // 消费 "}"
    
    if parser.peek() != Some(&";".to_string()) {
        return Err(format!("在函数 '{}' 定义末尾期望 ';', 但得到了 {:?}", name, parser.peek()));
    }
    parser.consume(); // 消费 ";"
    
    Ok(Function {
        name,
        parameters,
        return_type,
        body,
    })
}

// 辅助函数：尝试恢复到函数体内的下一个语句
fn recover_to_next_statement(parser: &mut ParserBase) {
    // 跳过当前错误的语句，直到找到分号或函数体结束
    let mut brace_count = 0;
    
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if token == ";" && brace_count == 0 {
                // 找到语句结束符，消费它并退出
                parser.consume();
                break;
            } else if token == "{" {
                brace_count += 1;
            } else if token == "}" {
                if brace_count == 0 {
                    // 找到函数体结束，不消费它并退出
                    break;
                }
                brace_count -= 1;
            }
            parser.consume();
        } else {
            break;
        }
    }
} 