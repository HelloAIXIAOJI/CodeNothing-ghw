use crate::ast::{Expression, BinaryOperator, CompareOperator, LogicalOperator, Parameter, Type, Statement};
use crate::parser::parser_base::ParserBase;
use crate::parser::pointer_parser::PointerParser;
use crate::interpreter::debug_println;

pub trait ExpressionParser {
    fn parse_expression(&mut self) -> Result<Expression, String>;
    fn parse_logical_expression(&mut self) -> Result<Expression, String>;
    fn parse_compare_expression(&mut self) -> Result<Expression, String>;
    fn parse_additive_expression(&mut self) -> Result<Expression, String>;
    fn parse_multiplicative_expression(&mut self) -> Result<Expression, String>;
    fn parse_unary_expression(&mut self) -> Result<Expression, String>;
    fn parse_primary_expression(&mut self) -> Result<Expression, String>;
    fn parse_expression_type(&mut self) -> Result<Type, String>;
    fn is_lambda_parameter_list(&self) -> bool;
    fn peek_ahead(&self, offset: usize) -> Option<&String>;
}

impl<'a> ExpressionParser for ParserBase<'a> {
    fn parse_expression(&mut self) -> Result<Expression, String> {
        // 解析条件表达式（三元运算符）
        let expr = self.parse_logical_expression()?;
        
        // 检查是否是三元运算符
        if self.peek() == Some(&"?".to_string()) {
            self.consume(); // 消费 "?"
            
            // 解析条件为真时的表达式
            let true_expr = self.parse_expression()?;
            
            // 期望 ":"
            self.expect(":")?;
            
            // 解析条件为假时的表达式
            let false_expr = self.parse_expression()?;
            
            // 构建三元运算符表达式
            return Ok(Expression::TernaryOp(
                Box::new(expr),
                Box::new(true_expr),
                Box::new(false_expr)
            ));
        }
        
        Ok(expr)
    }
    
    fn parse_logical_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_compare_expression()?;
        
