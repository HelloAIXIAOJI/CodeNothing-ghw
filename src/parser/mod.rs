pub mod lexer;
pub mod parser_base;
pub mod expression_parser;
pub mod statement_parser;

use crate::ast::{Program, Function, Statement, Expression, Type, BinaryOperator, Parameter, Namespace};
use lexer::{remove_comments, tokenize};
use parser_base::ParserBase;
use expression_parser::ExpressionParser;
use statement_parser::StatementParser;

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

fn parse_program(parser: &mut ParserBase) -> Result<Program, String> {
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    
    while parser.position < parser.tokens.len() {
        if parser.peek() == Some(&"ns".to_string()) {
            namespaces.push(parse_namespace(parser)?);
        } else if parser.peek() == Some(&"fn".to_string()) {
            functions.push(parse_function(parser)?);
        } else {
            return Err(format!("期望 'fn' 或 'ns', 但得到了 '{:?}'", parser.peek()));
        }
    }
    
    Ok(Program { functions, namespaces })
}

fn parse_namespace(parser: &mut ParserBase) -> Result<Namespace, String> {
    parser.expect("ns")?;
    
    let name = match parser.consume() {
        Some(name) => name,
        None => return Err("期望命名空间名".to_string()),
    };
    
    parser.expect("{")?;
    
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    
    while let Some(token) = parser.peek() {
        if token == "}" {
            break;
        } else if token == "fn" {
            functions.push(parse_function(parser)?);
        } else if token == "ns" {
            namespaces.push(parse_namespace(parser)?);
        } else {
            return Err(format!("期望 'fn', 'ns' 或 '}}', 但得到了 '{}'", token));
        }
    }
    
    parser.expect("}")?;
    parser.expect(";")?;
    
    Ok(Namespace { name, functions, namespaces })
}

fn parse_function(parser: &mut ParserBase) -> Result<Function, String> {
    parser.expect("fn")?;
    
    let name = match parser.consume() {
        Some(name) => name,
        None => return Err("期望函数名".to_string()),
    };
    
    parser.expect("(")?;
    
    // 解析函数参数
    let mut parameters = Vec::new();
    if parser.peek() != Some(&")".to_string()) {
        // 至少有一个参数
        let param_name = parser.consume().ok_or_else(|| "期望参数名".to_string())?;
        parser.expect(":")?;
        let param_type = parser.parse_type()?;
        parameters.push(Parameter {
            name: param_name,
            param_type,
        });
        
        // 解析剩余参数
        while parser.peek() == Some(&",".to_string()) {
            parser.consume(); // 消费逗号
            let param_name = parser.consume().ok_or_else(|| "期望参数名".to_string())?;
            parser.expect(":")?;
            let param_type = parser.parse_type()?;
            parameters.push(Parameter {
                name: param_name,
                param_type,
            });
        }
    }
    
    parser.expect(")")?;
    
    parser.expect(":")?;
    let return_type = parser.parse_type()?;
    
    parser.expect("{")?;
    
    let mut body = Vec::new();
    while let Some(token) = parser.peek() {
        if token == "}" {
            break;
        }
        body.push(parser.parse_statement()?);
    }
    
    parser.expect("}")?;
    parser.expect(";")?;
    
    Ok(Function {
        name,
        parameters,
        return_type,
        body,
    })
} 