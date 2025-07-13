// 词法分析器：负责将源代码转换为词法单元（tokens）

// 移除注释
pub fn remove_comments(source: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    
    let chars: Vec<char> = source.chars().collect();
    
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            // 单行注释：跳过到行尾
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            // 保留换行符
            if i < chars.len() && chars[i] == '\n' {
                result.push(chars[i]);
                i += 1;
            }
        } else if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '!' {
            // 多行注释：跳过直到找到 !/
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '!' && chars[i + 1] == '/') {
                i += 1;
            }
            // 跳过结束标记
            if i + 1 < chars.len() {
                i += 2;
            }
        } else {
            // 非注释内容
            result.push(chars[i]);
            i += 1;
        }
    }
    
    result
}

// 词法分析：将源代码转换为词法单元
pub fn tokenize(source: &str, debug: bool) -> Vec<String> {
    // 预处理字符串，保留字符串字面量
    let mut processed_source = String::new();
    let mut in_string = false;
    let mut escape = false;
    let mut current_string = String::new();
    let mut string_placeholders = Vec::new();
    
    // 先处理字符串
    for c in source.chars() {
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
                processed_source.push_str(&format!(" __STRING_{} ", string_placeholders.len() - 1));
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
    
    // 特殊处理命名空间分隔符，确保它被当作一个整体处理
    processed_source = processed_source.replace("::", " __NS_SEP__ ");
    
    // 特殊处理范围操作符，确保它被当作一个整体处理
    processed_source = processed_source.replace("..", " __RANGE_OP__ ");
    
    // 特殊处理复合操作符，必须在处理单个符号之前
    processed_source = processed_source
        .replace("++", " __INC_OP__ ")
        .replace("--", " __DEC_OP__ ")
        .replace("+=", " __ADD_ASSIGN__ ")
        .replace("-=", " __SUB_ASSIGN__ ")
        .replace("*=", " __MUL_ASSIGN__ ")
        .replace("/=", " __DIV_ASSIGN__ ")
        .replace("%=", " __MOD_ASSIGN__ ")
        .replace("==", " __EQ__ ")
        .replace("!=", " __NEQ__ ")
        .replace(">=", " __GTE__ ")
        .replace("<=", " __LTE__ ")
        .replace("&&", " __AND__ ")
        .replace("||", " __OR__ ")
        .replace("!", " __NOT__ ");
    
    // 处理其他分隔符
    let tokens = processed_source
        .replace(";", " ; ")
        .replace("(", " ( ")
        .replace(")", " ) ")
        .replace("{", " { ")
        .replace("}", " } ")
        .replace(":", " : ")
        .replace("=", " = ")
        .replace("+", " + ")
        .replace("-", " - ")
        .replace("*", " * ")
        .replace("/", " / ")
        .replace("%", " % ")
        .replace("[", " [ ")
        .replace("]", " ] ")
        .replace(",", " , ")
        .replace("<", " < ")
        .replace(">", " > ")
        .split_whitespace()
        .map(|s| {
            if s.starts_with("__STRING_") {
                let idx = s.trim_start_matches("__STRING_").parse::<usize>().unwrap();
                format!("\"{}\"", string_placeholders[idx])
            } else if s == "__NS_SEP__" {
                "::".to_string()
            } else if s == "__RANGE_OP__" {
                "..".to_string()
            } else if s == "__INC_OP__" {
                "++".to_string()
            } else if s == "__DEC_OP__" {
                "--".to_string()
            } else if s == "__ADD_ASSIGN__" {
                "+=".to_string()
            } else if s == "__SUB_ASSIGN__" {
                "-=".to_string()
            } else if s == "__MUL_ASSIGN__" {
                "*=".to_string()
            } else if s == "__DIV_ASSIGN__" {
                "/=".to_string()
            } else if s == "__MOD_ASSIGN__" {
                "%=".to_string()
            } else if s == "__EQ__" {
                "==".to_string()
            } else if s == "__NEQ__" {
                "!=".to_string()
            } else if s == "__GTE__" {
                ">=".to_string()
            } else if s == "__LTE__" {
                "<=".to_string()
            } else if s == "__AND__" {
                "&&".to_string()
            } else if s == "__OR__" {
                "||".to_string()
            } else if s == "__NOT__" {
                "!".to_string()
            } else {
                s.to_string()
            }
        })
        .collect::<Vec<String>>();
    
    if debug {
        println!("词法分析结果: {:?}", tokens);
    }
    
    tokens
} 