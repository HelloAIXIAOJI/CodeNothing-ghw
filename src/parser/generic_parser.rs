// 泛型解析器 - 处理泛型语法的解析
use crate::ast::*;
use crate::parser::parser_base::ParserBase;
use crate::parser::statement_parser::StatementParser;
use crate::parser::expression_parser::ExpressionParser;

impl<'a> ParserBase<'a> {
    /// 解析泛型参数列表 <T, U, K>
    pub fn parse_generic_parameters(&mut self) -> Result<Vec<GenericParameter>, String> {
        let mut generic_params = Vec::new();

        // 检查是否有 '<' 开始泛型参数
        if self.peek() != Some(&"<".to_string()) {
            return Ok(generic_params); // 没有泛型参数
        }
        self.consume(); // 消费 '<'

        // 解析第一个泛型参数
        if self.peek() != Some(&">".to_string()) {
            loop {
                let param = self.parse_generic_parameter()?;
                generic_params.push(param);

                if self.peek() != Some(&",".to_string()) {
                    break;
                }
                self.consume(); // 消费 ','
            }
        }

        // 期望 '>' 结束泛型参数列表
        if self.peek() != Some(&">".to_string()) {
            return Err("期望 '>' 结束泛型参数列表".to_string());
        }
        self.consume(); // 消费 '>'

        Ok(generic_params)
    }
    
