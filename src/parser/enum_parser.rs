use crate::ast::{Enum, EnumVariant, EnumField, Type};
use crate::parser::parser_base::ParserBase;
use crate::parser::statement_parser::StatementParser;
use crate::interpreter::debug_println;

pub trait EnumParser {
    fn parse_enum(&mut self) -> Result<Enum, String>;
    fn parse_enum_variant(&mut self) -> Result<EnumVariant, String>;
    fn parse_enum_field(&mut self) -> Result<EnumField, String>;
}

impl<'a> EnumParser for ParserBase<'a> {
    fn parse_enum(&mut self) -> Result<Enum, String> {
        debug_println("开始解析枚举");
        
        // 消费 "enum" 关键字
        self.expect("enum")?;
        
        // 获取枚举名
        let enum_name = self.consume().ok_or_else(|| "期望枚举名".to_string())?;
        debug_println(&format!("解析枚举: {}", enum_name));
        
        // 期望 "{"
        self.expect("{")?;
        
        let mut variants = Vec::new();
        
        // 解析枚举变体
        while self.peek() != Some(&"}".to_string()) {
            let variant = self.parse_enum_variant()?;
            variants.push(variant);
            
            // 检查是否有逗号分隔符
            if self.peek() == Some(&",".to_string()) {
                self.consume(); // 消费 ","
            } else if self.peek() != Some(&"}".to_string()) {
                return Err("期望 ',' 或 '}'".to_string());
            }
        }
        
        // 期望 "}"
        self.expect("}")?;
        self.expect(";")?;
        
        debug_println(&format!("枚举解析完成: {} (变体数: {})", enum_name, variants.len()));
        
        Ok(Enum {
            name: enum_name,
            variants,
        })
    }
    
    fn parse_enum_variant(&mut self) -> Result<EnumVariant, String> {
        debug_println("开始解析枚举变体");

        // 获取变体名
        let variant_name = self.consume().ok_or_else(|| "期望枚举变体名".to_string())?;
        debug_println(&format!("解析枚举变体: {}", variant_name));

        let mut fields = Vec::new();

        // 检查是否有显式值赋值（如 Success = 0）
        if self.peek() == Some(&"=".to_string()) {
            self.consume(); // 消费 "="
            // 跳过值，暂时不处理显式值
            self.consume(); // 消费值
        }

        // 检查是否有字段定义
        if self.peek() == Some(&"(".to_string()) {
            self.consume(); // 消费 "("

            // 解析字段列表
            while self.peek() != Some(&")".to_string()) {
                let field = self.parse_enum_field()?;
                fields.push(field);

                if self.peek() == Some(&",".to_string()) {
                    self.consume(); // 消费 ","
                } else if self.peek() != Some(&")".to_string()) {
                    return Err("期望 ',' 或 ')'".to_string());
                }
            }

            self.expect(")")?; // 期望 ")"
        }

        debug_println(&format!("枚举变体解析完成: {} (字段数: {})", variant_name, fields.len()));

        Ok(EnumVariant {
            name: variant_name,
            fields,
        })
    }
    
    fn parse_enum_field(&mut self) -> Result<EnumField, String> {
        debug_println("开始解析枚举字段");
        
        // 检查是否是命名字段（name : type）还是匿名字段（type）
        let first_token = self.consume().ok_or_else(|| "期望字段类型或字段名".to_string())?;
        
        if self.peek() == Some(&":".to_string()) {
            // 命名字段：name : type
            self.consume(); // 消费 ":"
            let field_type = self.parse_type()?;
            
            debug_println(&format!("解析命名字段: {} : {:?}", first_token, field_type));
            
            Ok(EnumField {
                name: Some(first_token),
                field_type,
            })
        } else {
            // 匿名字段：type
            // 将first_token作为类型名解析
            let field_type = self.parse_type_from_string(&first_token)?;
            
            debug_println(&format!("解析匿名字段: {:?}", field_type));
            
            Ok(EnumField {
                name: None,
                field_type,
            })
        }
    }
}

impl<'a> ParserBase<'a> {
    // 辅助方法：从字符串解析类型
    pub fn parse_type_from_string(&self, type_str: &str) -> Result<Type, String> {
        match type_str {
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "long" => Ok(Type::Long),
            "void" => Ok(Type::Void),
            "auto" => Ok(Type::Auto),
            _ => {
                // 检查是否是数组类型
                if type_str.ends_with("[]") {
                    let base_type_str = &type_str[..type_str.len() - 2];
                    let base_type = self.parse_type_from_string(base_type_str)?;
                    Ok(Type::Array(Box::new(base_type)))
                } else {
                    // 假设是类类型或枚举类型
                    Ok(Type::Class(type_str.to_string()))
                }
            }
        }
    }
}
