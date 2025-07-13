use crate::ast::{Statement, Type, BinaryOperator};
use crate::parser::parser_base::ParserBase;
use crate::parser::expression_parser::ExpressionParser;

pub trait StatementParser {
    fn parse_statement(&mut self) -> Result<Statement, String>;
    fn parse_type(&mut self) -> Result<Type, String>;
}

impl<'a> StatementParser for ParserBase<'a> {
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
            Some(token) if token == "if" => {
                self.consume(); // 消费 "if"
                
                // 解析条件
                self.expect("(")?;
                let condition = self.parse_expression()?;
                self.expect(")")?;
                
                // 解析 if 块
                self.expect("{")?;
                let mut if_block = Vec::new();
                while self.peek() != Some(&"}".to_string()) {
                    if_block.push(self.parse_statement()?);
                }
                self.expect("}")?;
                
                // 解析 else if 和 else 块
                let mut else_blocks = Vec::new();
                
                while self.peek() == Some(&"else".to_string()) {
                    self.consume(); // 消费 "else"
                    
                    if self.peek() == Some(&"if".to_string()) {
                        // else if 块
                        self.consume(); // 消费 "if"
                        
                        // 解析条件
                        self.expect("(")?;
                        let else_if_condition = self.parse_expression()?;
                        self.expect(")")?;
                        
                        // 解析 else if 块
                        self.expect("{")?;
                        let mut else_if_block = Vec::new();
                        while self.peek() != Some(&"}".to_string()) {
                            else_if_block.push(self.parse_statement()?);
                        }
                        self.expect("}")?;
                        
                        else_blocks.push((Some(else_if_condition), else_if_block));
                    } else {
                        // else 块
                        self.expect("{")?;
                        let mut else_block = Vec::new();
                        while self.peek() != Some(&"}".to_string()) {
                            else_block.push(self.parse_statement()?);
                        }
                        self.expect("}")?;
                        
                        else_blocks.push((None, else_block));
                        break; // else 块后不应该有更多块
                    }
                }
                
                self.expect(";")?;
                Ok(Statement::IfElse(condition, if_block, else_blocks))
            },
            Some(token) if token == "for" => {
                self.consume(); // 消费 "for"
                
                // 解析 for 循环结构: for (variable : range_start..range_end) { ... }
                self.expect("(")?;
                
                // 解析变量名
                let variable_name = self.consume().ok_or_else(|| "期望变量名".to_string())?;
                
                self.expect(":")?;
                
                // 解析范围起始值
                let range_start = self.parse_expression()?;
                
                self.expect("..")?;
                
                // 解析范围结束值
                let range_end = self.parse_expression()?;
                
                self.expect(")")?;
                
                // 解析循环体
                self.expect("{")?;
                let mut loop_body = Vec::new();
                while self.peek() != Some(&"}".to_string()) {
                    loop_body.push(self.parse_statement()?);
                }
                self.expect("}")?;
                self.expect(";")?;
                
                Ok(Statement::ForLoop(variable_name, range_start, range_end, loop_body))
            },
            Some(token) if token == "while" => {
                self.consume(); // 消费 "while"
                
                // 解析条件
                self.expect("(")?;
                let condition = self.parse_expression()?;
                self.expect(")")?;
                
                // 解析循环体
                self.expect("{")?;
                let mut loop_body = Vec::new();
                while self.peek() != Some(&"}".to_string()) {
                    loop_body.push(self.parse_statement()?);
                }
                self.expect("}")?;
                self.expect(";")?;
                
                Ok(Statement::WhileLoop(condition, loop_body))
            },
            // 添加对前置自增/自减的支持
            Some(token) if token == "++" => {
                self.consume(); // 消费 "++"
                
                // 获取变量名
                let var_name = self.consume().ok_or_else(|| "前置自增操作符后期望变量名".to_string())?;
                
                self.expect(";")?;
                Ok(Statement::PreIncrement(var_name))
            },
            Some(token) if token == "--" => {
                self.consume(); // 消费 "--"
                
                // 获取变量名
                let var_name = self.consume().ok_or_else(|| "前置自减操作符后期望变量名".to_string())?;
                
                self.expect(";")?;
                Ok(Statement::PreDecrement(var_name))
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
}