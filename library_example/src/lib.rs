use std::collections::HashMap;
use std::io::{self, Write};
use std::ffi::CString;

// 定义库函数类型
type LibraryFunction = fn(Vec<String>) -> String;

// 打印字符串到标准输出
fn cn_print(args: Vec<String>) -> String {
    let mut output = String::new();
    for arg in args {
        print!("{}", arg);
        output.push_str(&arg);
    }
    io::stdout().flush().unwrap();
    output
}

// 打印字符串到标准输出，并添加换行符
fn cn_println(args: Vec<String>) -> String {
    let mut output = String::new();
    for arg in args {
        println!("{}", arg);
        output.push_str(&arg);
    }
    output.push('\n');
    output
}

// 从标准输入读取一行
fn cn_read_line(_args: Vec<String>) -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            // 移除末尾的换行符
            if input.ends_with('\n') {
                input.pop();
                if input.ends_with('\r') {
                    input.pop();
                }
            }
            input
        },
        Err(_) => String::new(),
    }
}

// 初始化函数，返回函数映射
#[no_mangle]
pub extern "C" fn cn_init() -> *mut HashMap<String, LibraryFunction> {
    let mut functions = HashMap::new();
    functions.insert("print".to_string(), cn_print as LibraryFunction);
    functions.insert("println".to_string(), cn_println as LibraryFunction);
    functions.insert("read_line".to_string(), cn_read_line as LibraryFunction);
    
    // 将HashMap装箱并转换为原始指针
    Box::into_raw(Box::new(functions))
} 