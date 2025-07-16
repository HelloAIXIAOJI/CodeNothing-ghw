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

// 新增函数，收集所有错误
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

// 添加行号信息到错误消息
fn add_line_info(source: &str, error_msg: &str) -> String {
    // 如果错误消息中已经包含行号信息，就直接返回
    if error_msg.contains("第") && error_msg.contains("行") {
        return error_msg.to_string();
    }

    // 如果错误消息包含位置信息，尝试添加行号
    if let Some(pos_start) = error_msg.find("(位置:") {
        if let Some(pos_end) = error_msg[pos_start..].find(")") {
            let pos_str = &error_msg[pos_start+4..pos_start+pos_end];
            if let Ok(pos) = pos_str.trim().parse::<usize>() {
                // 计算行号和列号
                let mut line = 1;
                let mut col = 1;
                for (i, c) in source.chars().enumerate() {
                    if i == pos {
                        break;
                    }
                    if c == '\n' {
                        line += 1;
                        col = 1;
                    } else {
                        col += 1;
                    }
                }
                
                // 添加行号和列号信息，使用更清晰的格式
                let base_msg = error_msg[0..pos_start].trim();
                return format!("{} [第{}行,第{}列]", base_msg, line, col);
            }
        }
    }
    
    // 如果没有位置信息，返回原始错误
    error_msg.to_string()
}

