// 导入必要的模块
use crate::ast::{Statement, Expression, Type, Parameter, Function, BinaryOperator, NamespaceType};
use crate::parser::parser_base::ParserBase;
use crate::parser::expression_parser::ExpressionParser;
use crate::interpreter::debug_println;

pub trait StatementParser {
    fn parse_statement(&mut self) -> Result<Statement, String>;
    fn parse_statement_block(&mut self) -> Result<Vec<Statement>, String>;
    fn parse_variable_declaration(&mut self) -> Result<Statement, String>;
    fn parse_if_statement(&mut self) -> Result<Statement, String>;
    fn parse_for_loop(&mut self) -> Result<Statement, String>;
    fn parse_foreach_loop(&mut self) -> Result<Statement, String>;
    fn parse_while_loop(&mut self) -> Result<Statement, String>;
    fn parse_type(&mut self) -> Result<Type, String>;
}

impl<'a> StatementParser for ParserBase<'a> {
    fn parse_statement(&mut self) -> Result<Statement, String> {
        if let Some(token) = self.peek() {
            match token.as_str() {
                "return" => {
                    self.consume(); // 消费 "return" 关键字
                    
                    // 检查是否有返回值
                    if self.peek() == Some(&";".to_string()) {
                        self.consume(); // 消费分号
                        // 返回void
                        Ok(Statement::Return(Expression::BoolLiteral(false))) // 使用布尔字面量作为占位符
                    } else {
                        // 解析返回表达式
                        let expr = self.parse_expression()?;
                        self.expect(";")?;
                        Ok(Statement::Return(expr))
                    }
                },
                "if" => {
                    self.parse_if_statement()
                },
                "for" => {
                    self.parse_for_loop()
                },
                "foreach" => {
                    self.parse_foreach_loop()
                },
                "while" => {
                    self.parse_while_loop()
                },
                "break" => {
                self.consume(); // 消费 "break"
                self.expect(";")?;
                Ok(Statement::Break)
            },
                "continue" => {
                self.consume(); // 消费 "continue"
                self.expect(";")?;
                Ok(Statement::Continue)
            },
            // 添加对前置自增/自减的支持
                "++" => {
                self.consume(); // 消费 "++"
                
                // 获取变量名
                let var_name = self.consume().ok_or_else(|| "前置自增操作符后期望变量名".to_string())?;
                
                self.expect(";")?;
                Ok(Statement::PreIncrement(var_name))
            },
                "--" => {
                self.consume(); // 消费 "--"
                
                // 获取变量名
                let var_name = self.consume().ok_or_else(|| "前置自减操作符后期望变量名".to_string())?;
                
                self.expect(";")?;
                Ok(Statement::PreDecrement(var_name))
            },
                "const" => {
                    // 解析常量声明
                    self.consume(); // 消费 "const"
                    
                    // 获取常量名
                    let const_name = self.consume().ok_or_else(|| "期望常量名".to_string())?;
                    
                    self.expect(":")?;
                    
                    // 解析类型
                    let type_name = self.consume().ok_or_else(|| "期望类型名".to_string())?;
                    
                    // 转换为内部类型
                    let const_type = match type_name.as_str() {
                        "int" => crate::ast::Type::Int,
                        "float" => crate::ast::Type::Float,
                        "bool" => crate::ast::Type::Bool,
                        "string" => crate::ast::Type::String,
                        "long" => crate::ast::Type::Long,
                        _ => return Err(format!("不支持的常量类型: {}", type_name))
                    };
                    
                    self.expect("=")?;
                    
                    // 解析初始值表达式
                    let init_expr = self.parse_expression()?;
                    
                    self.expect(";")?;
                    
                    Ok(Statement::ConstantDeclaration(const_name, const_type, init_expr))
                },
                _ => {
                // 检查是否是变量声明、赋值或函数调用
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
                    } else if next_token == "::" {
                        // 命名空间函数调用
                        self.consume(); // 消费 "::"
                        
                        // 获取函数名
                        let func_name = self.consume().ok_or_else(|| "期望函数名".to_string())?;
                        
                        // 检查是否是库函数调用
                        if var_name.starts_with("lib_") {
                            // 库函数调用，格式为 lib_xxx::func_name
                            let lib_name = var_name.trim_start_matches("lib_").to_string();
                            
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
                            self.expect(";")?;
                            
                            Ok(Statement::LibraryFunctionCallStatement(lib_name, func_name, args))
                        } else {
                            // 命名空间函数调用
                            let mut path = Vec::new();
                            path.push(var_name); // 第一个命名空间名
                            path.push(func_name); // 函数名或下一级命名空间
                            
                            // 解析命名空间路径
                            while self.peek() == Some(&"::".to_string()) {
                                self.consume(); // 消费 "::"
                                if let Some(name) = self.consume() {
                                    path.push(name);
                                } else {
                                    return Err("期望标识符".to_string());
                                }
                            }
                            
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
                            self.expect(";")?;
                            
                            Ok(Statement::NamespacedFunctionCallStatement(path, args))
                        }
                    } else if next_token == "(" {
                        // 函数调用语句
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
                        self.expect(";")?;
                        
                        // 创建函数调用表达式
                        let func_call_expr = Expression::FunctionCall(var_name, args);
                        
                        // 返回函数调用语句
                        Ok(Statement::FunctionCallStatement(func_call_expr))
                    } else {
                        Err(format!("不支持的语句: {} {}", var_name, next_token))
                    }
                } else {
                    Err("不完整的语句".to_string())
                }
                }
            }
        } else {
            Err("期望语句".to_string())
        }
    }
    
    fn parse_statement_block(&mut self) -> Result<Vec<Statement>, String> {
        self.expect("{")?;
        let mut statements = Vec::new();
        while self.peek() != Some(&"}".to_string()) {
            statements.push(self.parse_statement()?);
        }
        self.expect("}")?;
        Ok(statements)
    }
    
    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        // 获取变量名
        let var_name = self.consume().ok_or_else(|| "期望变量名".to_string())?;
        
        // 期望类型声明
        self.expect(":")?;
        
        // 解析类型
        let var_type = self.parse_type()?;
        
        // 期望赋值符号
        self.expect("=")?;
        
        // 解析初始值表达式
        let init_expr = self.parse_expression()?;
        
        // 期望分号
        self.expect(";")?;
        
        Ok(Statement::VariableDeclaration(var_name, var_type, init_expr))
    }
    
    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.consume(); // 消费 "if"
        
        // 解析条件
        self.expect("(")?;
        let condition = self.parse_expression()?;
        self.expect(")")?;
        
        // 解析 if 块
        let if_block = self.parse_statement_block()?;
        
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
                let else_if_block = self.parse_statement_block()?;
                
                else_blocks.push((Some(else_if_condition), else_if_block));
            } else {
                // else 块
                let else_block = self.parse_statement_block()?;
                
                else_blocks.push((None, else_block));
                break; // else 块后不应该有更多块
            }
        }
        
        self.expect(";")?;
        Ok(Statement::IfElse(condition, if_block, else_blocks))
    }
    
    fn parse_for_loop(&mut self) -> Result<Statement, String> {
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
        let loop_body = self.parse_statement_block()?;
        self.expect(";")?;
        
        Ok(Statement::ForLoop(variable_name, range_start, range_end, loop_body))
    }
    
    fn parse_while_loop(&mut self) -> Result<Statement, String> {
        self.consume(); // 消费 "while"
        
        // 解析条件
        self.expect("(")?;
        let condition = self.parse_expression()?;
        self.expect(")")?;
        
        // 解析循环体
        let loop_body = self.parse_statement_block()?;
        self.expect(";")?;
        
        Ok(Statement::WhileLoop(condition, loop_body))
    }
    
    fn parse_type(&mut self) -> Result<Type, String> {
        let type_name = self.consume().ok_or_else(|| "期望类型名".to_string())?;
        
        match type_name.as_str() {
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "long" => Ok(Type::Long),
            "void" => Ok(Type::Void),
            "array" => {
                // 解析数组元素类型
                self.expect("<")?;
                let element_type = self.parse_type()?;
                self.expect(">")?;
                Ok(Type::Array(Box::new(element_type)))
            },
            "map" => {
                // 解析映射的键和值类型
                self.expect("<")?;
                let key_type = self.parse_type()?;
                self.expect(",")?;
                let value_type = self.parse_type()?;
                self.expect(">")?;
                Ok(Type::Map(Box::new(key_type), Box::new(value_type)))
            },
            _ => Err(format!("未知类型: {}", type_name)),
        }
    }

    fn parse_foreach_loop(&mut self) -> Result<Statement, String> {
        self.consume(); // 消费 "foreach"
        
        // 解析 foreach 循环结构: foreach (item in collection) { ... }
        self.expect("(")?;
        
        // 解析迭代变量名
        let variable_name = self.consume().ok_or_else(|| "期望迭代变量名".to_string())?;
        
        // 期望 "in" 关键字
        if self.peek() != Some(&"in".to_string()) {
            return Err("期望 'in' 关键字".to_string());
        }
        self.consume(); // 消费 "in"
        
        // 解析集合表达式
        let collection_expr = self.parse_expression()?;
        
        self.expect(")")?;
        
        // 解析循环体
        self.expect("{")?;
        let mut loop_body = Vec::new();
        while self.peek() != Some(&"}".to_string()) {
            loop_body.push(self.parse_statement()?);
        }
        self.expect("}")?;
        self.expect(";")?;
        
        Ok(Statement::ForEachLoop(variable_name, collection_expr, loop_body))
    }
}