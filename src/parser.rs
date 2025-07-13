use crate::ast::{Program, Function, Statement, Expression, Type, BinaryOperator, Parameter, Namespace};

pub fn parse(source: &str, debug: bool) -> Result<Program, String> {
    // 预处理：移除注释
    let source_without_comments = remove_comments(source);
    let mut parser = Parser::new(&source_without_comments, debug);
    parser.parse_program()
}

// 移除注释
fn remove_comments(source: &str) -> String {
    let mut result = String::new();
    let mut in_comment = false;
    let mut i = 0;
    
    let chars: Vec<char> = source.chars().collect();
    
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            // 找到注释开始
            in_comment = true;
            i += 2;
        } else if in_comment && chars[i] == '\n' {
            // 注释结束
            in_comment = false;
            result.push(chars[i]);
            i += 1;
        } else if !in_comment {
            // 非注释内容
            result.push(chars[i]);
            i += 1;
        } else {
            // 在注释内，跳过
            i += 1;
        }
    }
    
    result
}

struct Parser {
    source: String,
    tokens: Vec<String>,
    position: usize,
    debug: bool,
}

impl Parser {
    fn new(source: &str, debug: bool) -> Self {
        // 这里简化了词法分析，实际应该使用更复杂的词法分析器
        // 预处理字符串，保留字符串字面量
        let mut processed_source = String::new();
        let mut in_string = false;
        let mut escape = false;
        let mut current_string = String::new();
        let mut string_placeholders = Vec::new();
        
        // 先处理字符串
        for c in source.chars() {
            if in_string {
                if escape {
                    current_string.push(c);
                    escape = false;
                } else if c == '\\' {
                    current_string.push(c);
                    escape = true;
                } else if c == '"' {
                    in_string = false;
                    string_placeholders.push(current_string.clone());
                    processed_source.push_str(&format!(" __STRING_{} ", string_placeholders.len() - 1));
                    current_string.clear();
                } else {
                    current_string.push(c);
                }
            } else if c == '"' {
                in_string = true;
            } else {
                processed_source.push(c);
            }
        }
        
        // 特殊处理命名空间分隔符，确保它被当作一个整体处理
        processed_source = processed_source.replace("::", " __NS_SEP__ ");
        
        // 特殊处理复合操作符，必须在处理单个符号之前
        processed_source = processed_source
            .replace("++", " __INC_OP__ ")
            .replace("--", " __DEC_OP__ ")
            .replace("+=", " __ADD_ASSIGN__ ")
            .replace("-=", " __SUB_ASSIGN__ ")
            .replace("*=", " __MUL_ASSIGN__ ")
            .replace("/=", " __DIV_ASSIGN__ ")
            .replace("%=", " __MOD_ASSIGN__ ");
        
        // 处理其他分隔符
        let mut processed = processed_source
            .replace(";", " ; ")
            .replace("(", " ( ")
            .replace(")", " ) ")
            .replace("{", " { ")
            .replace("}", " } ")
            .replace(":", " : ")
            .replace("=", " = ")
            .replace("+", " + ")
            .replace("-", " - ")
            .replace("*", " * ")
            .replace("/", " / ")
            .replace("%", " % ")
            .replace("[", " [ ")
            .replace("]", " ] ")
            .replace(",", " , ")
            .replace("<", " < ")
            .replace(">", " > ")
            .split_whitespace()
            .map(|s| {
                if s.starts_with("__STRING_") {
                    let idx = s.trim_start_matches("__STRING_").parse::<usize>().unwrap();
                    format!("\"{}\"", string_placeholders[idx])
                } else if s == "__NS_SEP__" {
                    "::".to_string()
                } else if s == "__INC_OP__" {
                    "++".to_string()
                } else if s == "__DEC_OP__" {
                    "--".to_string()
                } else if s == "__ADD_ASSIGN__" {
                    "+=".to_string()
                } else if s == "__SUB_ASSIGN__" {
                    "-=".to_string()
                } else if s == "__MUL_ASSIGN__" {
                    "*=".to_string()
                } else if s == "__DIV_ASSIGN__" {
                    "/=".to_string()
                } else if s == "__MOD_ASSIGN__" {
                    "%=".to_string()
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<String>>();
        
        if debug {
            println!("词法分析结果: {:?}", processed);
        }
        
        Parser {
            source: source.to_string(),
            tokens: processed,
            position: 0,
            debug,
        }
    }
    
    fn peek(&self) -> Option<&String> {
        self.tokens.get(self.position)
    }
    
    fn consume(&mut self) -> Option<String> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }
    
    fn expect(&mut self, expected: &str) -> Result<(), String> {
        if let Some(token) = self.consume() {
            if token == expected {
                Ok(())
            } else {
                Err(format!("期望 '{}', 但得到了 '{}'", expected, token))
            }
        } else {
            Err(format!("期望 '{}', 但到达了文件末尾", expected))
        }
    }
    
    fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        let mut namespaces = Vec::new();
        
        while self.position < self.tokens.len() {
            if self.peek() == Some(&"ns".to_string()) {
                namespaces.push(self.parse_namespace()?);
            } else if self.peek() == Some(&"fn".to_string()) {
                functions.push(self.parse_function()?);
            } else {
                return Err(format!("期望 'fn' 或 'ns', 但得到了 '{:?}'", self.peek()));
            }
        }
        
        Ok(Program { functions, namespaces })
    }
    
    fn parse_namespace(&mut self) -> Result<Namespace, String> {
        self.expect("ns")?;
        
        let name = match self.consume() {
            Some(name) => name,
            None => return Err("期望命名空间名".to_string()),
        };
        
        self.expect("{")?;
        
        let mut functions = Vec::new();
        let mut namespaces = Vec::new();
        
        while let Some(token) = self.peek() {
            if token == "}" {
                break;
            } else if token == "fn" {
                functions.push(self.parse_function()?);
            } else if token == "ns" {
                namespaces.push(self.parse_namespace()?);
            } else {
                return Err(format!("期望 'fn', 'ns' 或 '}}', 但得到了 '{}'", token));
            }
        }
        
        self.expect("}")?;
        self.expect(";")?;
        
        Ok(Namespace { name, functions, namespaces })
    }
    
    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect("fn")?;
        
        let name = match self.consume() {
            Some(name) => name,
            None => return Err("期望函数名".to_string()),
        };
        
        self.expect("(")?;
        
        // 解析函数参数
        let mut parameters = Vec::new();
        if self.peek() != Some(&")".to_string()) {
            // 至少有一个参数
            let param_name = self.consume().ok_or_else(|| "期望参数名".to_string())?;
            self.expect(":")?;
            let param_type = self.parse_type()?;
            parameters.push(Parameter {
                name: param_name,
                param_type,
            });
            
            // 解析剩余参数
            while self.peek() == Some(&",".to_string()) {
                self.consume(); // 消费逗号
                let param_name = self.consume().ok_or_else(|| "期望参数名".to_string())?;
                self.expect(":")?;
                let param_type = self.parse_type()?;
                parameters.push(Parameter {
                    name: param_name,
                    param_type,
                });
            }
        }
        
        self.expect(")")?;
        
        self.expect(":")?;
        let return_type = self.parse_type()?;
        
        self.expect("{")?;
        
        let mut body = Vec::new();
        while let Some(token) = self.peek() {
            if token == "}" {
                break;
            }
            body.push(self.parse_statement()?);
        }
        
        self.expect("}")?;
        self.expect(";")?;
        
        Ok(Function {
            name,
            parameters,
            return_type,
            body,
        })
    }
    