    /// 解析单个泛型参数 T: Constraint + Constraint = DefaultType
    pub fn parse_generic_parameter(&mut self) -> Result<GenericParameter, String> {
        // 解析类型参数名
        let name = self.consume().ok_or_else(|| "期望泛型参数名".to_string())?;

        // 解析约束 (可选)
        let mut constraints = Vec::new();
        if self.peek() == Some(&":".to_string()) {
            self.consume(); // 消费 ':'
            constraints = self.parse_type_constraints()?;
        }

        // 解析默认类型 (可选)
        let default_type = if self.peek() == Some(&"=".to_string()) {
            self.consume(); // 消费 '='
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(GenericParameter {
            name,
            constraints,
            default_type,
        })
    }
    
    /// 解析类型约束列表 Constraint + Constraint + ...
    pub fn parse_type_constraints(&mut self) -> Result<Vec<TypeConstraint>, String> {
        let mut constraints = Vec::new();

        loop {
            let constraint = self.parse_type_constraint()?;
            constraints.push(constraint);

            if self.peek() != Some(&"+".to_string()) {
                break;
            }
            self.consume(); // 消费 '+'
        }

        Ok(constraints)
    }

    /// 解析单个类型约束
    pub fn parse_type_constraint(&mut self) -> Result<TypeConstraint, String> {
        let name = self.consume().ok_or_else(|| "期望类型约束名称".to_string())?;

        // 检查内置约束
        match name.as_str() {
            "Sized" => Ok(TypeConstraint::Sized),
            "Copy" => Ok(TypeConstraint::Copy),
            "Send" => Ok(TypeConstraint::Send),
            "Sync" => Ok(TypeConstraint::Sync),
            _ => Ok(TypeConstraint::Trait(name)),
        }
    }
    
    /// 解析 where 子句
    pub fn parse_where_clause(&mut self) -> Result<Vec<TypeConstraint>, String> {
        if self.peek() != Some(&"where".to_string()) {
            return Ok(Vec::new());
        }
        self.consume(); // 消费 'where'

        let mut constraints = Vec::new();

        loop {
            // 解析类型参数名
            let _type_param = self.consume().ok_or_else(|| "期望类型参数名在 where 子句中".to_string())?;

            // 期望 ':'
            if self.peek() != Some(&":".to_string()) {
                return Err("期望 ':' 在 where 子句中".to_string());
            }
            self.consume(); // 消费 ':'

            // 解析约束
            let type_constraints = self.parse_type_constraints()?;
            constraints.extend(type_constraints);

            if self.peek() != Some(&",".to_string()) {
                break;
            }
            self.consume(); // 消费 ','
        }

        Ok(constraints)
    }
    
    /// 解析泛型类型实例化 <Type, Type, ...>
    pub fn parse_generic_type_arguments(&mut self) -> Result<Vec<Type>, String> {
        let mut type_args = Vec::new();

        if self.peek() != Some(&"<".to_string()) {
            return Ok(type_args); // 没有类型参数
        }
        self.consume(); // 消费 '<'

        // 解析类型参数列表
        if self.peek() != Some(&">".to_string()) {
            loop {
                let type_arg = self.parse_type()?;
                type_args.push(type_arg);

                if self.peek() != Some(&",".to_string()) {
                    break;
                }
                self.consume(); // 消费 ','
            }
        }

        // 期望 '>' 结束类型参数列表
        if self.peek() != Some(&">".to_string()) {
            return Err("期望 '>' 结束类型参数列表".to_string());
        }
        self.consume(); // 消费 '>'

        Ok(type_args)
    }
    
    /// 解析泛型函数调用
    pub fn parse_generic_function_call(&mut self, function_name: String) -> Result<Expression, String> {
        // 解析类型参数
        let type_args = self.parse_generic_type_arguments()?;

        // 解析函数参数
        if self.peek() != Some(&"(".to_string()) {
            return Err("期望 '(' 开始函数参数".to_string());
        }
        self.consume(); // 消费 '('

        let mut args = Vec::new();
        if self.peek() != Some(&")".to_string()) {
            loop {
                args.push(self.parse_expression()?);
                if self.peek() != Some(&",".to_string()) {
                    break;
                }
                self.consume(); // 消费 ','
            }
        }

        if self.peek() != Some(&")".to_string()) {
            return Err("期望 ')' 结束函数参数".to_string());
        }
        self.consume(); // 消费 ')'

        Ok(Expression::GenericFunctionCall(function_name, type_args, args))
    }
    
    /// 解析泛型对象创建
    pub fn parse_generic_object_creation(&mut self, class_name: String) -> Result<Expression, String> {
        // 解析类型参数
        let type_args = self.parse_generic_type_arguments()?;

        // 解析构造函数参数
        if self.peek() != Some(&"(".to_string()) {
            return Err("期望 '(' 开始构造函数参数".to_string());
        }
        self.consume(); // 消费 '('

        let mut args = Vec::new();
        if self.peek() != Some(&")".to_string()) {
            loop {
                args.push(self.parse_expression()?);
                if self.peek() != Some(&",".to_string()) {
                    break;
                }
                self.consume(); // 消费 ','
            }
        }

        if self.peek() != Some(&")".to_string()) {
            return Err("期望 ')' 结束构造函数参数".to_string());
        }
        self.consume(); // 消费 ')'

        Ok(Expression::GenericObjectCreation(class_name, type_args, args))
    }
    
    /// 检查是否为泛型类型
    pub fn is_generic_type(&self, type_name: &str) -> bool {
        // 简单检查：如果类型名是单个大写字母，可能是泛型参数
        type_name.len() == 1 && type_name.chars().next().unwrap().is_uppercase()
    }
    
    /// 解析类型转换表达式
    pub fn parse_type_cast(&mut self, expr: Expression) -> Result<Expression, String> {
        // 期望 'as' 关键字
        if self.peek() != Some(&"as".to_string()) {
            return Err("期望 'as' 关键字进行类型转换".to_string());
        }
        self.consume(); // 消费 'as'

        let target_type = self.parse_type()?;
        Ok(Expression::TypeCast(Box::new(expr), target_type))
    }

    /// 解析 typeof 表达式
    pub fn parse_typeof_expression(&mut self) -> Result<Expression, String> {
        // 期望 'typeof' 关键字
        if self.peek() != Some(&"typeof".to_string()) {
            return Err("期望 'typeof' 关键字".to_string());
        }
        self.consume(); // 消费 'typeof'

        if self.peek() != Some(&"(".to_string()) {
            return Err("期望 '(' 在 typeof 表达式中".to_string());
        }
        self.consume(); // 消费 '('

        let expr = self.parse_expression()?;

        if self.peek() != Some(&")".to_string()) {
            return Err("期望 ')' 结束 typeof 表达式".to_string());
        }
        self.consume(); // 消费 ')'

        Ok(Expression::TypeOf(Box::new(expr)))
    }
}
