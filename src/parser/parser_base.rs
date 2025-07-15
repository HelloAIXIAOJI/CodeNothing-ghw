// 解析器基础结构，提供基本的词法分析和解析功能

pub struct ParserBase<'a> {
    pub source: &'a str,
    pub tokens: Vec<String>,
    pub position: usize,
    pub debug: bool,
    pub line_map: Vec<usize>, // 添加行号映射表，存储每行的起始位置
}

impl<'a> ParserBase<'a> {
    pub fn new(source: &'a str, tokens: Vec<String>, debug: bool) -> Self {
        // 构建行号映射表
        let mut line_map = Vec::new();
        line_map.push(0); // 第一行从0开始
        
        for (i, c) in source.chars().enumerate() {
            if c == '\n' {
                line_map.push(i + 1);
            }
        }
        
        ParserBase {
            source,
            tokens,
            position: 0,
            debug,
            line_map,
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
    
    // 获取当前位置的行号和列号
    pub fn get_line_column(&self) -> (usize, usize) {
        let token_pos = if self.position < self.tokens.len() {
            // 查找当前token在源码中的位置
            let current_token = &self.tokens[self.position];
            self.source.find(current_token).unwrap_or(0)
        } else if !self.tokens.is_empty() {
            // 如果已经到末尾，使用最后一个token的位置
            let last_token = &self.tokens[self.tokens.len() - 1];
            let pos = self.source.find(last_token).unwrap_or(0);
            pos + last_token.len()
        } else {
            0
        };
        
        // 根据位置计算行号和列号
        let mut line = 1;
        let mut column = 1;
        
        // 使用行号映射表快速查找行号
        for (i, &line_start) in self.line_map.iter().enumerate() {
            if line_start > token_pos {
                line = i;
                if i > 0 {
                    column = token_pos - self.line_map[i-1] + 1;
                } else {
                    column = token_pos + 1;
                }
                break;
            }
        }
        
        // 如果没找到，说明是最后一行
        if line == 1 && self.line_map.len() > 1 {
            let last_line = self.line_map.len();
            line = last_line;
            column = token_pos - self.line_map[last_line-1] + 1;
        }
        
        (line, column)
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
                let (line, column) = self.get_line_column();
                Err(format!("期望 '{}', 但得到了 '{}' (行: {}, 列: {}, 位置: {})", 
                    expected, token, line, column, self.position))
            }
        } else {
            let (line, column) = self.get_line_column();
            Err(format!("期望 '{}', 但到达了文件末尾 (行: {}, 列: {}, 位置: {})", 
                expected, line, column, self.position))
        }
    }
    
    // 创建带有位置信息的错误
    pub fn create_error(&self, message: &str) -> String {
        let (line, column) = self.get_line_column();
        format!("{} (行: {}, 列: {}, 位置: {})", message, line, column, self.position)
    }
}