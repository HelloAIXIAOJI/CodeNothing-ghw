// 函数解析模块
// 包含函数解析相关函数

use crate::ast::{Function, Parameter};
use crate::parser::parser_base::ParserBase;
use crate::parser::parser_utils::skip_to_next_statement_or_end;
use crate::parser::statement_parser::StatementParser;
use crate::parser::expression_parser::ExpressionParser;

/// 解析函数
pub fn parse_function(parser: &mut ParserBase) -> Result<Function, String> {
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

/// 收集函数解析错误
pub fn parse_function_collect_errors(parser: &mut ParserBase, errors: &mut Vec<String>) -> Result<Function, ()> {
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
        
        parameters.push(Parameter {
            name: param_name,
            param_type,
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
            
            parameters.push(Parameter {
                name: param_name,
                param_type,
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
                    skip_to_next_statement_or_end(parser);
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