    fn parse_type(&mut self) -> Result<Type, String> {
        let type_name = self.consume().ok_or_else(|| "期望类型名".to_string())?;
        
        match type_name.as_str() {
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "long" => Ok(Type::Long),
            "int[]" => Ok(Type::Array(Box::new(Type::Int))),
            "float[]" => Ok(Type::Array(Box::new(Type::Float))),
            "bool[]" => Ok(Type::Array(Box::new(Type::Bool))),
            "string[]" => Ok(Type::Array(Box::new(Type::String))),
            "long[]" => Ok(Type::Array(Box::new(Type::Long))),
            "Map" => {
                self.expect("<")?;
                let key_type = self.parse_type()?;
                self.expect(",")?;
                let value_type = self.parse_type()?;
                self.expect(">")?;
                Ok(Type::Map(Box::new(key_type), Box::new(value_type)))
            },
            _ => Err(format!("不支持的类型: {}", type_name)),
        }
    }
    
    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek() {
            Some(token) if token == "return" => {
                self.consume(); // 消费 "return"
                let expr = self.parse_expression()?;
                self.expect(";")?;
                Ok(Statement::Return(expr))
            },
            Some(token) if token == "using" => {
                self.consume(); // 消费 "using"
                self.expect("ns")?; // 期望 "ns" 关键字
                
                // 解析命名空间路径
                let mut path = Vec::new();
                let first_name = self.consume().ok_or_else(|| "期望命名空间名".to_string())?;
                path.push(first_name);
                
                // 解析嵌套命名空间路径
                while self.peek() == Some(&"::".to_string()) {
                    self.consume(); // 消费 "::"
                    let name = self.consume().ok_or_else(|| "期望命名空间名".to_string())?;
                    path.push(name);
                }
                
                self.expect(";")?;
                Ok(Statement::UsingNamespace(path))
            },
            Some(_) => {
                // 检查是否是变量声明或赋值
                let var_name = self.consume().unwrap();
                
                if let Some(next_token) = self.peek() {
                    if next_token == ":" {
                        self.consume(); // 消费 ":"
                        
                        // 检查是否是数组类型
                        let base_type = self.consume().ok_or_else(|| "期望类型名".to_string())?;
                        let var_type = if self.peek() == Some(&"[".to_string()) {
                            self.consume(); // 消费 "["
                            self.expect("]")?;
                            
                            match base_type.as_str() {
                                "int" => Type::Array(Box::new(Type::Int)),
                                "float" => Type::Array(Box::new(Type::Float)),
                                "bool" => Type::Array(Box::new(Type::Bool)),
                                "string" => Type::Array(Box::new(Type::String)),
                                "long" => Type::Array(Box::new(Type::Long)),
                                _ => return Err(format!("不支持的数组元素类型: {}", base_type)),
                            }
                        } else if base_type == "Map" {
                            self.expect("<")?;
                            let key_type = self.parse_type()?;
                            self.expect(",")?;
                            let value_type = self.parse_type()?;
                            self.expect(">")?;
                            Type::Map(Box::new(key_type), Box::new(value_type))
                        } else {
                            match base_type.as_str() {
                                "int" => Type::Int,
                                "float" => Type::Float,
                                "bool" => Type::Bool,
                                "string" => Type::String,
                                "long" => Type::Long,
                                _ => return Err(format!("不支持的类型: {}", base_type)),
                            }
                        };
                        
                        self.expect("=")?;
                        let init_expr = self.parse_expression()?;
                        self.expect(";")?;
                        Ok(Statement::VariableDeclaration(var_name, var_type, init_expr))
                    } else if next_token == "=" {
                        // 变量赋值
                        self.consume(); // 消费 "="
                        let value_expr = self.parse_expression()?;
                        self.expect(";")?;
                        Ok(Statement::VariableAssignment(var_name, value_expr))
                    } else if next_token == "+=" || next_token == "-=" || next_token == "*=" || next_token == "/=" || next_token == "%=" {
                        // 复合赋值
                        let op_token = self.consume().unwrap();
                        let operator = match op_token.as_str() {
                            "+=" => BinaryOperator::Add,
                            "-=" => BinaryOperator::Subtract,
                            "*=" => BinaryOperator::Multiply,
                            "/=" => BinaryOperator::Divide,
                            "%=" => BinaryOperator::Modulo,
                            _ => unreachable!(),
                        };
                        
                        let value_expr = self.parse_expression()?;
                        self.expect(";")?;
                        Ok(Statement::CompoundAssignment(var_name, operator, value_expr))
                    } else if next_token == "++" {
                        // 自增操作
                        self.consume(); // 消费 "++"
                        self.expect(";")?;
                        Ok(Statement::Increment(var_name))
                    } else if next_token == "--" {
                        // 自减操作
                        self.consume(); // 消费 "--"
                        self.expect(";")?;
                        Ok(Statement::Decrement(var_name))
                    } else {
                        Err(format!("不支持的语句: {} {}", var_name, next_token))
                    }
                } else {
                    Err("不完整的语句".to_string())
                }
            },
            None => Err("期望语句".to_string()),
        }
    }
    
    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_additive_expression()
    }
    
    fn parse_additive_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplicative_expression()?;
        
        while let Some(op) = self.peek() {
            if op == "+" || op == "-" {
                let operator = match op.as_str() {
                    "+" => BinaryOperator::Add,
                    "-" => BinaryOperator::Subtract,
                    _ => unreachable!(),
                };
                self.consume(); // 消费操作符
                let right = self.parse_multiplicative_expression()?;
                left = Expression::BinaryOp(Box::new(left), operator, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(left)
    }
    
    fn parse_multiplicative_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_primary_expression()?;
        
        while let Some(op) = self.peek() {
            if op == "*" || op == "/" || op == "%" {
                let operator = match op.as_str() {
                    "*" => BinaryOperator::Multiply,
                    "/" => BinaryOperator::Divide,
                    "%" => BinaryOperator::Modulo,
                    _ => unreachable!(),
                };
                self.consume(); // 消费操作符
                let right = self.parse_primary_expression()?;
                left = Expression::BinaryOp(Box::new(left), operator, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(left)
    }
    
    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        match self.peek() {
            Some(token) => {
                if token == "(" {
                    self.consume(); // 消费左括号
                    let expr = self.parse_expression()?;
                    self.expect(")")?;
                    Ok(expr)
                } else if token == "[" {
                    // 解析数组字面量
                    self.consume(); // 消费 "["
                    let mut elements = Vec::new();
                    
                    if let Some(next) = self.peek() {
                        if next != "]" {
                            elements.push(self.parse_expression()?);
                            
                            while let Some(token) = self.peek() {
                                if token == "]" {
                                    break;
                                }
                                self.expect(",")?;
                                elements.push(self.parse_expression()?);
                            }
                        }
                    }
                    
                    self.expect("]")?;
                    Ok(Expression::ArrayLiteral(elements))
                } else if token == "{" {
                    // 解析映射字面量
                    self.consume(); // 消费 "{"
                    let mut entries = Vec::new();
                    
                    if let Some(next) = self.peek() {
                        if next != "}" {
                            let key = self.parse_expression()?;
                            self.expect(":")?;
                            let value = self.parse_expression()?;
                            entries.push((key, value));
                            
                            while let Some(token) = self.peek() {
                                if token == "}" {
                                    break;
                                }
                                self.expect(",")?;
                                let key = self.parse_expression()?;
                                self.expect(":")?;
                                let value = self.parse_expression()?;
                                entries.push((key, value));
                            }
                        }
                    }
                    
                    self.expect("}")?;
                    Ok(Expression::MapLiteral(entries))
                } else if token == "::" {
                    // 解析全局函数调用
                    self.consume(); // 消费 "::"
                    
                    // 获取函数名
                    let func_name = self.consume().ok_or_else(|| "期望函数名".to_string())?;
                    
                    self.expect("(")?;
                    
                    // 解析函数调用参数
                    let mut args = Vec::new();
                    if self.peek() != Some(&")".to_string()) {
                        // 至少有一个参数
                        args.push(self.parse_expression()?);
                        
                        // 解析剩余参数
                        while self.peek() == Some(&",".to_string()) {
                            self.consume(); // 消费逗号
                            args.push(self.parse_expression()?);
                        }
                    }
                    
                    self.expect(")")?;
                    Ok(Expression::GlobalFunctionCall(func_name, args))
                } else if token == "true" {
                    self.consume();
                    Ok(Expression::BoolLiteral(true))
                } else if token == "false" {
                    self.consume();
                    Ok(Expression::BoolLiteral(false))
                } else if let Ok(value) = token.parse::<i32>() {
                    self.consume();
                    Ok(Expression::IntLiteral(value))
                } else if let Ok(value) = token.parse::<i64>() {
                    // 检查是否是长整型（超过i32范围）
                    if value > i32::MAX as i64 || value < i32::MIN as i64 {
                        self.consume();
                        Ok(Expression::LongLiteral(value))
                    } else {
                        self.consume();
                        Ok(Expression::IntLiteral(value as i32))
                    }
                } else if let Ok(value) = token.parse::<f64>() {
                    self.consume();
                    Ok(Expression::FloatLiteral(value))
                } else if token.starts_with("\"") && token.ends_with("\"") {
                    // 字符串字面量
                    let token_clone = token.clone();
                    self.consume();
                    let content = token_clone[1..token_clone.len()-1].to_string();
                    Ok(Expression::StringLiteral(content))
                } else if let Some(next_token) = self.tokens.get(self.position + 1) {
                    if next_token == "(" {
                        let func_name = self.consume().unwrap();
                        self.consume(); // 消费 "("
                        
                        // 解析函数调用参数
                        let mut args = Vec::new();
                        if self.peek() != Some(&")".to_string()) {
                            // 至少有一个参数
                            args.push(self.parse_expression()?);
                            
                            // 解析剩余参数
                            while self.peek() == Some(&",".to_string()) {
                                self.consume(); // 消费逗号
                                args.push(self.parse_expression()?);
                            }
                        }
                        
                        self.expect(")")?;
                        Ok(Expression::FunctionCall(func_name, args))
                    } else if next_token == "::" {
                        // 解析命名空间函数调用
                        let mut path = Vec::new();
                        path.push(self.consume().unwrap()); // 第一个命名空间名
                        
                        // 解析命名空间路径
                        while self.peek() == Some(&"::".to_string()) {
                            self.consume(); // 消费 "::"
                            if let Some(name) = self.consume() {
                                path.push(name);
                            } else {
                                return Err("期望标识符".to_string());
                            }
                            
                            // 如果下一个不是 "::" 或 "("，则结束路径解析
                            if self.peek() != Some(&"::".to_string()) && self.peek() != Some(&"(".to_string()) {
                                break;
                            }
                        }
                        
                        // 最后一个是函数名
                        if self.peek() == Some(&"(".to_string()) {
                            self.consume(); // 消费 "("
                            
                            // 解析函数调用参数
                            let mut args = Vec::new();
                            if self.peek() != Some(&")".to_string()) {
                                // 至少有一个参数
                                args.push(self.parse_expression()?);
                                
                                // 解析剩余参数
                                while self.peek() == Some(&",".to_string()) {
                                    self.consume(); // 消费逗号
                                    args.push(self.parse_expression()?);
                                }
                            }
                            
                            self.expect(")")?;
                            Ok(Expression::NamespacedFunctionCall(path, args))
                        } else {
                            Err("期望 '('".to_string())
                        }
                    } else {
                        // 变量引用
                        let var_name = self.consume().unwrap();
                        Ok(Expression::Variable(var_name))
                    }
                } else {
                    // 最后一个token，可能是变量
                    let var_name = self.consume().unwrap();
                    Ok(Expression::Variable(var_name))
                }
            },
            None => Err("期望表达式".to_string()),
        }
    }
}