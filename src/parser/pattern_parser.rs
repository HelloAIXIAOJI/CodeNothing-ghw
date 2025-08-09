// 模式匹配解析器
use crate::ast::{Pattern, MatchArm, Expression, Statement};
use crate::parser::parser_base::ParserBase;
use crate::parser::expression_parser::ExpressionParser;
use crate::parser::statement_parser::StatementParser;


pub trait PatternParser {
    fn parse_match_statement(&mut self) -> Result<(Expression, Vec<MatchArm>), String>;
    fn parse_match_expression(&mut self) -> Result<(Expression, Vec<MatchArm>), String>;
    fn parse_match_arms(&mut self) -> Result<Vec<MatchArm>, String>;
    fn parse_match_arm(&mut self) -> Result<MatchArm, String>;
    fn parse_pattern(&mut self) -> Result<Pattern, String>;
    fn parse_pattern_primary(&mut self) -> Result<Pattern, String>;
    fn parse_pattern_or(&mut self) -> Result<Pattern, String>;
    fn parse_guard_condition(&mut self) -> Result<Option<Expression>, String>;
}

impl<'a> PatternParser for ParserBase<'a> {
    /// 解析match语句
    fn parse_match_statement(&mut self) -> Result<(Expression, Vec<MatchArm>), String> {
        // 调试输出
        if self.debug {
            println!("开始解析match语句");
        }
        
        // 消费 'match' 关键字
        if !self.consume_keyword("match") {
            return Err("期望 'match' 关键字".to_string());
        }
        
        // 解析匹配表达式
        if !self.consume_symbol("(") {
            return Err("期望 '(' 在match表达式之前".to_string());
        }
        
        let match_expr = self.parse_expression()?;
        
        if !self.consume_symbol(")") {
            return Err("期望 ')' 在match表达式之后".to_string());
        }
        
        // 解析匹配分支
        if !self.consume_symbol("{") {
            return Err("期望 '{' 开始match分支".to_string());
        }
        
        let arms = self.parse_match_arms()?;
        
        if !self.consume_symbol("}") {
            return Err("期望 '}' 结束match分支".to_string());
        }
        
        if !self.consume_symbol(";") {
            return Err("期望 ';' 结束match语句".to_string());
        }
        
        if self.debug {
            println!("match语句解析完成");
        }
        Ok((match_expr, arms))
    }
    
    /// 解析match表达式
    fn parse_match_expression(&mut self) -> Result<(Expression, Vec<MatchArm>), String> {
        if self.debug {
            println!("开始解析match表达式");
        }
        
        // 消费 'match' 关键字
        if !self.consume_keyword("match") {
            return Err("期望 'match' 关键字".to_string());
        }
        
        // 解析匹配表达式
        if !self.consume_symbol("(") {
            return Err("期望 '(' 在match表达式之前".to_string());
        }
        
        let match_expr = self.parse_expression()?;
        
        if !self.consume_symbol(")") {
            return Err("期望 ')' 在match表达式之后".to_string());
        }
        
        // 解析匹配分支
        if !self.consume_symbol("{") {
            return Err("期望 '{' 开始match分支".to_string());
        }
        
        let arms = self.parse_match_arms()?;
        
        if !self.consume_symbol("}") {
            return Err("期望 '}' 结束match分支".to_string());
        }
        
        if self.debug {
            println!("match表达式解析完成");
        }
        Ok((match_expr, arms))
    }
    
    /// 解析所有匹配分支
    fn parse_match_arms(&mut self) -> Result<Vec<MatchArm>, String> {
        let mut arms = Vec::new();
        
        while !self.check_symbol("}") && !self.is_at_end() {
            let arm = self.parse_match_arm()?;
            arms.push(arm);
        }
        
        if arms.is_empty() {
            return Err("match语句至少需要一个分支".to_string());
        }
        
        Ok(arms)
    }
    
    /// 解析单个匹配分支
    fn parse_match_arm(&mut self) -> Result<MatchArm, String> {
        self.debug_println("开始解析match分支");
        
        // 解析模式
        let pattern = self.parse_pattern()?;
        
        // 解析可选的守卫条件
        let guard = self.parse_guard_condition()?;
        
        // 期望 '=>' 符号
        if !self.consume_symbol("=") || !self.consume_symbol(">") {
            return Err("期望 '=>' 在模式之后".to_string());
        }
        
        // 解析分支体
        let body = if self.check_symbol("{") {
            // 语句块
            self.consume_symbol("{");
            let mut statements = Vec::new();
            
            while !self.check_symbol("}") && !self.is_at_end() {
                let stmt = self.parse_statement()?;
                statements.push(stmt);
            }
            
            if !self.consume_symbol("}") {
                return Err("期望 '}' 结束match分支体".to_string());
            }
            
            statements
        } else {
            // 单个表达式语句
            let expr = self.parse_expression()?;
            vec![Statement::FunctionCallStatement(expr)]
        };
        
        // 可选的分号
        self.consume_symbol(";");
        
        self.debug_println("match分支解析完成");
        Ok(MatchArm {
            pattern,
            guard,
            body,
        })
    }
    
