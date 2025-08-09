// 导入必要的模块
use crate::ast::{Statement, Expression, Type, Parameter, Function, BinaryOperator, NamespaceType, SwitchCase, CasePattern, SwitchType, MatchArm};
use crate::parser::parser_base::ParserBase;
use crate::parser::expression_parser::ExpressionParser;
use crate::parser::enum_parser::EnumParser;
use crate::parser::pointer_parser::PointerParser;
use crate::parser::pattern_parser::PatternParser;
use crate::interpreter::debug_println;

pub trait StatementParser {
    fn parse_statement(&mut self) -> Result<Statement, String>;
    fn parse_statement_block(&mut self) -> Result<Vec<Statement>, String>;
    fn parse_variable_declaration(&mut self) -> Result<Statement, String>;
    fn parse_if_statement(&mut self) -> Result<Statement, String>;
    fn parse_for_loop(&mut self) -> Result<Statement, String>;
    fn parse_foreach_loop(&mut self) -> Result<Statement, String>;
    fn parse_while_loop(&mut self) -> Result<Statement, String>;
    fn parse_try_catch(&mut self) -> Result<Statement, String>;
    fn parse_throw_statement(&mut self) -> Result<Statement, String>;
    fn parse_switch_statement(&mut self) -> Result<Statement, String>;
    fn parse_case_pattern(&mut self) -> Result<CasePattern, String>;
    fn parse_type(&mut self) -> Result<Type, String>;
}

