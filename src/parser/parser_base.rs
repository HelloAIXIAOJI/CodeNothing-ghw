// 解析器基础结构，提供基本的词法分析和解析功能

pub struct ParserBase<'a> {
    pub source: &'a str,
    pub tokens: Vec<String>,
    pub position: usize,
    pub debug: bool,
}

impl<'a> ParserBase<'a> {
    pub fn new(source: &'a str, tokens: Vec<String>, debug: bool) -> Self {
        ParserBase {
            source,
            tokens,
            position: 0,
            debug,
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
    
    // 期望下一个词法单元是指定的值，如果是则消费，否则返回错误
    pub fn expect(&mut self, expected: &str) -> Result<(), String> {
        if let Some(token) = self.consume() {
            if token == expected {
                Ok(())
            } else {
                Err(format!("期望 '{}', 但得到了 '{}'", expected, token))
            }
        } else {
            Err(format!("期望 '{}', 但到达了文件末尾", expected))
        }
    }
} 