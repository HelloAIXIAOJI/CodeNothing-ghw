// 错误处理模块
// 包含错误信息处理和行号计算功能

/// 添加行号信息到错误消息
pub fn add_line_info(source: &str, error_msg: &str) -> String {
    // 如果错误消息中已经包含行号信息，就直接返回
    if error_msg.contains("第") && error_msg.contains("行") {
        return error_msg.to_string();
    }

    // 如果错误消息包含位置信息，尝试添加行号
    if let Some(pos_start) = error_msg.find("(位置:") {
        if let Some(pos_end) = error_msg[pos_start..].find(")") {
            let pos_str = &error_msg[pos_start+4..pos_start+pos_end];
            if let Ok(pos) = pos_str.trim().parse::<usize>() {
                // 计算行号和列号
                let mut line = 1;
                let mut col = 1;
                for (i, c) in source.chars().enumerate() {
                    if i == pos {
                        break;
                    }
                    if c == '\n' {
                        line += 1;
                        col = 1;
                    } else {
                        col += 1;
                    }
                }
                
                // 添加行号和列号信息，使用更清晰的格式
                let base_msg = error_msg[0..pos_start].trim();
                return format!("{} [第{}行,第{}列]", base_msg, line, col);
            }
        }
    }
    
    // 如果没有位置信息，返回原始错误
    error_msg.to_string()
} 