// 收集所有错误的程序解析函数
fn parse_program_collect_all_errors(parser: &mut ParserBase, errors: &mut Vec<String>) {
    let mut try_next_item = true;
    
    while parser.position < parser.tokens.len() && try_next_item {
        try_next_item = false;
        
        if parser.peek() == Some(&"ns".to_string()) {
            match parse_namespace_collect_errors(parser, errors) {
                Ok(_) => try_next_item = true,
                Err(_) => {
                    // 跳过当前命名空间，尝试在下一个 ns、fn 或 using 关键字处继续解析
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                }
            }
        } else if parser.peek() == Some(&"fn".to_string()) {
            match parse_function_collect_errors(parser, errors) {
                Ok(_) => try_next_item = true,
                Err(_) => {
                    // 跳过当前函数，尝试在下一个 ns、fn 或 using 关键字处继续解析
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                }
            }
        } else if parser.peek() == Some(&"const".to_string()) {
            // 解析常量定义
            parser.consume(); // 消费 "const"
            
            // 获取常量名
            let const_name = match parser.consume() {
                Some(name) => name,
                None => {
                    errors.push("期望常量名".to_string());
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
            };
            
            // 期望 ":" 符号
            if let Err(e) = parser.expect(":") {
                errors.push(e);
                skip_to_next_top_level_item(parser);
                try_next_item = parser.position < parser.tokens.len();
                continue;
            }
            
            // 解析类型
            let type_name = match parser.consume() {
                Some(t) => t,
                None => {
                    errors.push("期望类型名".to_string());
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
            };
            
            // 转换为内部类型
            let const_type = match type_name.as_str() {
                "int" => crate::ast::Type::Int,
                "float" => crate::ast::Type::Float,
                "bool" => crate::ast::Type::Bool,
                "string" => crate::ast::Type::String,
                "long" => crate::ast::Type::Long,
                _ => {
                    errors.push(format!("不支持的常量类型: {}", type_name));
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
            };
            
            // 期望 "=" 符号
            if let Err(e) = parser.expect("=") {
                errors.push(e);
                skip_to_next_top_level_item(parser);
                try_next_item = parser.position < parser.tokens.len();
                continue;
            }
            
            // 解析初始值表达式
            match parser.parse_expression() {
                Ok(_) => {},
                Err(e) => {
                    errors.push(e);
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
            }
            
            // 期望 ";" 符号
            if let Err(e) = parser.expect(";") {
                errors.push(e);
                skip_to_next_top_level_item(parser);
                try_next_item = parser.position < parser.tokens.len();
                continue;
            }
            
            try_next_item = true;
        } else if parser.peek() == Some(&"using".to_string()) {
            parser.consume(); // 消费 "using"
            
            if parser.peek() == Some(&"lib_once".to_string()) || parser.peek() == Some(&"lib".to_string()) {
                parser.consume(); // 消费 "lib_once" 或 "lib"
                
                // 期望 "<" 符号
                if let Err(e) = parser.expect("<") {
                    errors.push(e);
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
                
                // 获取库名
                if parser.consume().is_none() {
                    errors.push("期望库名".to_string());
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
                
                // 期望 ">" 符号
                if let Err(e) = parser.expect(">") {
                    errors.push(e);
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
                
                // 期望 ";" 符号
                if let Err(e) = parser.expect(";") {
                    errors.push(e);
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
                
                try_next_item = true;
            } else if parser.peek() == Some(&"file".to_string()) {
                parser.consume(); // 消费 "file"
                
                // 获取文件路径
                let file_path_token = match parser.consume() {
                    Some(path) => path,
                    None => {
                        errors.push(format!("期望文件路径 (位置: {})", parser.position));
                        skip_to_next_top_level_item(parser);
                        try_next_item = parser.position < parser.tokens.len();
                        continue;
                    }
                };
                
                // 移除可能存在的引号
                let _file_path = if file_path_token.starts_with("\"") && file_path_token.ends_with("\"") {
                    file_path_token[1..file_path_token.len()-1].to_string()
                } else if file_path_token.starts_with("'") && file_path_token.ends_with("'") {
                    file_path_token[1..file_path_token.len()-1].to_string()
                } else {
                    file_path_token
                };
                
                // 期望 ";" 符号
                if let Err(e) = parser.expect(";") {
                    errors.push(e);
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
                
                try_next_item = true;
            } else if parser.peek() == Some(&"ns".to_string()) || parser.peek() == Some(&"namespace".to_string()) {
                parser.consume(); // 消费 "ns" 或 "namespace"
                
                // 解析命名空间路径
                let mut path = Vec::new();
                
                // 获取第一个命名空间名称
                match parser.consume() {
                    Some(name) => path.push(name),
                    None => {
                        errors.push("期望命名空间名".to_string());
                        skip_to_next_top_level_item(parser);
                        try_next_item = parser.position < parser.tokens.len();
                        continue;
                    }
                }
                
                // 解析嵌套命名空间路径
                while parser.peek() == Some(&"::".to_string()) {
                    parser.consume(); // 消费 "::"
                    
                    match parser.consume() {
                        Some(name) => path.push(name),
                        None => {
                            errors.push("期望命名空间名".to_string());
                            break;
                        }
                    }
                }
                
                // 期望 ";" 符号
                if let Err(e) = parser.expect(";") {
                    errors.push(e);
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                    continue;
                }
                
                try_next_item = true;
            } else {
                errors.push(format!("期望 'lib', 'lib_once', 'file', 'ns' 或 'namespace', 但得到了 {:?} (位置: {})", parser.peek(), parser.position));
                skip_to_next_top_level_item(parser);
                try_next_item = parser.position < parser.tokens.len();
            }
        } else {
            errors.push(format!("期望 'fn', 'ns' 或 'using', 但得到了 {:?} (位置: {})", parser.peek(), parser.position));
            skip_to_next_top_level_item(parser);
            try_next_item = parser.position < parser.tokens.len();
        }
    }
}

// 跳过当前项，找到下一个顶层项（函数、命名空间或导入）的开始
fn skip_to_next_top_level_item(parser: &mut ParserBase) {
    let mut brace_count = 0;
    
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if brace_count == 0 && (token == "fn" || token == "ns" || token == "using") {
                // 找到下一个顶层项
                return;
            } else if token == "{" {
                brace_count += 1;
                parser.consume();
            } else if token == "}" {
                if brace_count > 0 {
                    brace_count -= 1;
                }
                parser.consume();
            } else {
                parser.consume();
            }
        } else {
            break;
        }
    }
}

// 收集命名空间解析错误
fn parse_namespace_collect_errors(parser: &mut ParserBase, errors: &mut Vec<String>) -> Result<Namespace, ()> {
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

// 跳过命名空间内的当前项，找到下一个成员（函数或嵌套命名空间）的开始
fn skip_to_next_ns_member(parser: &mut ParserBase) {
    let mut brace_count = 0;
    
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if token == "{" {
                brace_count += 1;
                parser.consume();
            } else if token == "}" {
                if brace_count == 0 {
                    return; // 找到了命名空间的结束
                }
                brace_count -= 1;
                parser.consume();
            } else if brace_count == 0 && (token == "fn" || token == "ns") {
                return; // 找到了下一个成员
            } else {
                parser.consume();
            }
        }
    }
}

// 收集函数解析错误
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

// 跳过到下一个语句开始或函数结束
fn skip_to_next_statement_or_end(parser: &mut ParserBase) {
    let mut brace_count = 0;
    
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if token == ";" {
                parser.consume();
                return; // 找到了语句的结束
            } else if token == "{" {
                brace_count += 1;
                parser.consume();
            } else if token == "}" {
                if brace_count == 0 {
                    return; // 找到了函数的结束
                }
                brace_count -= 1;
                parser.consume();
            } else {
                parser.consume();
            }
        }
    }
}

// 原有的parse_program函数保持不变
fn parse_program(parser: &mut ParserBase) -> Result<Program, String> {
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    let mut imported_namespaces = Vec::new();
    let mut file_imports = Vec::new();
    let mut constants = Vec::new(); // 新增：用于存储常量定义
    
    while parser.position < parser.tokens.len() {
        if parser.peek() == Some(&"ns".to_string()) {
            // 解析命名空间
            let namespace = parse_namespace(parser)?;
            namespaces.push(namespace);
        } else if parser.peek() == Some(&"fn".to_string()) {
            // 解析函数
            let function = parse_function(parser)?;
            functions.push(function);
        } else if parser.peek() == Some(&"const".to_string()) {
            // 解析常量定义
            parser.consume(); // 消费 "const"
            
            // 获取常量名
            let const_name = parser.consume()
                .ok_or_else(|| "期望常量名".to_string())?;
            
            parser.expect(":")?;
            
            // 解析类型
            let type_name = parser.consume()
                .ok_or_else(|| "期望类型名".to_string())?;
            
            // 转换为内部类型
            let const_type = match type_name.as_str() {
                "int" => crate::ast::Type::Int,
                "float" => crate::ast::Type::Float,
                "bool" => crate::ast::Type::Bool,
                "string" => crate::ast::Type::String,
                "long" => crate::ast::Type::Long,
                _ => return Err(format!("不支持的常量类型: {}", type_name))
            };
            
            parser.expect("=")?;
            
            // 解析初始值表达式
            let init_expr = parser.parse_expression()?;
            
            parser.expect(";")?;
            
            // 添加到常量列表
            constants.push((const_name, const_type, init_expr));
        } else if parser.peek() == Some(&"using".to_string()) {
            // 解析using语句
            parser.consume(); // 消费 "using"
            
            if parser.peek() == Some(&"lib_once".to_string()) || parser.peek() == Some(&"lib".to_string()) {
                let _lib_keyword = parser.consume().unwrap(); // 消费 "lib_once" 或 "lib"
                
                // 期望 "<" 符号
                parser.expect("<")?;
                
                // 获取库名
                let lib_name = parser.consume().ok_or_else(|| "期望库名".to_string())?;
                
                // 期望 ">" 符号
                parser.expect(">")?;
                
                // 期望 ";" 符号
                parser.expect(";")?;
                
                // 添加到命名空间导入列表，使用Library类型
                imported_namespaces.push((crate::ast::NamespaceType::Library, vec![lib_name]));
            } else if parser.peek() == Some(&"file".to_string()) {
                // 解析文件导入
                parser.consume(); // 消费 "file"
                
                // 获取文件路径（可能被引号包裹）
                let file_path = parser.consume().ok_or_else(|| "期望文件路径".to_string())?;
                
                // 移除可能存在的引号
                let file_path = if file_path.starts_with("\"") && file_path.ends_with("\"") {
                    file_path[1..file_path.len()-1].to_string()
                } else if file_path.starts_with("'") && file_path.ends_with("'") {
                    file_path[1..file_path.len()-1].to_string()
                } else {
                    file_path
                };
                
                // 期望 ";" 符号
                parser.expect(";")?;
                
                // 添加到文件导入列表
                file_imports.push(file_path);
            } else if parser.peek() == Some(&"ns".to_string()) || parser.peek() == Some(&"namespace".to_string()) {
                // 解析命名空间导入
                parser.consume(); // 消费 "ns" 或 "namespace"
                
                // 解析命名空间路径
                let mut path = Vec::new();
                let first_name = parser.consume().ok_or_else(|| "期望命名空间名".to_string())?;
                path.push(first_name);
                
                // 解析嵌套命名空间路径
                while parser.peek() == Some(&"::".to_string()) {
                    parser.consume(); // 消费 "::"
                    let name = parser.consume().ok_or_else(|| "期望命名空间名".to_string())?;
                    path.push(name);
                }
                
                // 期望 ";" 符号
                parser.expect(";")?;
                
                // 添加到命名空间导入列表，使用Code类型
                imported_namespaces.push((crate::ast::NamespaceType::Code, path));
            } else {
                return Err("期望 'lib_once'、'lib'、'file'、'ns' 或 'namespace' 关键字".to_string());
            }
        } else {
            return Err(format!("期望 'fn', 'ns', 或 'using', 但得到了 '{:?}'", parser.peek()));
        }
    }
    
    Ok(Program {
        functions,
        namespaces,
        imported_namespaces,
        file_imports,
        constants, // 添加常量列表
    })
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