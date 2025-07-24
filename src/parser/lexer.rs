use crate::interpreter::debug_println;

// 词法分析器：负责将源代码转换为词法单元（tokens）

// 移除注释
pub fn remove_comments(source: &str) -> String {
    let mut result = String::new();
    let mut in_single_line_comment = false;
    let mut multi_line_comment_depth = 0; // 使用计数器跟踪多行注释的嵌套深度
    let mut in_string = false; // 标记是否在字符串内
    let mut escape = false; // 标记是否是转义字符
    let mut i = 0;
    
    let chars: Vec<char> = source.chars().collect();
    
    while i < chars.len() {
        // 处理字符串
        if in_string {
            result.push(chars[i]);
            if escape {
                // 转义字符后的字符直接添加
                escape = false;
            } else if chars[i] == '\\' {
                // 转义字符标记
                escape = true;
            } else if chars[i] == '"' {
                // 字符串结束
                in_string = false;
            }
            i += 1;
            continue;
        } else if chars[i] == '"' {
            // 字符串开始
            in_string = true;
            result.push(chars[i]);
            i += 1;
            continue;
        }
        
        // 处理注释
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' && multi_line_comment_depth == 0 {
            // 单行注释开始（仅当不在多行注释中时）
            in_single_line_comment = true;
            i += 2;
        } else if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '!' && !in_single_line_comment {
            // 多行注释开始
            multi_line_comment_depth += 1;
            i += 2;
        } else if in_single_line_comment && chars[i] == '\n' {
            // 单行注释结束
            in_single_line_comment = false;
            result.push(chars[i]);
            i += 1;
        } else if i + 1 < chars.len() && chars[i] == '!' && chars[i + 1] == '/' && !in_single_line_comment {
            // 多行注释结束
            if multi_line_comment_depth > 0 {
                multi_line_comment_depth -= 1;
            }
            i += 2;
        } else if !in_single_line_comment && multi_line_comment_depth == 0 {
            // 非注释内容
            result.push(chars[i]);
            i += 1;
        } else {
            // 在注释内，跳过
            i += 1;
        }
    }
    
    result
}

// 词法分析：将源代码转换为词法单元
pub fn tokenize(source: &str, debug: bool) -> Vec<String> {
    // 1. 移除注释
    let source_without_comments = remove_comments(source);

    // 2. 将字符串字面量替换为占位符
    let mut processed_source = String::new();
    let mut string_placeholders = Vec::new();
    let mut in_string = false;
    let mut escape = false;
    let mut current_string = String::new();

    for c in source_without_comments.chars() {
        if in_string {
            if escape {
                current_string.push(c);
                escape = false;
            } else if c == '\\' {
                current_string.push(c);
                escape = true;
            } else if c == '"' {
                in_string = false;
                string_placeholders.push(current_string.clone());
                processed_source.push_str(&format!(" __STRING_{}__ ", string_placeholders.len() - 1));
                current_string.clear();
            } else {
                current_string.push(c);
            }
        } else if c == '"' {
            in_string = true;
        } else {
            processed_source.push(c);
        }
    }
    
    // 3. 逐字符进行词法分析
    let mut tokens = Vec::new();
    let mut chars = processed_source.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next(); // Skip whitespace
            continue;
        }

        // 检查多字符运算符
        let next_char = chars.clone().nth(1);
        if let Some(nc) = next_char {
            let two_char_op = format!("{}{}", c, nc);
            if ["==", "!=", ">=", "<=", "&&", "||", "::", "..", "++", "--", "+=", "-=", "*=", "/=", "%=", "=>"].contains(&two_char_op.as_str()) {
                tokens.push(two_char_op);
                chars.next();
                chars.next();
                continue;
            }
        }

        // 检查标识符、关键字、或字符串占位符
        if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while let Some(&p) = chars.peek() {
                if p.is_alphanumeric() || p == '_' {
                    s.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(s);
        }
        // 检查数字 (整数或浮点数)
        else if c.is_digit(10) || (c == '.' && chars.clone().nth(1).map_or(false, |c| c.is_digit(10))) {
            let mut s = String::new();
            let mut has_dot = false;
            
            while let Some(&p) = chars.peek() {
                if p.is_digit(10) {
                    s.push(chars.next().unwrap());
                } else if p == '.' && !has_dot {
                    // 检查下一个字符，如果是另一个点，则停止（这是范围操作符）
                    if chars.clone().nth(1) == Some('.') {
                        break;
                    }
                    has_dot = true;
                    s.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(s);
        }
        // 单字符符号
        else {
            tokens.push(chars.next().unwrap().to_string());
        }
    }

    // 4. 恢复字符串占位符
    let final_tokens = tokens.into_iter().map(|s| {
        if s.starts_with("__STRING_") && s.ends_with("__") {
            if let Ok(idx) = s.trim_start_matches("__STRING_").trim_end_matches("__").parse::<usize>() {
                if idx < string_placeholders.len() {
                    return format!("\"{}\"", string_placeholders[idx]);
                }
            }
        }
        s
    }).collect::<Vec<String>>();

    if debug {
        debug_println(&format!("词法分析结果: {:?}", final_tokens));
    }

    final_tokens
} 