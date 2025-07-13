use crate::ast::{Expression, BinaryOperator, CompareOperator, LogicalOperator};
use crate::parser::parser_base::ParserBase;

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
        self.parse_logical_expression()
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
                _ => {
                    // 变量或函数调用
                    let name = self.consume().unwrap();
                    
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
                        
                        // 检查是否是库函数调用
                        if name.starts_with("lib_") {
                            // 库函数调用，格式为 lib_xxx::func_name
                            let lib_name = name.trim_start_matches("lib_").to_string();
                            
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