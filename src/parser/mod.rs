pub mod lexer;
pub mod parser_base;
pub mod expression_parser;
pub mod statement_parser;

use crate::ast::{Program, Function, Statement, Expression, Type, BinaryOperator, Parameter, Namespace};
use lexer::{remove_comments, tokenize};
use parser_base::{ParserBase, ErrorLocation};
use expression_parser::ExpressionParser;
use statement_parser::StatementParser;

// 修改解析结果类型，包含可能的错误列表
pub type ParseResult = Result<Program, Vec<ErrorLocation>>;

pub fn parse(source: &str, debug: bool) -> ParseResult {
    // 预处理：移除注释
    let source_without_comments = remove_comments(source);
    
    // 词法分析：将源代码转换为词法单元
    let tokens = tokenize(&source_without_comments, debug);
    
    // 创建解析器
    let mut parser = ParserBase::new(&source_without_comments, tokens, debug);
    
    // 解析程序
    let result = parse_program(&mut parser);
    
    // 检查是否有收集到的错误
    if parser.has_errors() {
        // 返回所有收集到的错误
        return Err(parser.errors.clone());
    }
    
    // 如果没有收集到错误但解析失败，将单个错误包装为列表返回
    result.map_err(|err| {
        vec![ErrorLocation {
            message: err,
            token_position: parser.position,
            token: parser.peek().cloned(),
        }]
    })
}

fn parse_program(parser: &mut ParserBase) -> Result<Program, String> {
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    let mut library_imports = Vec::new();
    let mut file_imports = Vec::new();
    
    while parser.position < parser.tokens.len() {
        if parser.peek() == Some(&"ns".to_string()) {
            match parse_namespace(parser) {
                Ok(ns) => namespaces.push(ns),
                Err(err) => {
                    parser.add_error(err);
                    // 尝试恢复到下一个明确的点（例如，下一个顶级声明）
                    recover_to_next_declaration(parser);
                }
            }
        } else if parser.peek() == Some(&"fn".to_string()) {
            match parse_function(parser) {
                Ok(func) => functions.push(func),
                Err(err) => {
                    parser.add_error(err);
                    // 尝试恢复到下一个明确的点
                    recover_to_next_declaration(parser);
                }
            }
        } else if parser.peek() == Some(&"using".to_string()) {
            // 消费 "using"
            parser.consume();
            
            if parser.peek() == Some(&"lib_once".to_string()) || parser.peek() == Some(&"lib".to_string()) {
                let lib_keyword = parser.consume().unwrap();
                
                if parser.peek() != Some(&"<".to_string()) {
                    let err = format!("期望 '<', 但得到了 '{:?}' (位置: {})", parser.peek(), parser.position);
                    parser.add_error(err);
                    recover_to_next_declaration(parser);
                    continue;
                }
                parser.consume(); // 消费 "<"
                
                if let Some(lib_name) = parser.consume() {
                    if parser.peek() != Some(&">".to_string()) {
                        let err = format!("期望 '>', 但得到了 '{:?}' (位置: {})", parser.peek(), parser.position);
                        parser.add_error(err);
                    } else {
                        parser.consume(); // 消费 ">"
                        if parser.peek() != Some(&";".to_string()) {
                            let err = format!("期望 ';', 但得到了 '{:?}' (位置: {})", parser.peek(), parser.position);
                            parser.add_error(err);
                        } else {
                            parser.consume(); // 消费 ";"
                            // 添加库导入
                            library_imports.push((lib_name, lib_keyword == "lib_once".to_string()));
                        }
                    }
                } else {
                    let err = "期望库名".to_string();
                    parser.add_error(err);
                }
                
                recover_to_next_declaration(parser);
            } else if parser.peek() == Some(&"file".to_string()) {
                parser.consume(); // 消费 "file"
                
                if parser.peek().map_or(false, |t| t.starts_with('"') && t.ends_with('"')) {
                    let file_path = parser.consume().unwrap();
                    let path = file_path[1..file_path.len()-1].to_string();
                    
                    if parser.peek() != Some(&";".to_string()) {
                        let err = format!("期望 ';', 但得到了 '{:?}' (位置: {})", parser.peek(), parser.position);
                        parser.add_error(err);
                    } else {
                        parser.consume(); // 消费 ";"
                        // 添加文件导入
                        file_imports.push(path);
                    }
                } else {
                    let err = "期望文件路径字符串".to_string();
                    parser.add_error(err);
                }
                
                recover_to_next_declaration(parser);
            } else {
                let err = format!("期望 'lib', 'lib_once' 或 'file', 但得到了 '{:?}' (位置: {})", 
                    parser.peek(), parser.position);
                parser.add_error(err);
                recover_to_next_declaration(parser);
            }
        } else {
            let err = format!("期望顶级声明（如 'fn', 'ns' 或 'using'）, 但得到了 '{:?}' (位置: {})", 
                parser.peek(), parser.position);
            parser.add_error(err);
            
            // 跳过当前token，尝试继续解析
            parser.consume();
        }
    }
    
    Ok(Program {
        functions,
        namespaces,
        library_imports,
        file_imports,
    })
}

// 添加错误恢复辅助函数
fn recover_to_next_declaration(parser: &mut ParserBase) {
    // 向前扫描，直到找到明确的下一个声明起始点
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if token == "fn" || token == "ns" || token == "using" || token == ";" {
                // 如果找到分号，消费它并退出
                if token == ";" {
                    parser.consume();
                }
                break;
            }
            // 否则继续前进
            parser.consume();
        } else {
            break;
        }
    }
}

fn parse_namespace(parser: &mut ParserBase) -> Result<Namespace, String> {
    parser.expect("ns")?;
    
    let name = match parser.consume() {
        Some(name) => name,
        None => return Err("期望命名空间名".to_string()),
    };
    
    if parser.debug {
        println!("开始解析命名空间: {}", name);
    }
    
    parser.expect("{")?;
    
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
            functions.push(parse_function(parser)?);
        } else if token == "ns" {
            namespaces.push(parse_namespace(parser)?);
        } else {
            return Err(format!("期望 'fn', 'ns' 或 '}}', 但得到了 '{}' (位置: {})", 
                token, parser.position));
        }
    }
    
    if parser.debug {
        println!("命名空间 {} 解析完成, 期望 '}}', 当前token = {:?}, 位置 = {}", 
            name, parser.peek(), parser.position);
    }
    
    parser.expect("}")?;
    
    if parser.debug {
        println!("命名空间 {} 的 '}}' 已消费, 期望 ';', 当前token = {:?}, 位置 = {}", 
            name, parser.peek(), parser.position);
    }
    
    parser.expect(";")?;
    
    if parser.debug {
        println!("命名空间 {} 解析成功", name);
    }
    
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