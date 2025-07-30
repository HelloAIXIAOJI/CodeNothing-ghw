use crate::ast::{Type, Expression, PointerArithmeticOp, PointerMemberAccessOp};
use crate::parser::parser_base::ParserBase;
use crate::parser::statement_parser::StatementParser;
use crate::parser::expression_parser::ExpressionParser;
use crate::parser::enum_parser::EnumParser;
use crate::interpreter::debug_println;

pub trait PointerParser {
    fn parse_pointer_type(&mut self) -> Result<Type, String>;
    fn parse_address_of(&mut self) -> Result<Expression, String>;
    fn parse_dereference(&mut self) -> Result<Expression, String>;
    fn parse_function_pointer_type(&mut self) -> Result<Type, String>;
    fn parse_pointer_arithmetic(&mut self, left: Expression) -> Result<Expression, String>;
    fn parse_pointer_member_access(&mut self, left: Expression) -> Result<Expression, String>;
    fn parse_array_pointer_type(&mut self) -> Result<Type, String>;
    fn parse_pointer_array_type(&mut self) -> Result<Type, String>;
    fn parse_array_pointer_access(&mut self, left: Expression) -> Result<Expression, String>;
    fn parse_pointer_array_access(&mut self, left: Expression) -> Result<Expression, String>;
    fn is_pointer_type(&self, token: &str) -> bool;
    fn count_pointer_level(&self, tokens: &[String]) -> usize;
}

impl<'a> PointerParser for ParserBase<'a> {
    fn parse_pointer_type(&mut self) -> Result<Type, String> {
        debug_println("开始解析指针类型");

        // 检查是否是可选指针 (?*)
        if self.peek() == Some(&"?".to_string()) {
            self.consume(); // 消费 "?"
            self.expect("*")?; // 期望 "*"

            // 解析指向的类型，支持多级指针
            let target_type = self.parse_pointer_target_type()?;
            debug_println(&format!("解析可选指针类型: ?*{:?}", target_type));

            Ok(Type::OptionalPointer(Box::new(target_type)))
        } else if self.peek() == Some(&"*".to_string()) {
            // 检查是否是数组指针 (*[size]Type)
            if self.peek_ahead(1) == Some(&"[".to_string()) {
                return self.parse_array_pointer_type();
            }

            // 计算指针级别
            let mut level = 0;
            while self.peek() == Some(&"*".to_string()) {
                self.consume(); // 消费 "*"
                level += 1;
            }

            // 检查是否是函数指针
            if self.peek() == Some(&"fn".to_string()) {
                return self.parse_function_pointer_type();
            }

            // 解析指向的类型
            let mut target_type = self.parse_base_type()?;

            // 构建多级指针类型（从内到外）
            for _ in 0..level {
                target_type = Type::Pointer(Box::new(target_type));
            }

            debug_println(&format!("解析{}级指针类型: {:?}", level, target_type));
            Ok(target_type)
        } else if self.peek() == Some(&"[".to_string()) {
            // 检查是否是指针数组 ([size]*Type)
            return self.parse_pointer_array_type();
        } else {
            Err("期望指针类型标记 '*'、'?*' 或 '['".to_string())
        }
    }
    
    fn parse_address_of(&mut self) -> Result<Expression, String> {
        debug_println("开始解析取地址表达式");
        
        // 消费 "&" 符号
        self.expect("&")?;
        
        // 解析被取地址的表达式
        let target_expr = self.parse_primary_expression()?;
        debug_println(&format!("解析取地址表达式: &{:?}", target_expr));
        
        Ok(Expression::AddressOf(Box::new(target_expr)))
    }
    
    fn parse_dereference(&mut self) -> Result<Expression, String> {
        debug_println("开始解析解引用表达式");
        
        // 消费 "*" 符号
        self.expect("*")?;
        
        // 解析被解引用的表达式
        let target_expr = self.parse_primary_expression()?;
        debug_println(&format!("解析解引用表达式: *{:?}", target_expr));
        
        Ok(Expression::Dereference(Box::new(target_expr)))
    }
    
    fn parse_function_pointer_type(&mut self) -> Result<Type, String> {
        debug_println("开始解析函数指针类型");

        self.expect("fn")?;
        self.expect("(")?;

        // 解析参数类型
        let mut param_types = Vec::new();
        while self.peek() != Some(&")".to_string()) {
            let param_type = self.parse_base_type()?;
            param_types.push(param_type);

            if self.peek() == Some(&",".to_string()) {
                self.consume();
            } else {
                break;
            }
        }

        self.expect(")")?;
        self.expect(":")?;  // 使用冒号而不是箭头

        // 解析返回类型
        let return_type = self.parse_base_type()?;

        debug_println(&format!("解析函数指针类型: fn({:?}) : {:?}", param_types, return_type));
        Ok(Type::FunctionPointer(param_types, Box::new(return_type)))
    }

    fn parse_pointer_arithmetic(&mut self, left: Expression) -> Result<Expression, String> {
        debug_println("开始解析指针算术");

        let op = if self.peek() == Some(&"+".to_string()) {
            self.consume();
            PointerArithmeticOp::Add
        } else if self.peek() == Some(&"-".to_string()) {
            self.consume();
            PointerArithmeticOp::Sub
        } else {
            return Err("期望指针算术操作符".to_string());
        };

        let right = self.parse_primary_expression()?;

        debug_println(&format!("解析指针算术: {:?} {:?} {:?}", left, op, right));
        Ok(Expression::PointerArithmetic(Box::new(left), op, Box::new(right)))
    }

    fn is_pointer_type(&self, token: &str) -> bool {
        token == "*" || token == "?*"
    }

    fn count_pointer_level(&self, tokens: &[String]) -> usize {
        let mut level = 0;
        for token in tokens {
            if token == "*" {
                level += 1;
            } else {
                break;
            }
        }
        level
    }

