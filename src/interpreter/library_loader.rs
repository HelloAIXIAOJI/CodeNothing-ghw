use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::env;
use std::fs;
use std::io::Read;
use libloading::{Library, Symbol};
use once_cell::sync::Lazy;
use crate::interpreter::debug_println;
use crate::interpreter::value::Value;

// 已加载库的缓存，使用Lazy静态变量确保线程安全的初始化
static LOADED_LIBRARIES: Lazy<Arc<Mutex<HashMap<String, Arc<Library>>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// 库函数类型定义
pub type LibraryFunction = fn(Vec<String>) -> String;

// 库初始化函数类型
type InitFn = unsafe fn() -> *mut HashMap<String, LibraryFunction>;

// 获取平台特定的库文件扩展名（CodeNothing规范：无lib前缀）
fn get_library_filename(lib_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{}.dll", lib_name)
    }
    #[cfg(target_os = "macos")]
    {
        format!("{}.dylib", lib_name)
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        format!("{}.so", lib_name)
    }
}

// 获取所有可能的库文件名（CodeNothing规范）
fn get_possible_library_filenames(lib_name: &str) -> Vec<String> {
    vec![
        // 当前平台的标准格式
        get_library_filename(lib_name),
        // 其他平台格式（跨平台兼容）
        format!("{}.dll", lib_name),
        format!("{}.dylib", lib_name),
        format!("{}.so", lib_name),
    ]
}

// 获取库路径
fn get_library_path(lib_name: &str) -> PathBuf {
    let mut path = match env::current_exe() {
        Ok(exe_path) => {
            // 获取可执行文件所在目录
            match exe_path.parent() {
                Some(parent) => parent.to_path_buf(),
                None => PathBuf::from("."),
            }
        },
        Err(_) => PathBuf::from("."),
    };

    // 添加library子目录
    path.push("library");

    // 使用平台特定的库文件名
    path.push(get_library_filename(lib_name));

    debug_println(&format!("尝试加载库文件: {:?}", path));
    path
}

// 查找库文件（CodeNothing规范：只查找两个目录）
fn find_library_file(lib_name: &str) -> Option<PathBuf> {
    let search_paths = vec![
        // 1. 解释器目录/library
        {
            let mut path = match env::current_exe() {
                Ok(exe_path) => {
                    match exe_path.parent() {
                        Some(parent) => parent.to_path_buf(),
                        None => PathBuf::from("."),
                    }
                },
                Err(_) => PathBuf::from("."),
            };
            path.push("library");
            path
        },
        // 2. 当前目录/library
        {
            let mut path = PathBuf::from(".");
            path.push("library");
            path
        },
    ];

    let possible_filenames = get_possible_library_filenames(lib_name);

    for search_path in search_paths {
        for filename in &possible_filenames {
            let mut full_path = search_path.clone();
            full_path.push(filename);

            debug_println(&format!("检查库文件: {:?}", full_path));

            if full_path.exists() {
                debug_println(&format!("找到库文件: {:?}", full_path));
                return Some(full_path);
            }
        }
    }

    None
}

// 获取库支持的命名空间
pub fn get_library_namespaces(lib_name: &str) -> Result<Vec<String>, String> {
    // 加载库函数
    let functions = load_library(lib_name)?;
    let mut namespaces = Vec::new();
    
    // 从函数名中提取命名空间
    for func_name in functions.keys() {
        if func_name.contains("::") {
            let parts: Vec<&str> = func_name.split("::").collect();
            if parts.len() >= 2 {
                let ns_name = parts[0].to_string();
                if !namespaces.contains(&ns_name) {
                    debug_println(&format!("从函数名 '{}' 中检测到命名空间: {}", func_name, ns_name));
                    namespaces.push(ns_name);
                }
            }
        }
    }
    
    if namespaces.is_empty() {
        debug_println(&format!("库 '{}' 中未检测到命名空间", lib_name));
    } else {
        debug_println(&format!("库 '{}' 支持的命名空间: {:?}", lib_name, namespaces));
    }
    
    Ok(namespaces)
}

// 添加一个函数来打印库中的所有函数
pub fn debug_library_functions(lib_name: &str) -> Result<(), String> {
    let functions = load_library(lib_name)?;
    
    debug_println(&format!("库 '{}' 中的所有函数:", lib_name));
    for (func_name, _) in functions.iter() {
        debug_println(&format!("  - {}", func_name));
    }
    
    Ok(())
}

