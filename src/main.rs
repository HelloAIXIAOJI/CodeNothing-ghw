use std::fs;

mod parser;
mod ast;
mod interpreter;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("用法: {} <文件路径> [--cn-parser]", args[0]);
        return;
    }
    
    let file_path = &args[1];
    let debug_parser = args.iter().any(|arg| arg == "--cn-parser");
    
    match fs::read_to_string(file_path) {
        Ok(content) => {
            let ast = parser::parse(&content, debug_parser);
            match ast {
                Ok(program) => {
                    let result = interpreter::interpret(&program);
                    println!("程序执行结果: {}", result);
                },
                Err(err) => println!("解析错误: {}", err),
            }
        },
        Err(err) => println!("读取文件错误: {}", err),
    }
} 