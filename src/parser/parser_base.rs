// 解析器基础结构，提供基本的词法分析和解析功能

// 错误位置信息结构体
#[derive(Debug, Clone)]
pub struct ErrorLocation {
    pub message: String,
    pub token_position: usize,    // token在tokens数组中的索引
    pub token: Option<String>,
}

pub struct ParserBase<'a> {
    pub source: &'a str,
    pub tokens: Vec<String>,
    pub position: usize,
    pub debug: bool,
    pub errors: Vec<ErrorLocation>, // 错误收集器
}

impl<'a> ParserBase<'a> {
    pub fn new(source: &'a str, tokens: Vec<String>, debug: bool) -> Self {
        ParserBase {
            source,
            tokens,
            position: 0,
            debug,
            errors: Vec::new(), // 初始化错误收集器
        }
    }
    
    // 查看当前词法单元，不消费
    pub fn peek(&self) -> Option<&String> {
        self.tokens.get(self.position)
    }
    
    // 消费当前词法单元并返回
    pub fn consume(&mut self) -> Option<String> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }
    
    // 添加错误而不中断解析
    pub fn add_error(&mut self, message: String) {
        let token = self.peek().cloned();
        self.errors.push(ErrorLocation {
            message,
            token_position: self.position,
            token,
        });
    }
    
    // 期望下一个词法单元是指定的值，如果是则消费，否则返回错误
    pub fn expect(&mut self, expected: &str) -> Result<(), String> {
        if self.debug {
            println!("期望标记符: {}", expected);
            println!("下一个token: {:?}", self.peek());
        }
        
        if let Some(token) = self.consume() {
            if token == expected {
                Ok(())
            } else {
                let error_msg = format!("期望 '{}', 但得到了 '{}' (位置: {})", expected, token, self.position);
                self.add_error(error_msg.clone());
                Err(error_msg)
            }
        } else {
            let error_msg = format!("期望 '{}', 但到达了文件末尾 (位置: {})", expected, self.position);
            self.add_error(error_msg.clone());
            Err(error_msg)
        }
    }
    
    // 获取收集到的所有错误
    pub fn get_errors(&self) -> &[ErrorLocation] {
        &self.errors
    }
    
    // 判断是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
} 