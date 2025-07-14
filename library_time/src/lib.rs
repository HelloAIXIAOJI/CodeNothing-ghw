use ::std::collections::HashMap;
use ::std::thread;
use ::std::time::Duration as StdDuration;
use chrono::{Local, Utc, DateTime, Datelike, Timelike, Duration};

// 导入通用库
use cn_common::namespace::{LibraryFunction, NamespaceBuilder, create_library_pointer, LibraryRegistry};

// 命名空间函数
mod std {
    use super::*;
    
    // 获取当前本地时间的时间戳（秒）
    pub fn cn_now(_args: Vec<String>) -> String {
        Local::now().timestamp().to_string()
    }
    
    // 获取当前UTC时间的时间戳（秒）
    pub fn cn_utc_now(_args: Vec<String>) -> String {
        Utc::now().timestamp().to_string()
    }
    
    // 获取当前本地时间的毫秒时间戳
    pub fn cn_now_millis(_args: Vec<String>) -> String {
        let now = Local::now();
        let millis = now.timestamp() * 1000 + now.timestamp_subsec_millis() as i64;
        millis.to_string()
    }
    
    // 获取当前UTC时间的毫秒时间戳
    pub fn cn_utc_now_millis(_args: Vec<String>) -> String {
        let now = Utc::now();
        let millis = now.timestamp() * 1000 + now.timestamp_subsec_millis() as i64;
        millis.to_string()
    }
    
    // 格式化当前本地时间
    // 参数: [format]，默认为 "%Y-%m-%d %H:%M:%S"
    pub fn cn_format_now(args: Vec<String>) -> String {
        let format = if args.is_empty() { "%Y-%m-%d %H:%M:%S" } else { &args[0] };
        Local::now().format(format).to_string()
    }
    
    // 格式化当前UTC时间
    // 参数: [format]，默认为 "%Y-%m-%d %H:%M:%S"
    pub fn cn_format_utc_now(args: Vec<String>) -> String {
        let format = if args.is_empty() { "%Y-%m-%d %H:%M:%S" } else { &args[0] };
        Utc::now().format(format).to_string()
    }
    
    // 从时间戳（秒）格式化时间
    // 参数: timestamp, [format]
    pub fn cn_format_timestamp(args: Vec<String>) -> String {
        if args.is_empty() {
            return "错误: 缺少时间戳参数".to_string();
        }
        
        let timestamp = match args[0].parse::<i64>() {
            Ok(ts) => ts,
            Err(_) => return "错误: 无效的时间戳".to_string(),
        };
        
        let format = if args.len() > 1 { &args[1] } else { "%Y-%m-%d %H:%M:%S" };
        
        let dt = match DateTime::from_timestamp(timestamp, 0) {
            Some(dt) => dt,
            None => return "错误: 无法创建日期时间对象".to_string(),
        };
        
        dt.format(format).to_string()
    }
    
    // 获取当前年份
    pub fn cn_year(_args: Vec<String>) -> String {
        Local::now().year().to_string()
    }
    
    // 获取当前月份
    pub fn cn_month(_args: Vec<String>) -> String {
        Local::now().month().to_string()
    }
    
    // 获取当前日
    pub fn cn_day(_args: Vec<String>) -> String {
        Local::now().day().to_string()
    }
    
    // 获取当前小时
    pub fn cn_hour(_args: Vec<String>) -> String {
        Local::now().hour().to_string()
    }
    
    // 获取当前分钟
    pub fn cn_minute(_args: Vec<String>) -> String {
        Local::now().minute().to_string()
    }
    
    // 获取当前秒
    pub fn cn_second(_args: Vec<String>) -> String {
        Local::now().second().to_string()
    }
    
    // 计算两个时间戳之间的差值（秒）
    // 参数: timestamp1, timestamp2
    pub fn cn_diff(args: Vec<String>) -> String {
        if args.len() < 2 {
            return "错误: 需要两个时间戳参数".to_string();
        }
        
        let ts1 = match args[0].parse::<i64>() {
            Ok(ts) => ts,
            Err(_) => return "错误: 第一个参数不是有效的时间戳".to_string(),
        };
        
        let ts2 = match args[1].parse::<i64>() {
            Ok(ts) => ts,
            Err(_) => return "错误: 第二个参数不是有效的时间戳".to_string(),
        };
        
        (ts1 - ts2).to_string()
    }
    
