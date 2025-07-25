// 命名空间解析模块
// 包含命名空间解析相关函数

use crate::ast::{Namespace, Function};
use crate::parser::parser_base::ParserBase;
use crate::parser::parser_utils::skip_to_next_ns_member;
use crate::parser::statement_parser::StatementParser;
use crate::parser::expression_parser::ExpressionParser;

/// 解析命名空间
pub fn parse_namespace(parser: &mut ParserBase) -> Result<Namespace, String> {
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
            let mut sub_namespace = parse_namespace(parser)?;
            sub_namespace.ns_type = crate::ast::NamespaceType::Code; // 设置为代码命名空间
            namespaces.push(sub_namespace);
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
    
    // 创建命名空间，ns_type默认为Code，将在调用处设置
    Ok(Namespace { 
        name, 
        ns_type: crate::ast::NamespaceType::Code, // 默认为代码命名空间
        functions, 
        namespaces 
    })
}

/// 收集命名空间解析错误
pub fn parse_namespace_collect_errors(parser: &mut ParserBase, errors: &mut Vec<String>) -> Result<Namespace, ()> {
    if let Err(e) = parser.expect("ns") {
        errors.push(e);
        return Err(());
    }
    
    let name = match parser.consume() {
        Some(name) => name,
        None => {
            errors.push("期望命名空间名".to_string());
            return Err(());
        }
    };
    
    if parser.debug {
        println!("开始解析命名空间: {}", name);
    }
    
    if let Err(e) = parser.expect("{") {
        errors.push(e);
        return Err(());
    }
    
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    
    while let Some(token) = parser.peek() {
        if token == "}" {
            break;
        } else if token == "fn" {
            match parse_function_collect_errors(parser, errors) {
                Ok(func) => functions.push(func),
                Err(_) => {
                    // 跳过当前函数，在下一个函数或命名空间的开始继续
                    skip_to_next_ns_member(parser);
                }
            }
        } else if token == "ns" {
            match parse_namespace_collect_errors(parser, errors) {
                Ok(ns) => namespaces.push(ns),
                Err(_) => {
                    // 跳过当前命名空间，在下一个函数或命名空间的开始继续
                    skip_to_next_ns_member(parser);
                }
            }
        } else {
            errors.push(format!("期望 'fn', 'ns' 或 '}}', 但得到了 '{}' (位置: {})", 
                token, parser.position));
            // 尝试跳过当前错误
            parser.consume();
        }
    }
    
    if parser.debug {
        println!("命名空间 {} 解析完成, 期望 '}}', 当前token = {:?}, 位置 = {}", 
            name, parser.peek(), parser.position);
    }
    
    if let Err(e) = parser.expect("}") {
        errors.push(e);
        return Err(());
    }
    
    if parser.debug {
        println!("命名空间 {} 的 '}}' 已消费, 期望 ';', 当前token = {:?}, 位置 = {}", 
            name, parser.peek(), parser.position);
    }
    
    if let Err(e) = parser.expect(";") {
        errors.push(e);
        return Err(());
    }
    
    if parser.debug {
        println!("命名空间 {} 解析成功", name);
    }
    
    Ok(Namespace { 
        name, 
        ns_type: crate::ast::NamespaceType::Code, // 设置为代码命名空间类型
        functions, 
        namespaces 
    })
}

/// 解析函数（用于命名空间内部）
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
        parameters.push(crate::ast::Parameter {
            name: param_name,
            param_type,
            default_value: None,
        });
        
        // 解析剩余参数
        while parser.peek() == Some(&",".to_string()) {
            parser.consume(); // 消费逗号
            let param_name = parser.consume().ok_or_else(|| "期望参数名".to_string())?;
            parser.expect(":")?;
            let param_type = parser.parse_type()?;
            parameters.push(crate::ast::Parameter {
                name: param_name,
                param_type,
                default_value: None,
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

/// 收集函数解析错误（用于命名空间内部）
fn parse_function_collect_errors(parser: &mut ParserBase, errors: &mut Vec<String>) -> Result<Function, ()> {
    if let Err(e) = parser.expect("fn") {
        errors.push(e);
        return Err(());
    }
    
    let name = match parser.consume() {
        Some(name) => name,
        None => {
            errors.push("期望函数名".to_string());
            return Err(());
        }
    };
    
    if let Err(e) = parser.expect("(") {
        errors.push(e);
        return Err(());
    }
    
    // 解析函数参数
    let mut parameters = Vec::new();
    if parser.peek() != Some(&")".to_string()) {
        // 至少有一个参数
        let param_name = match parser.consume() {
            Some(name) => name,
            None => {
                errors.push("期望参数名".to_string());
                return Err(());
            }
        };
        
        if let Err(e) = parser.expect(":") {
            errors.push(e);
            return Err(());
        }
        
        let param_type = match parser.parse_type() {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                return Err(());
            }
        };
        
        parameters.push(crate::ast::Parameter {
            name: param_name.clone(),
            param_type,
            default_value: None,
        });
        
        // 解析剩余参数
        while parser.peek() == Some(&",".to_string()) {
            parser.consume(); // 消费逗号
            
            let param_name = match parser.consume() {
                Some(name) => name,
                None => {
                    errors.push("期望参数名".to_string());
                    return Err(());
                }
            };
            
            if let Err(e) = parser.expect(":") {
                errors.push(e);
                return Err(());
            }
            
            let param_type = match parser.parse_type() {
                Ok(t) => t,
                Err(e) => {
                    errors.push(e);
                    return Err(());
                }
            };
            
            parameters.push(crate::ast::Parameter {
                name: param_name.clone(),
                param_type,
                default_value: None,
            });
        }
    }
    
    if let Err(e) = parser.expect(")") {
        errors.push(e);
        return Err(());
    }
    
    if let Err(e) = parser.expect(":") {
        errors.push(e);
        return Err(());
    }
    
    let return_type = match parser.parse_type() {
        Ok(t) => t,
        Err(e) => {
            errors.push(e);
            return Err(());
        }
    };
    
    if let Err(e) = parser.expect("{") {
        errors.push(e);
        return Err(());
    }
    
    let mut body = Vec::new();
    let mut brace_count = 1; // 我们已经消费了开括号
    
    while parser.position < parser.tokens.len() {
        if parser.peek() == Some(&"}".to_string()) {
            brace_count -= 1;
            if brace_count == 0 {
                // 找到了匹配的右括号
                break;
            }
            parser.consume();
        } else if parser.peek() == Some(&"{".to_string()) {
            brace_count += 1;
            parser.consume();
        } else {
            match parser.parse_statement() {
                Ok(stmt) => body.push(stmt),
                Err(e) => {
                    errors.push(e);
                    // 跳过到下一个语句的开始，或者函数结束
                    crate::parser::parser_utils::skip_to_next_statement_or_end(parser);
                    if parser.peek() == Some(&"}".to_string()) && brace_count == 1 {
                        break;
                    }
                }
            }
        }
    }
    
    if parser.peek() != Some(&"}".to_string()) {
        errors.push(format!("期望 '}}', 但得到了 {:?}", parser.peek()));
        return Err(());
    }
    parser.consume(); // 消费 "}"
    
    if parser.peek() != Some(&";".to_string()) {
        errors.push(format!("在函数 '{}' 定义末尾期望 ';', 但得到了 {:?}", name, parser.peek()));
        return Err(());
    }
    parser.consume(); // 消费 ";"
    
    Ok(Function {
        name,
        parameters,
        return_type,
        body,
    })
} 