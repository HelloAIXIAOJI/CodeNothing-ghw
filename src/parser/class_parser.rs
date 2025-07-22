// 类解析模块
use crate::ast::{Class, Field, Method, Constructor, Parameter, Type, Visibility};
use crate::parser::parser_base::ParserBase;
use crate::parser::statement_parser::StatementParser;
use crate::parser::expression_parser::ExpressionParser;

pub trait ClassParser {
    fn parse_class(&mut self) -> Result<Class, String>;
    fn parse_visibility(&mut self) -> Visibility;
    fn parse_field(&mut self) -> Result<Field, String>;
    fn parse_method(&mut self) -> Result<Method, String>;
    fn parse_constructor(&mut self) -> Result<Constructor, String>;
}

impl<'a> ClassParser for ParserBase<'a> {
    fn parse_class(&mut self) -> Result<Class, String> {
        // 消费 "class" 关键字
        self.consume(); // class
        
        // 获取类名
        let class_name = self.consume().ok_or_else(|| "期望类名".to_string())?;
        
        // 解析类体
        self.expect("{")?;
        
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut constructors = Vec::new();
        
        while self.peek() != Some(&"}".to_string()) {
            // 解析访问修饰符
            let visibility = self.parse_visibility();
            
            if self.peek() == Some(&"constructor".to_string()) {
                // 解析构造函数
                let constructor = self.parse_constructor()?;
                constructors.push(constructor);
            } else if self.peek() == Some(&"fn".to_string()) {
                // 解析方法
                let mut method = self.parse_method()?;
                method.visibility = visibility;
                methods.push(method);
            } else if self.peek().is_some() {
                // 解析字段
                let mut field = self.parse_field()?;
                field.visibility = visibility;
                fields.push(field);
            } else {
                return Err("期望字段、方法或构造函数".to_string());
            }
        }
        
        self.expect("}")?;
        self.expect(";")?;
        
        Ok(Class {
            name: class_name,
            fields,
            methods,
            constructors,
        })
    }
    
    fn parse_visibility(&mut self) -> Visibility {
        match self.peek() {
            Some(token) if token == "private" => {
                self.consume();
                Visibility::Private
            },
            Some(token) if token == "protected" => {
                self.consume();
                Visibility::Protected
            },
            Some(token) if token == "public" => {
                self.consume();
                Visibility::Public
            },
            _ => Visibility::Public, // 默认为public
        }
    }
    
    fn parse_field(&mut self) -> Result<Field, String> {
        // 字段名
        let field_name = self.consume().ok_or_else(|| "期望字段名".to_string())?;
        
        self.expect(":")?;
        
        // 字段类型
        let field_type = self.parse_type()?;
        
        // 可选的初始值
        let initial_value = if self.peek() == Some(&"=".to_string()) {
            self.consume(); // 消费 "="
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(";")?;
        
        Ok(Field {
            name: field_name,
            field_type,
            visibility: Visibility::Public, // 将在调用处设置
            initial_value,
        })
    }
    
    fn parse_method(&mut self) -> Result<Method, String> {
        self.consume(); // 消费 "fn"
        
        // 方法名
        let method_name = self.consume().ok_or_else(|| "期望方法名".to_string())?;
        
        // 解析参数列表
        self.expect("(")?;
        let mut parameters = Vec::new();
        
        if self.peek() != Some(&")".to_string()) {
            loop {
                let param_name = self.consume().ok_or_else(|| "期望参数名".to_string())?;
                self.expect(":")?;
                let param_type = self.parse_type()?;
                
                parameters.push(Parameter {
                    name: param_name,
                    param_type,
                });
                
                if self.peek() != Some(&",".to_string()) {
                    break;
                }
                self.consume(); // 消费 ","
            }
        }
        
        self.expect(")")?;
        self.expect(":")?;
        
        // 返回类型
        let return_type = self.parse_type()?;
        
        // 方法体
        self.expect("{")?;
        let mut body = Vec::new();
        
        while self.peek() != Some(&"}".to_string()) {
            let stmt = self.parse_statement()?;
            body.push(stmt);
        }
        
        self.expect("}")?;
        self.expect(";")?;
        
        Ok(Method {
            name: method_name,
            parameters,
            return_type,
            body,
            visibility: Visibility::Public, // 将在调用处设置
        })
    }
    
    fn parse_constructor(&mut self) -> Result<Constructor, String> {
        self.consume(); // 消费 "constructor"
        
        // 解析参数列表
        self.expect("(")?;
        let mut parameters = Vec::new();
        
        if self.peek() != Some(&")".to_string()) {
            loop {
                let param_name = self.consume().ok_or_else(|| "期望参数名".to_string())?;
                self.expect(":")?;
                let param_type = self.parse_type()?;
                
                parameters.push(Parameter {
                    name: param_name,
                    param_type,
                });
                
                if self.peek() != Some(&",".to_string()) {
                    break;
                }
                self.consume(); // 消费 ","
            }
        }
        
        self.expect(")")?;
        
        // 构造函数体
        self.expect("{")?;
        let mut body = Vec::new();
        
        while self.peek() != Some(&"}".to_string()) {
            let stmt = self.parse_statement()?;
            body.push(stmt);
        }
        
        self.expect("}")?;
        self.expect(";")?;
        
        Ok(Constructor {
            parameters,
            body,
        })
    }
}