// 加载库并返回其函数映射
pub fn load_library(lib_name: &str) -> Result<Arc<HashMap<String, LibraryFunction>>, String> {
    debug_println(&format!("开始加载库: {}", lib_name));
    
    let mut loaded_libs = match LOADED_LIBRARIES.lock() {
        Ok(guard) => guard,
        Err(_) => return Err("无法获取库缓存锁".to_string()),
    };
    
    // 检查库是否已经加载
    if let Some(lib) = loaded_libs.get(lib_name) {
        debug_println(&format!("库 '{}' 已加载，获取其函数映射", lib_name));
        // 库已加载，获取其函数映射
        unsafe {
            let init_fn: Symbol<InitFn> = match lib.get(b"cn_init") {
                Ok(f) => f,
                Err(e) => return Err(format!("无法获取库初始化函数: {}", e)),
            };
            
            let functions_ptr = init_fn();
            if functions_ptr.is_null() {
                return Err("库初始化函数返回空指针".to_string());
            }
            
            // 将原始指针转换为HashMap，然后包装为Arc
            let boxed_functions = Box::from_raw(functions_ptr);
            let functions = *boxed_functions; // 解引用Box<HashMap>为HashMap
            
            // 调试输出函数列表
            debug_println(&format!("库 '{}' 中的函数:", lib_name));
            for (func_name, _) in &functions {
                debug_println(&format!("  - {}", func_name));
            }
            
            return Ok(Arc::new(functions));
        }
    }
    
    // 库尚未加载，尝试查找并加载
    let lib_path = match find_library_file(lib_name) {
        Some(path) => path,
        None => {
            let primary_path = get_library_path(lib_name);
            return Err(format!(
                "找不到库文件 '{}'\n查找位置:\n- 解释器目录/library/\n- 当前目录/library/\n支持的文件格式: {}",
                lib_name,
                get_possible_library_filenames(lib_name).join(", ")
            ));
        }
    };

    unsafe {
        // 加载库
        let lib = match Library::new(&lib_path) {
            Ok(l) => Arc::new(l),
            Err(e) => return Err(format!("无法加载库 '{:?}': {}", lib_path, e)),
        };

        debug_println(&format!("成功加载库文件: {:?}", lib_path));

        // 获取初始化函数
        let init_fn: Symbol<InitFn> = match lib.get(b"cn_init") {
            Ok(f) => f,
            Err(e) => return Err(format!("无法获取库初始化函数 'cn_init': {}", e)),
        };

        // 调用初始化函数获取函数映射
        let functions_ptr = init_fn();
        if functions_ptr.is_null() {
            return Err("库初始化函数返回空指针".to_string());
        }

        // 将原始指针转换为HashMap，然后包装为Arc
        let boxed_functions = Box::from_raw(functions_ptr);
        let functions = *boxed_functions; // 解引用Box<HashMap>为HashMap

        // 调试输出函数列表
        debug_println(&format!("库 '{}' 中的函数:", lib_name));
        for (func_name, _) in &functions {
            debug_println(&format!("  - {}", func_name));
        }

        let functions_arc = Arc::new(functions);

        // 将库添加到已加载库缓存
        loaded_libs.insert(lib_name.to_string(), lib);

        Ok(functions_arc)
    }
}

// 调用库函数
pub fn call_library_function(lib_name: &str, func_name: &str, args: Vec<String>) -> Result<String, String> {
    debug_println(&format!("调用库函数: {}::{}", lib_name, func_name));
    
    // 加载库
    let functions = load_library(lib_name)?;
    
    // 查找函数
    match functions.get(func_name) {
        Some(func) => {
            // 调用函数
            debug_println(&format!("找到并调用函数: {}::{}", lib_name, func_name));
            Ok(func(args))
        },
        None => Err(format!("库 '{}' 中未找到函数 '{}'", lib_name, func_name)),
    }
} 

// 新增函数，将Value类型转换为字符串参数
pub fn convert_value_to_string_arg(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Long(l) => l.to_string(),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter()
                .map(|v| convert_value_to_string_arg(v))
                .collect();
            format!("[{}]", elements.join(", "))
        },
        Value::Map(map) => {
            let entries: Vec<String> = map.iter()
                .map(|(k, v)| format!("{}:{}", k, convert_value_to_string_arg(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        },
        Value::Object(obj) => {
            format!("{}@{:p}", obj.class_name, obj)
        },
        Value::Lambda(params, _) => {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            format!("lambda({})", param_names.join(", "))
        },
        Value::LambdaBlock(params, _) => {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            format!("lambda_block({})", param_names.join(", "))
        },
        Value::FunctionReference(name) => {
            format!("function_ref({})", name)
        },
        Value::EnumValue(enum_val) => {
            if enum_val.fields.is_empty() {
                format!("{}::{}", enum_val.enum_name, enum_val.variant_name)
            } else {
                let field_strs: Vec<String> = enum_val.fields.iter().map(|f| convert_value_to_string_arg(f)).collect();
                format!("{}::{}({})", enum_val.enum_name, enum_val.variant_name, field_strs.join(", "))
            }
        },
        Value::Pointer(ptr) => {
            if ptr.is_null {
                "null".to_string()
            } else {
                format!("*{:p}", ptr.address as *const usize)
            }
        },
        Value::FunctionPointer(func_ptr) => {
            if func_ptr.is_null {
                "null".to_string()
            } else if func_ptr.is_lambda {
                "*fn(lambda)".to_string()
            } else {
                format!("*fn({})", func_ptr.function_name)
            }
        },
        Value::LambdaFunctionPointer(lambda_ptr) => {
            if lambda_ptr.is_null {
                "null".to_string()
            } else {
                "*fn(lambda)".to_string()
            }
        },
        Value::ArrayPointer(array_ptr) => {
            if array_ptr.is_null {
                "null".to_string()
            } else {
                format!("*[{}]@0x{:x}", array_ptr.array_size, array_ptr.address)
            }
        },
        Value::PointerArray(ptr_array) => {
            format!("[{}]*ptr", ptr_array.array_size)
        },
        Value::None => "null".to_string(),
    }
}

// 从Vector<Value>转换为Vector<String>，用于库函数调用
pub fn convert_values_to_string_args(values: &[Value]) -> Vec<String> {
    values.iter().map(|v| convert_value_to_string_arg(v)).collect()
} 