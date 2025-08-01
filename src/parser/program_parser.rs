// 程序解析模块
// 包含程序解析相关函数

use crate::ast::Program;
use crate::parser::parser_base::ParserBase;
use crate::parser::parser_utils::skip_to_next_top_level_item;
use crate::parser::namespace_parser::{parse_namespace, parse_namespace_collect_errors};
use crate::parser::function_parser::{parse_function, parse_function_collect_errors};
use crate::parser::statement_parser::StatementParser;
use crate::parser::expression_parser::ExpressionParser;
use crate::parser::class_parser::ClassParser;
use crate::parser::interface_parser::InterfaceParser;
use crate::parser::enum_parser::EnumParser;

/// 解析程序
pub fn parse_program(parser: &mut ParserBase) -> Result<Program, String> {
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    let mut imported_namespaces = Vec::new();
    let mut file_imports = Vec::new();
    let mut constants = Vec::new(); // 新增：用于存储常量定义
    let mut classes = Vec::new(); // 新增：用于存储类定义
    let mut interfaces = Vec::new(); // 新增：用于存储接口定义
    let mut enums = Vec::new(); // 新增：用于存储枚举定义
    
    while parser.position < parser.tokens.len() {
        if parser.peek() == Some(&"ns".to_string()) {
            // 解析命名空间
            let namespace = parse_namespace(parser)?;
            namespaces.push(namespace);
        } else if parser.peek() == Some(&"fn".to_string()) {
            // 解析函数
            let function = parse_function(parser)?;
            functions.push(function);
        } else if parser.peek() == Some(&"class".to_string()) || parser.peek() == Some(&"abstract".to_string()) {
            // 解析类（包括抽象类）
            let class = parser.parse_class()?;
            classes.push(class);
        } else if parser.peek() == Some(&"interface".to_string()) {
            // 解析接口
            let interface = parser.parse_interface()?;
            interfaces.push(interface);
        } else if parser.peek() == Some(&"enum".to_string()) {
            // 解析枚举
            let enum_def = parser.parse_enum()?;
            enums.push(enum_def);
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
                // 文件导入已在预处理阶段处理，这里跳过
                parser.consume(); // 消费 "file"
                
                // 跳过文件路径
                parser.consume();
                
                // 期望 ";" 符号
                parser.expect(";")?;
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

                // 🔧 修复：根据命名空间名称判断类型
                // std是内置命名空间，不是外部库
                let namespace_type = if path.len() == 1 && (path[0] == "io" || path[0] == "time" || path[0] == "math" || path[0] == "fs" || path[0] == "os" || path[0] == "http" || path[0] == "json") {
                    crate::ast::NamespaceType::Library // 库命名空间
                } else {
                    crate::ast::NamespaceType::Code // 代码命名空间（包括std）
                };

                // 添加到命名空间导入列表
                imported_namespaces.push((namespace_type, path));
            } else {
                return Err("期望 'lib_once'、'lib'、'file'、'ns' 或 'namespace' 关键字".to_string());
            }
        } else {
            return Err(format!("期望 'fn', 'ns', 'class', 'abstract', 'interface', 'enum' 或 'using', 但得到了 '{:?}'", parser.peek()));
        }
    }
    
    Ok(Program {
        functions,
        namespaces,
        imported_namespaces,
        file_imports,
        constants, // 添加常量列表
        classes, // 添加类列表
        interfaces, // 添加接口列表
        enums, // 添加枚举列表
    })
}

/// 收集所有错误的程序解析函数
pub fn parse_program_collect_all_errors(parser: &mut ParserBase, errors: &mut Vec<String>) {
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
        } else if parser.peek() == Some(&"class".to_string()) || parser.peek() == Some(&"abstract".to_string()) {
            match parser.parse_class() {
                Ok(_) => try_next_item = true,
                Err(error) => {
                    errors.push(error);
                    // 跳过当前类，尝试在下一个关键字处继续解析
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                }
            }
        } else if parser.peek() == Some(&"interface".to_string()) {
            match parser.parse_interface() {
                Ok(_) => try_next_item = true,
                Err(error) => {
                    errors.push(error);
                    // 跳过当前接口，尝试在下一个关键字处继续解析
                    skip_to_next_top_level_item(parser);
                    try_next_item = parser.position < parser.tokens.len();
                }
            }
        } else if parser.peek() == Some(&"enum".to_string()) {
            match parser.parse_enum() {
                Ok(_) => try_next_item = true,
                Err(error) => {
                    errors.push(error);
                    // 跳过当前枚举，尝试在下一个关键字处继续解析
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
            errors.push(format!("期望 'fn', 'ns', 'class', 'abstract', 'interface' 或 'using', 但得到了 {:?} (位置: {})", parser.peek(), parser.position));
            skip_to_next_top_level_item(parser);
            try_next_item = parser.position < parser.tokens.len();
        }
    }
} 