    /// 解析模式
    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        self.parse_pattern_or()
    }
    
    /// 解析或模式 (pattern1 | pattern2 | pattern3)
    fn parse_pattern_or(&mut self) -> Result<Pattern, String> {
        let mut patterns = vec![self.parse_pattern_primary()?];
        
        while self.consume_symbol("|") {
            patterns.push(self.parse_pattern_primary()?);
        }
        
        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            Ok(Pattern::Or(patterns))
        }
    }
    
    /// 解析基础模式
    fn parse_pattern_primary(&mut self) -> Result<Pattern, String> {
        if let Some(token) = self.peek() {
            match token.as_str() {
                // 通配符模式
                "_" => {
                    self.advance();
                    Ok(Pattern::Wildcard)
                },
                
                // 元组模式
                "(" => {
                    self.advance();
                    let mut patterns = Vec::new();
                    
                    if !self.check_symbol(")") {
                        patterns.push(self.parse_pattern()?);
                        
                        while self.consume_symbol(",") {
                            if self.check_symbol(")") {
                                break;
                            }
                            patterns.push(self.parse_pattern()?);
                        }
                    }
                    
                    if !self.consume_symbol(")") {
                        return Err("期望 ')' 结束元组模式".to_string());
                    }
                    
                    Ok(Pattern::Tuple(patterns))
                },
                
                // 数组模式
                "[" => {
                    self.advance();
                    let mut patterns = Vec::new();
                    
                    if !self.check_symbol("]") {
                        patterns.push(self.parse_pattern()?);
                        
                        while self.consume_symbol(",") {
                            if self.check_symbol("]") {
                                break;
                            }
                            patterns.push(self.parse_pattern()?);
                        }
                    }
                    
                    if !self.consume_symbol("]") {
                        return Err("期望 ']' 结束数组模式".to_string());
                    }
                    
                    Ok(Pattern::Array(patterns))
                },
                
                // 字符串字面量模式
                s if s.starts_with('"') && s.ends_with('"') => {
                    self.advance();
                    let content = s[1..s.len()-1].to_string();
                    Ok(Pattern::StringLiteral(content))
                },
                
                // 布尔字面量模式
                "true" => {
                    self.advance();
                    Ok(Pattern::BoolLiteral(true))
                },
                "false" => {
                    self.advance();
                    Ok(Pattern::BoolLiteral(false))
                },
                
                // 数字字面量模式
                s if s.chars().next().unwrap().is_digit(10) || s.starts_with('-') => {
                    self.advance();
                    if s.contains('.') {
                        if let Ok(value) = s.parse::<f64>() {
                            Ok(Pattern::FloatLiteral(value))
                        } else {
                            Err(format!("无效的浮点数字面量: {}", s))
                        }
                    } else {
                        if let Ok(value) = s.parse::<i32>() {
                            Ok(Pattern::IntLiteral(value))
                        } else {
                            Err(format!("无效的整数字面量: {}", s))
                        }
                    }
                },
                
                // 变量模式或枚举模式
                _ if token.chars().next().unwrap().is_alphabetic() || token.starts_with('_') => {
                    let name = self.advance().unwrap();
                    
                    // 检查是否是枚举模式 (EnumName::Variant)
                    if self.consume_symbol("::") {
                        let variant = self.advance().ok_or("期望枚举变体名称")?;
                        
                        // 检查是否有参数
                        let params = if self.consume_symbol("(") {
                            let mut patterns = Vec::new();
                            
                            if !self.check_symbol(")") {
                                patterns.push(self.parse_pattern()?);
                                
                                while self.consume_symbol(",") {
                                    if self.check_symbol(")") {
                                        break;
                                    }
                                    patterns.push(self.parse_pattern()?);
                                }
                            }
                            
                            if !self.consume_symbol(")") {
                                return Err("期望 ')' 结束枚举变体参数".to_string());
                            }
                            
                            patterns
                        } else {
                            Vec::new()
                        };
                        
                        Ok(Pattern::EnumVariant(name, variant, params))
                    } else {
                        // 普通变量模式
                        Ok(Pattern::Variable(name))
                    }
                },
                
                _ => Err(format!("无效的模式: {}", token))
            }
        } else {
            Err("意外的输入结束".to_string())
        }
    }
    
    /// 解析守卫条件
    fn parse_guard_condition(&mut self) -> Result<Option<Expression>, String> {
        if self.consume_keyword("if") {
            let condition = self.parse_expression()?;
            Ok(Some(condition))
        } else {
            Ok(None)
        }
    }
}