        while let Some(op) = self.peek() {
            if op == "&&" || op == "||" {
                let operator = match op.as_str() {
                    "&&" => LogicalOperator::And,
                    "||" => LogicalOperator::Or,
                    _ => unreachable!(),
                };
                self.consume(); // 消费操作符
                let right = self.parse_compare_expression()?;
                left = Expression::LogicalOp(Box::new(left), operator, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(left)
    }
    
    fn parse_compare_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_additive_expression()?;
        
        while let Some(op) = self.peek() {
            if op == "==" || op == "!=" || op == ">" || op == "<" || op == ">=" || op == "<=" {
                let operator = match op.as_str() {
                    "==" => CompareOperator::Equal,
                    "!=" => CompareOperator::NotEqual,
                    ">" => CompareOperator::Greater,
                    "<" => CompareOperator::Less,
                    ">=" => CompareOperator::GreaterEqual,
                    "<=" => CompareOperator::LessEqual,
                    _ => unreachable!(),
                };
                self.consume(); // 消费操作符
                let right = self.parse_additive_expression()?;
                left = Expression::CompareOp(Box::new(left), operator, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(left)
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
        let mut left = self.parse_unary_expression()?;
        
        while let Some(op) = self.peek() {
            if op == "*" || op == "/" || op == "%" {
                let operator = match op.as_str() {
                    "*" => BinaryOperator::Multiply,
                    "/" => BinaryOperator::Divide,
                    "%" => BinaryOperator::Modulo,
                    _ => unreachable!(),
                };
                self.consume(); // 消费操作符
                let right = self.parse_unary_expression()?;
                left = Expression::BinaryOp(Box::new(left), operator, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(left)
    }
    
    fn parse_unary_expression(&mut self) -> Result<Expression, String> {
        if let Some(op) = self.peek() {
            if op == "!" {
                self.consume(); // 消费操作符
                let expr = self.parse_unary_expression()?;
                return Ok(Expression::LogicalOp(Box::new(expr), LogicalOperator::Not, Box::new(Expression::BoolLiteral(false))));
            } else if op == "&" {
                // 取地址操作
                return self.parse_address_of();
            } else if op == "*" {
                // 解引用操作
                return self.parse_dereference();
            } else if op == "++" {
                // 前置自增
                self.consume(); // 消费 "++"
                if let Some(var_name) = self.peek() {
                    let var = self.consume().unwrap();
                    return Ok(Expression::PreIncrement(var));
                } else {
                    return Err("前置自增操作符后期望变量名".to_string());
                }
            } else if op == "--" {
                // 前置自减
                self.consume(); // 消费 "--"
                if let Some(var_name) = self.peek() {
                    let var = self.consume().unwrap();
                    return Ok(Expression::PreDecrement(var));
                } else {
                    return Err("前置自减操作符后期望变量名".to_string());
                }
            } else if op == "throw" {
                // throw 表达式
                self.consume(); // 消费 "throw"
                let exception_expr = self.parse_primary_expression()?;
                return Ok(Expression::Throw(Box::new(exception_expr)));
            }
        }
        
        self.parse_primary_expression()
    }
    
    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        if let Some(token) = self.peek() {
            match token.as_str() {
                "INTERP_START" => {
                    // 解析字符串插值
                    self.consume(); // 消费 "INTERP_START"
                    
                    let mut segments = Vec::new();
                    while self.peek() != Some(&"INTERP_END".to_string()) {
                        let token = self.consume().ok_or_else(|| "期望字符串插值片段".to_string())?;
                        
                        if token.starts_with("INTERP_TEXT:") {
                            // 文本片段
                            let text = token.strip_prefix("INTERP_TEXT:").unwrap_or("").to_string();
                            segments.push(crate::ast::StringInterpolationSegment::Text(text));
                        } else if token.starts_with("INTERP_EXPR:") {
                            // 表达式片段
                            let expr_str = token.strip_prefix("INTERP_EXPR:").unwrap_or("").to_string();
                            
                            // 创建临时解析器处理表达式
                            let mut temp_tokens = crate::parser::lexer::tokenize(&expr_str, false);
                            let mut temp_parser = ParserBase::new(&expr_str, temp_tokens, false);
                            
                            let expr = match temp_parser.parse_expression() {
                                Ok(e) => e,
                                Err(e) => return Err(format!("解析插值表达式错误: {}", e)),
                            };
                            
                            segments.push(crate::ast::StringInterpolationSegment::Expression(Box::new(expr)));
                        } else {
                            return Err(format!("未知的字符串插值片段: {}", token));
                        }
                    }
                    
                    self.consume(); // 消费 "INTERP_END"
                    return Ok(Expression::StringInterpolation(segments));
                },
                "(" => {
                    // 检查是否是多参数Lambda表达式: (x, y) => expr
                    if self.is_lambda_parameter_list() {
                        self.consume(); // 消费 "("
                        let mut params = Vec::new();
                        
                        // 解析参数列表
                        if self.peek() != Some(&")".to_string()) {
                            loop {
                                let param_name = self.consume().ok_or_else(|| "期望参数名".to_string())?;
                                let param_type = if self.peek() == Some(&":".to_string()) {
                                    self.consume(); // 消费 ":"
                                    self.parse_expression_type()?
                                } else {
                                    Type::Auto // 默认auto类型
                                };
                                
                                // 检查是否有默认值
                                let default_value = if self.peek() == Some(&"=".to_string()) {
                                    self.consume(); // 消费 "="
                                    Some(self.parse_expression()?)
                                } else {
                                    None
                                };
                                
                                params.push(Parameter {
                                    name: param_name,
                                    param_type,
                                    default_value,
                                });
                                
                                if self.peek() != Some(&",".to_string()) {
                                    break;
                                }
                                self.consume(); // 消费 ","
                            }
                        }
                        
                        self.expect(")")?;
                        self.expect("=>")?;
                        
                        // 检查是否是块表达式
                        if self.peek() == Some(&"{".to_string()) {
                            // Lambda块: (x, y) => { statements }
                            self.consume(); // 消费 "{"
                            let mut statements = Vec::new();
                            
                            while self.peek() != Some(&"}".to_string()) {
                                use crate::parser::statement_parser::StatementParser;
                                statements.push(StatementParser::parse_statement(self)?);
                            }
                            
                            self.expect("}")?;
                            return Ok(Expression::LambdaBlock(params, statements));
                        } else {
                            // Lambda表达式: (x, y) => expr
                            let body = self.parse_expression()?;
                            return Ok(Expression::Lambda(params, Box::new(body)));
                        }
                    }
                    
                    // 普通括号表达式
                    self.consume(); // 消费左括号
                    let expr = self.parse_expression()?;
                    self.expect(")")?;
                    Ok(expr)
                },
                "[" => {
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
                },
                "{" => {
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
                },
                "::" => {
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
                },
                "true" => {
                    self.consume();
                    Ok(Expression::BoolLiteral(true))
                },
                "false" => {
                    self.consume();
                    Ok(Expression::BoolLiteral(false))
                },
                "new" => {
                    // 解析对象创建: new ClassName(args)
                    self.consume(); // 消费 "new"
                    let class_name = self.consume().ok_or_else(|| "期望类名".to_string())?;
                    self.expect("(")?;
                    
                    let mut args = Vec::new();
                    if self.peek() != Some(&")".to_string()) {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.peek() != Some(&",".to_string()) {
                                break;
                            }
                            self.consume(); // 消费 ","
                        }
                    }
                    self.expect(")")?;
                    
                    Ok(Expression::ObjectCreation(class_name, args))
                },
                _ => {
                    // 检查是否是字符串字面量
                    if token.starts_with('"') && token.ends_with('"') {
                        let string_value = token[1..token.len()-1].to_string();
                        self.consume();
                        return Ok(Expression::StringLiteral(string_value));
                    }
                    
                    // 检查是否是原始字符串字面量
                    if token.starts_with("r\"") && token.ends_with('"') {
                        let string_value = token[2..token.len()-1].to_string();
                        self.consume();
                        return Ok(Expression::RawStringLiteral(string_value));
                    }
                    
                    // 检查是否是数字字面量
                    if let Ok(int_value) = token.parse::<i32>() {
                        self.consume();
                        return Ok(Expression::IntLiteral(int_value));
                    } else if let Ok(float_value) = token.parse::<f64>() {
                        self.consume();
                        return Ok(Expression::FloatLiteral(float_value));
                    } else if token.ends_with('L') || token.ends_with('l') {
                        // 长整型字面量
                        if let Ok(long_value) = token[..token.len()-1].parse::<i64>() {
                            self.consume();
                            return Ok(Expression::LongLiteral(long_value));
                        }
                    }
                    
                    // 检查是否是Lambda表达式 (x => expr 或 x : int => expr)
                    if self.peek_ahead(1) == Some(&"=>".to_string()) {
                        // 单参数Lambda: x => expr
                        let param_name = self.consume().unwrap();
                        self.consume(); // 消费 "=>"

                        let param = Parameter {
                            name: param_name,
                            param_type: Type::Auto, // Lambda参数默认使用auto类型
                            default_value: None,
                        };

                        let body = self.parse_expression()?;
                        return Ok(Expression::Lambda(vec![param], Box::new(body)));
                    } else if self.peek_ahead(1) == Some(&":".to_string()) {
                        // 带类型的单参数Lambda: x : int => expr
                        let param_name = self.consume().unwrap();
                        self.consume(); // 消费 ":"
                        let param_type = self.parse_expression_type()?;

                        if self.peek() == Some(&"=>".to_string()) {
                            self.consume(); // 消费 "=>"

                            let param = Parameter {
                                name: param_name,
                                param_type,
                                default_value: None,
                            };

                            let body = self.parse_expression()?;
                            return Ok(Expression::Lambda(vec![param], Box::new(body)));
                        }
                        // 如果不是Lambda，回退处理（这里简化处理）
                        return Err("期望 '=>' 在类型注解后".to_string());
                    }
                    
                    // 变量或函数调用
                    let name = self.consume().unwrap();
                    
                    // 调试输出
                    debug_println(&format!("解析标识符: {}", name));
                    debug_println(&format!("下一个token: {:?}", self.peek()));
                    
                    if self.peek() == Some(&"(".to_string()) {
                        // 函数调用
                        self.consume(); // 消费 "("
                        
                        let mut args = Vec::new();
                        
                        if self.peek() != Some(&")".to_string()) {
                            // 解析参数列表
                            loop {
                                let arg = self.parse_expression()?;
                                args.push(arg);
                                
                                if self.peek() != Some(&",".to_string()) {
                                    break;
                                }
                                
                                self.consume(); // 消费 ","
                            }
                        }
                        
                        self.expect(")")?;
                        
                        Ok(Expression::FunctionCall(name, args))
                    } else if self.peek() == Some(&"::".to_string()) {
                        // 静态访问、命名空间函数调用或库函数调用
                        self.consume(); // 消费 "::"
                        
                        // 获取成员名或函数名
                        let member_name = self.consume().ok_or_else(|| "期望成员名或函数名".to_string())?;
                        debug_println(&format!("解析静态访问或库函数调用: {}::{}", name, member_name));
                        
                        // 构建完整的命名空间路径
                        let mut path = Vec::new();
                        path.push(name.clone());
                        path.push(member_name.clone());
                        
                        // 继续解析更多的 :: 和标识符
                        while self.peek() == Some(&"::".to_string()) {
                            self.consume(); // 消费 "::"
                            let next_name = self.consume().ok_or_else(|| "期望命名空间或函数名".to_string())?;
                            path.push(next_name.clone());
                        }
                        
                        // 检查下一个token来决定是静态访问还是函数调用
                        if self.peek() == Some(&"(".to_string()) {
                            // 这是一个函数调用
                            // 检查是否是库函数调用
                            if name.starts_with("lib_") && path.len() == 2 {
                                // 库函数调用，格式为 lib_xxx::func_name
                                let lib_name = name.trim_start_matches("lib_").to_string();
                                debug_println(&format!("识别为库函数调用: {} -> {}", lib_name, member_name));
                                
                                self.expect("(")?;
                                
                                let mut args = Vec::new();
                                
                                if self.peek() != Some(&")".to_string()) {
                                    // 解析参数列表
                                    loop {
                                        let arg = self.parse_expression()?;
                                        args.push(arg);
                                        
                                        if self.peek() != Some(&",".to_string()) {
                                            break;
                                        }
                                        
                                        self.consume(); // 消费 ","
                                    }
                                }
                                
                                self.expect(")")?;
                                
                                Ok(Expression::LibraryFunctionCall(lib_name, member_name, args))
                            } else {
                                // 静态方法调用或命名空间函数调用
                                debug_println(&format!("识别为静态方法调用或命名空间函数调用，路径: {:?}", path));
                                
                                // 期望 "("
                                self.expect("(")?;
                                
                                let mut args = Vec::new();
                                
                                if self.peek() != Some(&")".to_string()) {
                                    // 解析参数列表
                                    loop {
                                        let arg = self.parse_expression()?;
                                        args.push(arg);
                                        
                                        if self.peek() != Some(&",".to_string()) {
                                            break;
                                        }
                                        
                                        self.consume(); // 消费 ","
                                    }
                                }
                                
                                self.expect(")")?;
                                
                                // 需要更智能地区分静态方法调用和命名空间函数调用
                                // 检查第一个标识符是否是已知的类名或命名空间
                                if path.len() == 2 {
                                    // 对于两个部分的情况，我们需要在运行时决定
                                    // 暂时都当作命名空间函数调用处理，让解释器来区分
                                    debug_println(&format!("两部分路径，当作命名空间函数调用处理: {:?}", path));
                                    Ok(Expression::NamespacedFunctionCall(path, args))
                                } else {
                                    // 多于两个部分，肯定是命名空间函数调用
                                    debug_println(&format!("使用NamespacedFunctionCall处理: {:?}", path));
                                    Ok(Expression::NamespacedFunctionCall(path, args))
                                }
                            }
                        } else {
                            // 这是静态访问（不是函数调用）
                            // 可能是：1. 静态成员访问 2. 枚举变体访问 3. 命名空间中的常量或变量访问
                            if path.len() == 2 {
                                debug_println(&format!("识别为静态访问或枚举变体访问: {}::{}", name, member_name));

                                // 检查是否有参数（枚举变体创建）
                                if self.peek() == Some(&"(".to_string()) {
                                    // 枚举变体创建：EnumName::Variant(args)
                                    self.consume(); // 消费 "("

                                    let mut args = Vec::new();

                                    if self.peek() != Some(&")".to_string()) {
                                        // 解析参数列表
                                        loop {
                                            let arg = self.parse_expression()?;
                                            args.push(arg);

                                            if self.peek() != Some(&",".to_string()) {
                                                break;
                                            }

                                            self.consume(); // 消费 ","
                                        }
                                    }

                                    self.expect(")")?;

                                    debug_println(&format!("识别为枚举变体创建: {}::{}({} args)", name, member_name, args.len()));
                                    Ok(Expression::EnumVariantCreation(name, member_name, args))
                                } else {
                                    // 静态访问或枚举变体访问（无参数）
                                    debug_println(&format!("识别为静态访问或枚举变体访问: {}::{}", name, member_name));
                                    Ok(Expression::EnumVariantAccess(name, member_name))
                                }
                            } else {
                                // 多层命名空间访问，暂时不支持
                                return Err(format!("不支持多层命名空间访问: {:?}", path));
                            }
                        }
                    } else if self.peek() == Some(&"++".to_string()) {
                        // 后置自增
                        let var_name = self.consume().unwrap();
                        self.consume(); // 消费 "++"
                        Ok(Expression::PostIncrement(var_name))
                    } else if self.peek() == Some(&"--".to_string()) {
                        // 后置自减
                        let var_name = self.consume().unwrap();
                        self.consume(); // 消费 "--"
                        Ok(Expression::PostDecrement(var_name))
                    } else if self.peek() == Some(&".".to_string()) {
                        // 字段访问或方法调用或链式调用
                        self.consume(); // 消费 "."
                        
                        // 获取方法名
                        let method_name = self.consume().ok_or_else(|| "期望方法名".to_string())?;
                        
                        // 检查是否有参数
                        if self.peek() == Some(&"(".to_string()) {
                            self.consume(); // 消费 "("
                            
                            let mut args = Vec::new();
                            
                            if self.peek() != Some(&")".to_string()) {
                                // 解析参数列表
                                loop {
                                    let arg = self.parse_expression()?;
                                    args.push(arg);
                                    
                                    if self.peek() != Some(&",".to_string()) {
                                        break;
                                    }
                                    
                                    self.consume(); // 消费 ","
                                }
                            }
                            
                            self.expect(")")?;
                            
                            // 检查是否有更多的链式调用
                            let mut all_calls = Vec::new();
                            all_calls.push((method_name.clone(), args.clone()));
                            
                            while self.peek() == Some(&".".to_string()) {
                                self.consume(); // 消费 "."
                                
                                let next_method = self.consume().ok_or_else(|| "期望方法名".to_string())?;
                                
                                if self.peek() == Some(&"(".to_string()) {
                                    self.consume(); // 消费 "("
                                    
                                    let mut next_args = Vec::new();
                                    
                                    if self.peek() != Some(&")".to_string()) {
                                        // 解析参数列表
                                        loop {
                                            let arg = self.parse_expression()?;
                                            next_args.push(arg);
                                            
                                            if self.peek() != Some(&",".to_string()) {
                                                break;
                                            }
                                            
                                            self.consume(); // 消费 ","
                                        }
                                    }
                                    
                                    self.expect(")")?;
                                    
                                    all_calls.push((next_method, next_args));
                                } else {
                                    return Err("方法调用后期望左括号".to_string());
                                }
                            }
                            
                            if all_calls.len() == 1 {
                                // 只有一个方法调用
                                Ok(Expression::MethodCall(Box::new(Expression::Variable(name.clone())), method_name, args))
                            } else {
                                // 多个方法调用，构建链式调用
                                Ok(Expression::ChainCall(Box::new(Expression::Variable(name)), all_calls))
                            }
                        } else {
                            // 字段访问
                            Ok(Expression::FieldAccess(Box::new(Expression::Variable(name)), method_name))
                        }
                    } else {
                        // 变量
                        Ok(Expression::Variable(name))
                    }
                }
            }
        } else {
            Err("期望表达式".to_string())
        }
    }
    
    fn parse_expression_type(&mut self) -> Result<Type, String> {
        if let Some(token) = self.peek() {
            match token.as_str() {
                "int" => {
                    self.consume();
                    Ok(Type::Int)
                },
                "float" => {
                    self.consume();
                    Ok(Type::Float)
                },
                "bool" => {
                    self.consume();
                    Ok(Type::Bool)
                },
                "string" => {
                    self.consume();
                    Ok(Type::String)
                },
                "long" => {
                    self.consume();
                    Ok(Type::Long)
                },
                "void" => {
                    self.consume();
                    Ok(Type::Void)
                },
                "auto" => {
                    self.consume();
                    Ok(Type::Auto)
                },
                "*" => {
                    // 指针类型或函数指针类型
                    self.consume(); // 消费 "*"

                    if self.peek() == Some(&"fn".to_string()) {
                        // 函数指针类型: *fn(int, int) : int
                        self.consume(); // 消费 "fn"
                        self.expect("(")?;

                        let mut param_types = Vec::new();
                        if self.peek() != Some(&")".to_string()) {
                            loop {
                                param_types.push(self.parse_expression_type()?);
                                if self.peek() != Some(&",".to_string()) {
                                    break;
                                }
                                self.consume(); // 消费 ","
                            }
                        }

                        self.expect(")")?;
                        self.expect(":")?; // 使用冒号而不是箭头
                        let return_type = Box::new(self.parse_expression_type()?);

                        Ok(Type::FunctionPointer(param_types, return_type))
                    } else {
                        // 普通指针类型: *int, *string 等
                        let target_type = Box::new(self.parse_expression_type()?);
                        Ok(Type::Pointer(target_type))
                    }
                },
                "?" => {
                    // 可选指针类型: ?*int 或 ?*fn(...)
                    self.consume(); // 消费 "?"

                    if self.peek() == Some(&"*".to_string()) {
                        self.consume(); // 消费 "*"

                        if self.peek() == Some(&"fn".to_string()) {
                            // 可选函数指针类型: ?*fn(int, int) : int
                            self.consume(); // 消费 "fn"
                            self.expect("(")?;

                            let mut param_types = Vec::new();
                            if self.peek() != Some(&")".to_string()) {
                                loop {
                                    param_types.push(self.parse_expression_type()?);
                                    if self.peek() != Some(&",".to_string()) {
                                        break;
                                    }
                                    self.consume(); // 消费 ","
                                }
                            }

                            self.expect(")")?;
                            self.expect(":")?;
                            let return_type = Box::new(self.parse_expression_type()?);

                            let func_ptr_type = Type::FunctionPointer(param_types, return_type);
                            Ok(Type::OptionalPointer(Box::new(func_ptr_type)))
                        } else {
                            // 普通可选指针类型: ?*int
                            let target_type = Box::new(self.parse_expression_type()?);
                            Ok(Type::OptionalPointer(target_type))
                        }
                    } else {
                        Err("期望 '*' 在 '?' 之后".to_string())
                    }
                },
                "fn" => {
                    // 函数类型: fn(int, string) -> bool
                    self.consume(); // 消费 "fn"
                    self.expect("(")?;
                    
                    let mut param_types = Vec::new();
                    if self.peek() != Some(&")".to_string()) {
                        loop {
                            param_types.push(self.parse_expression_type()?);
                            if self.peek() != Some(&",".to_string()) {
                                break;
                            }
                            self.consume(); // 消费 ","
                        }
                    }
                    
                    self.expect(")")?;
                    self.expect("->")?;
                    let return_type = Box::new(self.parse_expression_type()?);
                    
                    Ok(Type::Function(param_types, return_type))
                },
                _ => {
                    // 可能是类类型
                    let type_name = self.consume().unwrap();
                    Ok(Type::Class(type_name))
                }
            }
        } else {
            Err("期望类型".to_string())
        }
    }
    
    
    fn is_lambda_parameter_list(&self) -> bool {
        // 检查是否是Lambda参数列表: (param1, param2) => ...
        // 我们需要向前查看，找到匹配的右括号，然后检查是否有 "=>"
        let mut depth = 0;
        let mut pos = 0;
        
        // 跳过当前的 "("
        pos += 1;
        depth += 1;
        
        while let Some(token) = self.peek_ahead(pos) {
            match token.as_str() {
                "(" => depth += 1,
                ")" => {
                    depth -= 1;
                    if depth == 0 {
                        // 找到匹配的右括号，检查下一个token是否是 "=>"
                        return self.peek_ahead(pos + 1) == Some(&"=>".to_string());
                    }
                },
                _ => {}
            }
            pos += 1;
        }
        
        false
    }
    
    fn peek_ahead(&self, offset: usize) -> Option<&String> {
        self.tokens.get(self.position + offset)
    }
} 