    // 添加时间
    // 参数: timestamp, amount, unit (seconds, minutes, hours, days)
    pub fn cn_add(args: Vec<String>) -> String {
        if args.len() < 3 {
            return "错误: 需要三个参数 (时间戳, 数量, 单位)".to_string();
        }
        
        let timestamp = match args[0].parse::<i64>() {
            Ok(ts) => ts,
            Err(_) => return "错误: 第一个参数不是有效的时间戳".to_string(),
        };
        
        let amount = match args[1].parse::<i64>() {
            Ok(a) => a,
            Err(_) => return "错误: 第二个参数不是有效的数字".to_string(),
        };
        
        let dt = match DateTime::from_timestamp(timestamp, 0) {
            Some(dt) => dt,
            None => return "错误: 无法创建日期时间对象".to_string(),
        };
        
        let result = match args[2].as_str() {
            "seconds" => dt + Duration::seconds(amount),
            "minutes" => dt + Duration::minutes(amount),
            "hours" => dt + Duration::hours(amount),
            "days" => dt + Duration::days(amount),
            _ => return "错误: 单位必须是 seconds, minutes, hours 或 days".to_string(),
        };
        
        result.timestamp().to_string()
    }
    
    // 获取当前星期几 (1-7, 周一为1)
    pub fn cn_weekday(_args: Vec<String>) -> String {
        let weekday = Local::now().weekday();
        // chrono中周日是0，但我们返回1-7，周一为1
        let day_num = match weekday.num_days_from_monday() {
            0 => 1, // 周一
            1 => 2, // 周二
            2 => 3, // 周三
            3 => 4, // 周四
            4 => 5, // 周五
            5 => 6, // 周六
            6 => 7, // 周日
            _ => 0, // 不应该发生
        };
        day_num.to_string()
    }
    
    // 延时指定的毫秒数
    // 参数: milliseconds
    pub fn cn_sleep(args: Vec<String>) -> String {
        if args.is_empty() {
            return "错误: 缺少毫秒参数".to_string();
        }
        
        let millis = match args[0].parse::<u64>() {
            Ok(ms) => ms,
            Err(_) => return "错误: 无效的毫秒数".to_string(),
        };
        
        thread::sleep(StdDuration::from_millis(millis));
        "ok".to_string()
    }
    
    // 延时指定的秒数
    // 参数: seconds
    pub fn cn_sleep_seconds(args: Vec<String>) -> String {
        if args.is_empty() {
            return "错误: 缺少秒数参数".to_string();
        }
        
        let seconds = match args[0].parse::<u64>() {
            Ok(s) => s,
            Err(_) => return "错误: 无效的秒数".to_string(),
        };
        
        thread::sleep(StdDuration::from_secs(seconds));
        "ok".to_string()
    }
}

// 初始化函数，返回函数映射
#[no_mangle]
pub extern "C" fn cn_init() -> *mut HashMap<String, LibraryFunction> {
    // 创建库函数注册器
    let mut registry = LibraryRegistry::new();
    
    // 注册std命名空间下的函数
    let std_ns = registry.namespace("std");
    std_ns.add_function("now", std::cn_now)
          .add_function("utc_now", std::cn_utc_now)
          .add_function("now_millis", std::cn_now_millis)
          .add_function("utc_now_millis", std::cn_utc_now_millis)
          .add_function("format_now", std::cn_format_now)
          .add_function("format_utc_now", std::cn_format_utc_now)
          .add_function("format_timestamp", std::cn_format_timestamp)
          .add_function("year", std::cn_year)
          .add_function("month", std::cn_month)
          .add_function("day", std::cn_day)
          .add_function("hour", std::cn_hour)
          .add_function("minute", std::cn_minute)
          .add_function("second", std::cn_second)
          .add_function("diff", std::cn_diff)
          .add_function("add", std::cn_add)
          .add_function("weekday", std::cn_weekday)
          .add_function("sleep", std::cn_sleep)
          .add_function("sleep_seconds", std::cn_sleep_seconds);
    
    // 同时注册为直接函数，不需要命名空间前缀
    registry.add_direct_function("now", std::cn_now)
            .add_direct_function("now_millis", std::cn_now_millis)
            .add_direct_function("format_now", std::cn_format_now)
            .add_direct_function("sleep", std::cn_sleep)
            .add_direct_function("sleep_seconds", std::cn_sleep_seconds);
    
    // 构建并返回库指针
    registry.build_library_pointer()
} 