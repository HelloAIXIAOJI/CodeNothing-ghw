use crate::ast::{Expression, BinaryOperator, CompareOperator, LogicalOperator};
use crate::parser::parser_base::ParserBase;
use crate::interpreter::debug_println;

pub trait ExpressionParser {
    fn parse_expression(&mut self) -> Result<Expression, String>;
    fn parse_logical_expression(&mut self) -> Result<Expression, String>;
    fn parse_compare_expression(&mut self) -> Result<Expression, String>;
    fn parse_additive_expression(&mut self) -> Result<Expression, String>;
    fn parse_multiplicative_expression(&mut self) -> Result<Expression, String>;
    fn parse_unary_expression(&mut self) -> Result<Expression, String>;
    fn parse_primary_expression(&mut self) -> Result<Expression, String>;
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
                "(" => {
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
                        // 命名空间函数调用或库函数调用
                        self.consume(); // 消费 "::"
                        
                        // 获取函数名
                        let func_name = self.consume().ok_or_else(|| "期望函数名".to_string())?;
                        debug_println(&format!("解析库函数调用: {}::{}", name, func_name));
                        
                        // 检查是否是库函数调用
                        if name.starts_with("lib_") {
                            // 库函数调用，格式为 lib_xxx::func_name
                            let lib_name = name.trim_start_matches("lib_").to_string();
                            debug_println(&format!("识别为库函数调用: {} -> {}", lib_name, func_name));
                            
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
                            
                            Ok(Expression::LibraryFunctionCall(lib_name, func_name, args))
                        } else {
                            // 命名空间函数调用
                            debug_println(&format!("识别为命名空间函数调用: {}::{}", name, func_name));
                            
                            // 构建完整的函数名称（包含命名空间）
                            let mut full_name = format!("{}::{}", name, func_name);
                            
                            // 检查是否有更多的命名空间层级
                            let mut path = Vec::new();
                            path.push(name);
                            path.push(func_name);
                            
                            while self.peek() == Some(&"::".to_string()) {
                                self.consume(); // 消费 "::"
                                let next_name = self.consume().ok_or_else(|| "期望命名空间或函数名".to_string())?;
                                path.push(next_name.clone());
                                full_name.push_str("::");
                                full_name.push_str(&next_name);
                            }
                            
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
                            
                            // 使用统一的接口处理所有命名空间函数调用，不再硬编码特定命名空间
                            debug_println(&format!("使用NamespacedFunctionCall处理: {:?}", path));
                            Ok(Expression::NamespacedFunctionCall(path, args))
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
} 