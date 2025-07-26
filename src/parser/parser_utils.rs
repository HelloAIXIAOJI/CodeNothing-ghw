// 解析器工具模块
// 包含各种跳过和辅助函数

use crate::parser::parser_base::ParserBase;

/// 跳过当前项，找到下一个顶层项（函数、命名空间或导入）的开始
pub fn skip_to_next_top_level_item(parser: &mut ParserBase) {
    let mut brace_count = 0;
    
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if brace_count == 0 && (token == "fn" || token == "ns" || token == "using" || token == "class" || token == "abstract" || token == "interface" || token == "enum") {
                // 找到下一个顶层项
                return;
            } else if token == "{" {
                brace_count += 1;
                parser.consume();
            } else if token == "}" {
                if brace_count > 0 {
                    brace_count -= 1;
                }
                parser.consume();
            } else {
                parser.consume();
            }
        } else {
            break;
        }
    }
}

/// 跳过命名空间内的当前项，找到下一个成员（函数或嵌套命名空间）的开始
pub fn skip_to_next_ns_member(parser: &mut ParserBase) {
    let mut brace_count = 0;
    
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if token == "{" {
                brace_count += 1;
                parser.consume();
            } else if token == "}" {
                if brace_count == 0 {
                    return; // 找到了命名空间的结束
                }
                brace_count -= 1;
                parser.consume();
            } else if brace_count == 0 && (token == "fn" || token == "ns") {
                return; // 找到了下一个成员
            } else {
                parser.consume();
            }
        }
    }
}

/// 跳过到下一个语句开始或函数结束
pub fn skip_to_next_statement_or_end(parser: &mut ParserBase) {
    let mut brace_count = 0;
    
    while parser.position < parser.tokens.len() {
        if let Some(token) = parser.peek() {
            if token == ";" {
                parser.consume();
                return; // 找到了语句的结束
            } else if token == "{" {
                brace_count += 1;
                parser.consume();
            } else if token == "}" {
                if brace_count == 0 {
                    return; // 找到了函数的结束
                }
                brace_count -= 1;
                parser.consume();
            } else {
                parser.consume();
            }
        }
    }
} 