impl<'a> StatementParser for ParserBase<'a> {
    fn parse_statement(&mut self) -> Result<Statement, String> {
        if let Some(token) = self.peek() {
            // 支持 using ns xxx; 语句
            if token == "using" {
                self.consume(); // 消费 using
                if self.peek() == Some(&"ns".to_string()) {
                    self.consume(); // 消费 ns
                    // 解析命名空间路径
                    let mut path = Vec::new();
                    
                    // 解析第一个标识符
                    if let Some(first_id) = self.peek() {
                        if first_id.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            path.push(self.consume().unwrap());
                        } else {
                            return Err("期望命名空间标识符".to_string());
                        }
                    } else {
                        return Err("期望命名空间标识符".to_string());
                    }
                    
                    // 解析后续的 :: 和标识符
                    while self.peek() == Some(&"::".to_string()) {
                        self.consume(); // 消费 "::"
                        if let Some(next_id) = self.peek() {
                            if next_id.chars().all(|c| c.is_alphanumeric() || c == '_') {
                                path.push(self.consume().unwrap());
                            } else {
                                return Err("期望命名空间标识符".to_string());
                            }
                        } else {
                            return Err("期望命名空间标识符".to_string());
                        }
                    }
                    
                    self.expect(";")?;
                    return Ok(Statement::ImportNamespace(crate::ast::NamespaceType::Code, path));
                } else {
                    return Err("不支持的using语句，仅支持using ns".to_string());
                }
            }
            match token.as_str() {
                "return" => {
                    self.consume(); // 消费 "return" 关键字
                    
                    // 检查是否有返回值
                    if self.peek() == Some(&";".to_string()) {
                        self.consume(); // 消费分号
                        // 返回void
                        Ok(Statement::Return(None))
                    } else {
                        // 解析返回表达式
                        let expr = self.parse_expression()?;
                        self.expect(";")?;
                        Ok(Statement::Return(Some(expr)))
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
                "try" => {
                    self.parse_try_catch()
                },
                "throw" => {
                    self.parse_throw_statement()
                },
                "match" => {
                    self.parse_match_statement()
                },
                "switch" => {
                    self.parse_switch_statement()
                },
                "enum" => {
                    // 枚举声明
                    let enum_def = self.parse_enum()?;
                    Ok(Statement::EnumDeclaration(enum_def))
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

                        // 使用parse_type方法解析类型（支持指针类型）
                        let var_type = self.parse_type()?;

                        // 检查是否有初始值
                        if self.peek() == Some(&"=".to_string()) {
                            // 有初始值
                            self.consume(); // 消费 "="
                            let init_expr = self.parse_expression()?;
                            self.expect(";")?;
                            Ok(Statement::VariableDeclaration(var_name, var_type, init_expr))
                        } else {
                            // 没有初始值，只是类型声明
                            self.expect(";")?;
                            let default_expr = Expression::None;
                            Ok(Statement::VariableDeclaration(var_name, var_type, default_expr))
                        }
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
                        // 静态访问或命名空间函数调用
                        self.consume(); // 消费 "::"
                        
                        // 获取成员名或函数名
                        let member_name = self.consume().ok_or_else(|| "期望成员名或函数名".to_string())?;
                        
                        // 检查下一个token来决定是静态赋值还是函数调用
                        if self.peek() == Some(&"=".to_string()) {
                            // 静态字段赋值: ClassName::field = value
                            self.consume(); // 消费 "="
                            let value_expr = self.parse_expression()?;
                            self.expect(";")?;
                            
                            // 创建静态字段赋值语句
                            let static_access = Expression::StaticAccess(var_name, member_name);
                            Ok(Statement::FieldAssignment(
                                Box::new(static_access),
                                "".to_string(), // 静态访问不需要字段名
                                value_expr
                            ))
                        } else if self.peek() == Some(&"(".to_string()) || self.peek() == Some(&"::".to_string()) {
                            // 🔧 修复：这是函数调用或多级命名空间调用
                            // 检查是否是库函数调用
                            if var_name.starts_with("lib_") && self.peek() == Some(&"(".to_string()) {
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

                                Ok(Statement::LibraryFunctionCallStatement(lib_name, member_name, args))
                            } else {
                                // 静态方法调用或命名空间函数调用
                                let mut path = Vec::new();
                                path.push(var_name.clone()); // 第一个命名空间名
                                path.push(member_name.clone()); // 函数名或下一级命名空间

                                // 🔧 修复：解析多级命名空间路径（无论下一个是::还是(）
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

                                // 检查是否是静态方法调用（只有两个部分：ClassName::methodName）
                                if path.len() == 2 {
                                    // 创建静态方法调用表达式
                                    let static_call = Expression::StaticMethodCall(path[0].clone(), path[1].clone(), args);
                                    Ok(Statement::FunctionCallStatement(static_call))
                                } else {
                                    Ok(Statement::NamespacedFunctionCallStatement(path, args))
                                }
                            }
                        } else {
                            return Err(format!("期望 '=' 或 '(' 在 '{}::{}' 之后", var_name, member_name));
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
                    } else if next_token == "." {
                        // 处理对象方法调用或字段访问
                        self.consume(); // 消费 "."
                        let member_name = self.consume().ok_or_else(|| "期望成员名".to_string())?;

                        if self.peek() == Some(&"(".to_string()) {
                            // 方法调用: obj.method(args)
                            self.consume(); // 消费 "("

                            let mut args = Vec::new();
                            if self.peek() != Some(&")".to_string()) {
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

                            // 创建方法调用表达式
                            let obj_expr = if var_name == "this" {
                                Expression::This
                            } else {
                                Expression::Variable(var_name)
                            };
                            let method_call_expr = Expression::MethodCall(
                                Box::new(obj_expr),
                                member_name,
                                args
                            );

                            Ok(Statement::FunctionCallStatement(method_call_expr))
                        } else if self.peek() == Some(&"=".to_string()) {
                            // 字段赋值: obj.field = value
                            self.expect("=")?;
                            let value_expr = self.parse_expression()?;
                            self.expect(";")?;

                            let obj_expr = if var_name == "this" {
                                Expression::This
                            } else {
                                Expression::Variable(var_name)
                            };

                            Ok(Statement::FieldAssignment(
                                Box::new(obj_expr),
                                member_name,
                                value_expr
                            ))
                        } else {
                            Err(format!("期望 '(' 或 '=' 在 '{}.{}' 之后", var_name, member_name))
                        }
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

        // 检查是否有初始值
        if self.peek() == Some(&"=".to_string()) {
            // 有初始值
            self.consume(); // 消费 "="

            // 解析初始值表达式
            let init_expr = self.parse_expression()?;

            // 期望分号
            self.expect(";")?;

            Ok(Statement::VariableDeclaration(var_name, var_type, init_expr))
        } else {
            // 没有初始值，只是类型声明
            self.expect(";")?;

            // 创建一个默认的None表达式作为占位符
            let default_expr = Expression::None;

            Ok(Statement::VariableDeclaration(var_name, var_type, default_expr))
        }
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
        // 首先检查是否是指针类型或数组类型
        if let Some(token) = self.peek() {
            if token == "?" && self.peek_ahead(1) == Some(&"*".to_string()) {
                // 可选指针类型 ?*Type
                return self.parse_pointer_type();
            } else if token == "*" {
                // 普通指针类型 *Type
                return self.parse_pointer_type();
            } else if token == "[" {
                // 数组类型或函数指针数组类型: []int 或 []*fn(int, int) : int
                self.consume(); // 消费 "["
                self.expect("]")?; // 期望 "]"

                if self.peek() == Some(&"*".to_string()) {
                    self.consume(); // 消费 "*"

                    if self.peek() == Some(&"fn".to_string()) {
                        // 函数指针数组类型: []*fn(int, int) : int
                        self.consume(); // 消费 "fn"
                        self.expect("(")?;

                        let mut param_types = Vec::new();
                        if self.peek() != Some(&")".to_string()) {
                            loop {
                                param_types.push(self.parse_type()?);
                                if self.peek() != Some(&",".to_string()) {
                                    break;
                                }
                                self.consume(); // 消费 ","
                            }
                        }

                        self.expect(")")?;
                        self.expect(":")?;
                        let return_type = Box::new(self.parse_type()?);

                        let func_ptr_type = Type::FunctionPointer(param_types, return_type);
                        return Ok(Type::Array(Box::new(func_ptr_type)));
                    } else {
                        // 指针数组类型: []*int
                        let target_type = Box::new(self.parse_type()?);
                        let ptr_type = Type::Pointer(target_type);
                        return Ok(Type::Array(Box::new(ptr_type)));
                    }
                } else {
                    // 普通数组类型: []int
                    let element_type = Box::new(self.parse_type()?);
                    return Ok(Type::Array(element_type));
                }
            }
        }

        let type_name = self.consume().ok_or_else(|| "期望类型名".to_string())?;

        match type_name.as_str() {
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "long" => Ok(Type::Long),
            "void" => Ok(Type::Void),
            "Auto" => Ok(Type::Auto), // 添加Auto类型支持
            "Exception" => Ok(Type::Exception),
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
            _ => Ok(Type::Class(type_name)), // 假设是类类型
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

    fn parse_try_catch(&mut self) -> Result<Statement, String> {
        self.consume(); // 消费 "try"
        
        // 解析 try 块
        let try_block = self.parse_statement_block()?;
        
        // 解析 catch 块
        let mut catch_blocks = Vec::new();
        
        while self.peek() == Some(&"catch".to_string()) {
            self.consume(); // 消费 "catch"
            
            // 解析异常参数
            self.expect("(")?;
            let exception_name = self.consume().ok_or_else(|| "期望异常变量名".to_string())?;
            self.expect(":")?;
            let exception_type = self.parse_type()?;
            self.expect(")")?;
            
            // 解析 catch 块
            let catch_block = self.parse_statement_block()?;
            
            catch_blocks.push((exception_name, exception_type, catch_block));
        }
        
        // 解析 finally 块（可选）
        let finally_block = if self.peek() == Some(&"finally".to_string()) {
            self.consume(); // 消费 "finally"
            Some(self.parse_statement_block()?)
        } else {
            None
        };
        
        self.expect(";")?;
        
        Ok(Statement::TryCatch(try_block, catch_blocks, finally_block))
    }

    fn parse_throw_statement(&mut self) -> Result<Statement, String> {
        self.consume(); // 消费 "throw"
        
        // 解析要抛出的异常表达式
        let exception_expr = self.parse_expression()?;
        
        self.expect(";")?;
        
        Ok(Statement::Throw(exception_expr))
    }

    fn parse_switch_statement(&mut self) -> Result<Statement, String> {
        self.consume(); // 消费 "switch"
        
        // 解析 switch 表达式
        self.expect("(")?;
        let switch_expr = self.parse_expression()?;
        self.expect(")")?;
        
        // 解析 switch 块
        self.expect("{")?;
        
        let mut cases = Vec::new();
        let mut default_block = None;
        
        while self.peek() != Some(&"}".to_string()) {
            if self.peek() == Some(&"case".to_string()) {
                self.consume(); // 消费 "case"
                
                // 解析 case 模式
                let case_pattern = self.parse_case_pattern()?;
                
                // 检查是否是表达式形式 (使用 => )
                if self.peek() == Some(&"=>".to_string()) {
                    self.consume(); // 消费 "=>"
                    
                    // 解析表达式
                    let case_expr = self.parse_expression()?;
                    self.expect(",")?; // 表达式形式用逗号分隔
                    
                    cases.push(SwitchCase {
                        pattern: case_pattern,
                        statements: Vec::new(),
                        expression: Some(case_expr),
                        has_break: true, // 表达式形式自动break
                    });
                } else {
                    // 语句形式
                    self.expect("{")?;
                    let mut case_statements = Vec::new();
                    let mut has_break = false;
                    
                    while self.peek() != Some(&"}".to_string()) {
                        let stmt = self.parse_statement()?;
                        
                        // 检查是否是 break 语句
                        if matches!(stmt, Statement::Break) {
                            has_break = true;
                            case_statements.push(stmt);
                            break; // break 后不再解析更多语句
                        } else {
                            case_statements.push(stmt);
                        }
                    }
                    
                    self.expect("}")?;
                    self.expect(";")?;
                    
                    cases.push(SwitchCase {
                        pattern: case_pattern,
                        statements: case_statements,
                        expression: None,
                        has_break,
                    });
                }
            } else if self.peek() == Some(&"default".to_string()) {
                self.consume(); // 消费 "default"
                
                // 检查是否是表达式形式
                if self.peek() == Some(&"=>".to_string()) {
                    self.consume(); // 消费 "=>"
                    let default_expr = self.parse_expression()?;
                    // 对于表达式形式的default，我们将其转换为语句块
                    default_block = Some(vec![Statement::Return(Some(default_expr))]);
                } else {
                    // 语句形式
                    self.expect("{")?;
                    let mut default_statements = Vec::new();
                    
                    while self.peek() != Some(&"}".to_string()) {
                        default_statements.push(self.parse_statement()?);
                    }
                    
                    self.expect("}")?;
                    self.expect(";")?;
                    
                    default_block = Some(default_statements);
                }
            } else {
                return Err(format!("期望 'case' 或 'default'，但找到: {:?}", self.peek()));
            }
        }
        
        self.expect("}")?;
        self.expect(";")?;
        
        Ok(Statement::Switch(switch_expr, cases, default_block, SwitchType::Statement))
    }

    fn parse_case_pattern(&mut self) -> Result<CasePattern, String> {
        // 先尝试解析第一个表达式
        let first_expr = self.parse_expression()?;
        
        // 检查是否是范围匹配
        if self.peek() == Some(&"..".to_string()) {
            self.consume(); // 消费 ".."
            let end_expr = self.parse_expression()?;
            return Ok(CasePattern::Range(first_expr, end_expr));
        }
        
        // 检查是否是Guard条件
        if self.peek() == Some(&"if".to_string()) {
            // 第一个表达式应该是变量
            if let Expression::Variable(var_name) = first_expr {
                self.consume(); // 消费 "if"
                let guard_condition = self.parse_expression()?;
                return Ok(CasePattern::Guard(var_name, guard_condition));
            } else {
                return Err("Guard模式中期望变量名".to_string());
            }
        }
        
        // 默认是值匹配
        Ok(CasePattern::Value(first_expr))
    }

    /// 解析match语句
    fn parse_match_statement(&mut self) -> Result<Statement, String> {
        // 使用PatternParser trait的方法
        let (match_expr, arms) = PatternParser::parse_match_statement(self)?;

        Ok(Statement::Match(match_expr, arms))
    }
}