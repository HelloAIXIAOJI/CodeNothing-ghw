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
        if self.debug {
            println!("期望标记符: {}", expected);
            println!("下一个token: {:?}", self.peek());
        }
        
        if let Some(token) = self.consume() {
            if token == expected {
                Ok(())
            } else {
                Err(format!("期望 '{}', 但得到了 '{}' (位置: {})", expected, token, self.position))
            }
        } else {
            Err(format!("期望 '{}', 但到达了文件末尾 (位置: {})", expected, self.position))
        }
    }
    
    // 向前查看指定偏移量的词法单元
    pub fn peek_ahead(&self, offset: usize) -> Option<&String> {
        self.tokens.get(self.position + offset)
    }

    // 检查当前token是否是指定的符号，不消费
    pub fn check_symbol(&self, symbol: &str) -> bool {
        if let Some(token) = self.peek() {
            token == symbol
        } else {
            false
        }
    }

    // 如果当前token是指定的符号，则消费并返回true，否则返回false
    pub fn consume_symbol(&mut self, symbol: &str) -> bool {
        if self.check_symbol(symbol) {
            self.consume();
            true
        } else {
            false
        }
    }

    // 检查当前token是否是指定的关键字，不消费
    pub fn check_keyword(&self, keyword: &str) -> bool {
        if let Some(token) = self.peek() {
            token == keyword
        } else {
            false
        }
    }

    // 如果当前token是指定的关键字，则消费并返回true，否则返回false
    pub fn consume_keyword(&mut self, keyword: &str) -> bool {
        if self.check_keyword(keyword) {
            self.consume();
            true
        } else {
            false
        }
    }

    // 消费当前token并返回，类似consume但名字不同
    pub fn advance(&mut self) -> Option<String> {
        self.consume()
    }

    // 检查是否到达了输入的末尾
    pub fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}