use crate::ast::{Interface, InterfaceMethod, Parameter, Type, Visibility};
use crate::parser::parser_base::ParserBase;
use crate::parser::statement_parser::StatementParser;
use crate::interpreter::debug_println;

pub trait InterfaceParser {
    fn parse_interface(&mut self) -> Result<Interface, String>;
    fn parse_interface_method(&mut self) -> Result<InterfaceMethod, String>;
}

impl<'a> InterfaceParser for ParserBase<'a> {
    fn parse_interface(&mut self) -> Result<Interface, String> {
        debug_println("开始解析接口");
        
        // 消费 "interface" 关键字
        self.expect("interface")?;
        
        // 获取接口名
        let interface_name = self.consume().ok_or_else(|| "期望接口名".to_string())?;
        debug_println(&format!("解析接口: {}", interface_name));
        
        // 检查是否有接口继承 (extends Interface1, Interface2)
        let mut extends = Vec::new();
        if self.peek() == Some(&"extends".to_string()) {
            self.consume(); // 消费 "extends"
            
            // 解析继承的接口列表
            loop {
                let parent_interface = self.consume().ok_or_else(|| "期望父接口名".to_string())?;
                extends.push(parent_interface);
                
                if self.peek() != Some(&",".to_string()) {
                    break;
                }
                self.consume(); // 消费 ","
            }
        }
        
        // 期望 "{"
        self.expect("{")?;
        
        let mut methods = Vec::new();
        
        // 解析接口方法
        while self.peek() != Some(&"}".to_string()) {
            let method = self.parse_interface_method()?;
            methods.push(method);
        }
        
        // 期望 "}"
        self.expect("}")?;
        self.expect(";")?;
        
        debug_println(&format!("接口解析完成: {} (继承: {:?}, 方法数: {})", interface_name, extends, methods.len()));
        
        Ok(Interface {
            name: interface_name,
            methods,
            extends,
        })
    }
    
    fn parse_interface_method(&mut self) -> Result<InterfaceMethod, String> {
        debug_println("开始解析接口方法");
        
        // 接口方法默认为public，但也可以显式指定
        let visibility = if self.peek() == Some(&"public".to_string()) {
            self.consume();
            Visibility::Public
        } else if self.peek() == Some(&"protected".to_string()) {
            self.consume();
            Visibility::Protected
        } else if self.peek() == Some(&"private".to_string()) {
            self.consume();
            Visibility::Private
        } else {
            Visibility::Public // 接口方法默认为public
        };
        
        // 期望 "fn"
        self.expect("fn")?;
        
        // 获取方法名
        let method_name = self.consume().ok_or_else(|| "期望方法名".to_string())?;
        debug_println(&format!("解析接口方法: {}", method_name));
        
        // 期望 "("
        self.expect("(")?;
        
        // 解析参数列表
        let mut parameters = Vec::new();
        
        if self.peek() != Some(&")".to_string()) {
            loop {
                // 参数名
                let param_name = self.consume().ok_or_else(|| "期望参数名".to_string())?;
                
                // 期望 ":"
                self.expect(":")?;
                
                // 参数类型
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
        
        // 期望 ")"
        self.expect(")")?;
        
        // 期望 ":"
        self.expect(":")?;
        
        // 返回类型
        let return_type = self.parse_type()?;
        
        // 接口方法只有声明，没有实现，期望 ";"
        self.expect(";")?;
        
        debug_println(&format!("接口方法解析完成: {} (参数数: {}, 返回类型: {:?})", method_name, parameters.len(), return_type));
        
        Ok(InterfaceMethod {
            name: method_name,
            parameters,
            return_type,
            visibility,
        })
    }
}