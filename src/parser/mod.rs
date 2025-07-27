pub mod lexer;
pub mod parser_base;
pub mod expression_parser;
pub mod statement_parser;
pub mod error_handler;
pub mod parser_utils;
pub mod namespace_parser;
pub mod function_parser;
pub mod program_parser;
pub mod class_parser;
pub mod interface_parser;
pub mod enum_parser;
pub mod pointer_parser;

use crate::ast::Program;
use lexer::{remove_comments, tokenize};
use parser_base::ParserBase;
use error_handler::add_line_info;
use program_parser::{parse_program, parse_program_collect_all_errors};

/// 主要的解析入口函数
pub fn parse(source: &str, debug: bool) -> Result<Program, String> {
    // 预处理：移除注释
    let source_without_comments = remove_comments(source);
    
    // 词法分析：将源代码转换为词法单元
    let tokens = tokenize(&source_without_comments, debug);
    
    // 创建解析器
    let mut parser = ParserBase::new(&source_without_comments, tokens, debug);
    
    // 解析程序
    parse_program(&mut parser)
}

/// 收集所有错误的解析函数
pub fn parse_all_errors(source: &str, debug: bool) -> Result<(Program, Vec<String>), Vec<String>> {
    // 预处理：移除注释
    let source_without_comments = remove_comments(source);
    
    // 词法分析：将源代码转换为词法单元
    let tokens = tokenize(&source_without_comments, debug);
    
    // 创建解析器
    let mut parser = ParserBase::new(&source_without_comments, tokens.clone(), debug);
    
    // 先尝试常规解析，如果成功则没有错误
    match parse_program(&mut parser) {
        Ok(program) => Ok((program, Vec::new())), // 没有错误，返回成功解析的程序和空警告列表
        Err(_) => {
            // 如果常规解析失败，切换到收集所有错误的模式
            // 重置解析器
            let mut parser = ParserBase::new(&source_without_comments, tokens, debug);
            
            // 收集所有错误
            let mut errors = Vec::new();
            parse_program_collect_all_errors(&mut parser, &mut errors);
            
            // 如果有错误，返回错误列表
            if !errors.is_empty() {
                // 添加行号信息到错误
                let errors_with_line = errors.into_iter()
                    .map(|error| add_line_info(&source_without_comments, &error))
                    .collect();
                    
                Err(errors_with_line)
            } else {
                // 这种情况不应该发生，因为前面的解析已经失败了
                Err(vec!["未知解析错误".to_string()])
            }
        }
    }
} 