    // 新增：解析指针成员访问
    fn parse_pointer_member_access(&mut self, left: Expression) -> Result<Expression, String> {
        debug_println("开始解析指针成员访问");

        // 检查操作符类型
        let op = if self.peek() == Some(&"->".to_string()) {
            self.consume(); // 消费 "->"
            PointerMemberAccessOp::Arrow
        } else if self.peek() == Some(&".".to_string()) {
            self.consume(); // 消费 "."
            PointerMemberAccessOp::Dot
        } else {
            return Err("期望指针成员访问操作符 '->' 或 '.'".to_string());
        };

        // 解析成员名
        let member_name = self.consume().ok_or_else(|| "期望成员名".to_string())?;

        debug_println(&format!("解析指针成员访问: {:?} {:?} {}", left, op, member_name));
        Ok(Expression::PointerMemberAccess(Box::new(left), member_name))
    }

    // 新增：解析数组指针类型 (*[size]Type)
    fn parse_array_pointer_type(&mut self) -> Result<Type, String> {
        debug_println("开始解析数组指针类型");

        self.expect("*")?; // 消费 "*"
        self.expect("[")?; // 消费 "["

        // 解析数组大小
        let size_token = self.consume().ok_or_else(|| "期望数组大小".to_string())?;
        let size = size_token.parse::<usize>()
            .map_err(|_| format!("无效的数组大小: {}", size_token))?;

        self.expect("]")?; // 消费 "]"

        // 解析元素类型
        let element_type = self.parse_base_type()?;

        debug_println(&format!("解析数组指针类型: *[{}]{:?}", size, element_type));
        Ok(Type::ArrayPointer(Box::new(element_type), size))
    }

    // 新增：解析指针数组类型 ([size]*Type)
    fn parse_pointer_array_type(&mut self) -> Result<Type, String> {
        debug_println("开始解析指针数组类型");

        self.expect("[")?; // 消费 "["

        // 解析数组大小
        let size_token = self.consume().ok_or_else(|| "期望数组大小".to_string())?;
        let size = size_token.parse::<usize>()
            .map_err(|_| format!("无效的数组大小: {}", size_token))?;

        self.expect("]")?; // 消费 "]"
        self.expect("*")?; // 消费 "*"

        // 解析指针目标类型
        let target_type = self.parse_base_type()?;

        debug_println(&format!("解析指针数组类型: [{}]*{:?}", size, target_type));
        Ok(Type::PointerArray(Box::new(target_type), size))
    }

    // 新增：解析数组指针访问 ((*arrayPtr)[index])
    fn parse_array_pointer_access(&mut self, left: Expression) -> Result<Expression, String> {
        debug_println("开始解析数组指针访问");

        self.expect("[")?; // 消费 "["

        // 解析索引表达式
        let index_expr = self.parse_expression()?;

        self.expect("]")?; // 消费 "]"

        debug_println(&format!("解析数组指针访问: {:?}[{:?}]", left, index_expr));
        Ok(Expression::ArrayPointerAccess(Box::new(left), Box::new(index_expr)))
    }

    // 新增：解析指针数组访问 (ptrArray[index])
    fn parse_pointer_array_access(&mut self, left: Expression) -> Result<Expression, String> {
        debug_println("开始解析指针数组访问");

        self.expect("[")?; // 消费 "["

        // 解析索引表达式
        let index_expr = self.parse_expression()?;

        self.expect("]")?; // 消费 "]"

        debug_println(&format!("解析指针数组访问: {:?}[{:?}]", left, index_expr));
        Ok(Expression::PointerArrayAccess(Box::new(left), Box::new(index_expr)))
    }
}

impl<'a> ParserBase<'a> {
    // 辅助方法：检查是否是指针操作符
    pub fn is_pointer_operator(&self, token: &str) -> bool {
        token == "&" || token == "*" || token == "->" || token == "."
    }


    
    // 辅助方法：解析指针目标类型（支持多级指针）
    fn parse_pointer_target_type(&mut self) -> Result<Type, String> {
        if self.peek() == Some(&"*".to_string()) {
            // 递归解析多级指针
            self.parse_pointer_type()
        } else {
            self.parse_base_type()
        }
    }

    // 辅助方法：解析基础类型（不包括指针类型）
    fn parse_base_type(&mut self) -> Result<Type, String> {
        let type_name = self.consume().ok_or_else(|| "期望类型名".to_string())?;

        match type_name.as_str() {
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "long" => Ok(Type::Long),
            "void" => Ok(Type::Void),
            "auto" => Ok(Type::Auto),
            _ => {
                // 检查是否是数组类型
                if type_name.ends_with("[]") {
                    let base_type_str = &type_name[..type_name.len() - 2];
                    let base_type = self.parse_type_from_string(base_type_str)?;
                    Ok(Type::Array(Box::new(base_type)))
                } else {
                    // 假设是类类型或枚举类型
                    Ok(Type::Class(type_name))
                }
            }
        }
    }

    // 辅助方法：解析指针类型字符串
    pub fn parse_pointer_type_from_string(&self, type_str: &str) -> Result<Type, String> {
        if type_str.starts_with("?*") {
            // 可选指针类型
            let target_type_str = &type_str[2..];
            let target_type = self.parse_type_from_string(target_type_str)?;
            Ok(Type::OptionalPointer(Box::new(target_type)))
        } else if type_str.starts_with("*") {
            // 普通指针类型
            let target_type_str = &type_str[1..];
            let target_type = self.parse_type_from_string(target_type_str)?;
            Ok(Type::Pointer(Box::new(target_type)))
        } else {
            Err(format!("无效的指针类型字符串: {}", type_str))
        }
    }
    

}
