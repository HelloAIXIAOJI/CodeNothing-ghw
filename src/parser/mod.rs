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
    let mut library_imports = Vec::new();
    
    while parser.position < parser.tokens.len() {
        if parser.peek() == Some(&"ns".to_string()) {
            namespaces.push(parse_namespace(parser)?);
        } else if parser.peek() == Some(&"fn".to_string()) {
            functions.push(parse_function(parser)?);
        } else if parser.peek() == Some(&"using".to_string()) {
            // 解析using语句
            parser.consume(); // 消费 "using"
            
            if parser.peek() == Some(&"lib_once".to_string()) || parser.peek() == Some(&"lib".to_string()) {
                let lib_keyword = parser.consume().unwrap(); // 消费 "lib_once" 或 "lib"
                
                // 期望 "<" 符号
                parser.expect("<")?;
                
                // 获取库名
                let lib_name = parser.consume().ok_or_else(|| "期望库名".to_string())?;
                
                // 期望 ">" 符号
                parser.expect(">")?;
                
                // 期望 ";" 符号
                parser.expect(";")?;
                
                // 添加到库导入列表
                library_imports.push(lib_name);
            } else if parser.peek() == Some(&"ns".to_string()) || parser.peek() == Some(&"namespace".to_string()) {
                // 解析命名空间导入，但在顶层不做任何处理，因为命名空间导入只在函数内部有效
                parser.consume(); // 消费 "ns" 或 "namespace"
                
                // 解析命名空间路径
                while parser.peek().is_some() && parser.peek() != Some(&";".to_string()) {
                    parser.consume();
                }
                
                parser.expect(";")?;
            } else {
                return Err("期望 'lib_once'、'lib'、'ns' 或 'namespace' 关键字".to_string());
            }
        } else {
            return Err(format!("期望 'fn', 'ns', 或 'using', 但得到了 '{:?}'", parser.peek()));
        }
    }
    
    Ok(Program { 
        functions, 
        namespaces,
        library_imports,
    })
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