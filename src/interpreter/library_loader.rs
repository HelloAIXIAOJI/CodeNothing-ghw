use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::env;
use std::fs;
use std::io::Read;
use libloading::{Library, Symbol};
use once_cell::sync::Lazy;
use dashmap::DashMap;
use crate::interpreter::debug_println;
use crate::interpreter::value::Value;

// 🚀 v0.6.0 LLL优化：使用无锁并发HashMap替代全局锁
// DashMap提供了高性能的并发访问，无需全局锁
static LOADED_LIBRARIES: Lazy<DashMap<String, Arc<Library>>> =
    Lazy::new(|| DashMap::new());

// 🚀 函数缓存：避免重复的库函数查找
static FUNCTION_CACHE: Lazy<DashMap<String, Arc<HashMap<String, LibraryFunction>>>> =
    Lazy::new(|| DashMap::new());

// 📊 性能统计（可选，用于监控优化效果）
use std::sync::atomic::{AtomicU64, Ordering};
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static LIBRARY_LOADS: AtomicU64 = AtomicU64::new(0);

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

// 🚀 v0.6.0 LLL优化：无锁库加载函数
pub fn load_library(lib_name: &str) -> Result<Arc<HashMap<String, LibraryFunction>>, String> {
    debug_println(&format!("🚀 无锁加载库: {}", lib_name));

    // 🔥 首先检查函数缓存（最快路径）
    if let Some(functions) = FUNCTION_CACHE.get(lib_name) {
        CACHE_HITS.fetch_add(1, Ordering::Relaxed);
        debug_println(&format!("✅ 函数缓存命中: {} (命中次数: {})", lib_name, CACHE_HITS.load(Ordering::Relaxed)));
        return Ok(functions.clone());
    }

    // 🔥 检查库是否已加载（无锁读取）
    if let Some(lib_entry) = LOADED_LIBRARIES.get(lib_name) {
        debug_println(&format!("✅ 库已加载，提取函数: {}", lib_name));

        // 提取函数映射并缓存
        let functions = extract_library_functions(&lib_entry.value(), lib_name)?;
        FUNCTION_CACHE.insert(lib_name.to_string(), functions.clone());

        return Ok(functions);
    }

    // 🔥 库尚未加载，执行实际加载（这是唯一可能阻塞的地方）
    CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
    LIBRARY_LOADS.fetch_add(1, Ordering::Relaxed);
    debug_println(&format!("🔄 开始实际加载库: {} (缓存未命中: {}, 总加载: {})",
        lib_name,
        CACHE_MISSES.load(Ordering::Relaxed),
        LIBRARY_LOADS.load(Ordering::Relaxed)
    ));

    let lib_path = match find_library_file(lib_name) {
        Some(path) => path,
        None => {
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

        debug_println(&format!("✅ 成功加载库文件: {:?}", lib_path));

        // 提取函数映射
        let functions = extract_library_functions(&lib, lib_name)?;

        // 🚀 无锁插入到缓存中
        LOADED_LIBRARIES.insert(lib_name.to_string(), lib);
        FUNCTION_CACHE.insert(lib_name.to_string(), functions.clone());

        debug_println(&format!("🎯 库 '{}' 加载完成并缓存", lib_name));
        Ok(functions)
    }
}

// 🚀 提取库函数的辅助函数（避免重复代码）
fn extract_library_functions(lib: &Arc<Library>, lib_name: &str) -> Result<Arc<HashMap<String, LibraryFunction>>, String> {
    unsafe {
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
        debug_println(&format!("📋 库 '{}' 中的函数:", lib_name));
        for (func_name, _) in &functions {
            debug_println(&format!("  - {}", func_name));
        }

        Ok(Arc::new(functions))
    }
}

// 🚀 v0.6.0 LLL优化：超高速库函数调用
pub fn call_library_function(lib_name: &str, func_name: &str, args: Vec<String>) -> Result<String, String> {
    debug_println(&format!("🚀 快速调用: {}::{}", lib_name, func_name));

    // 🔥 直接从函数缓存获取（最快路径）
    if let Some(functions) = FUNCTION_CACHE.get(lib_name) {
        if let Some(func) = functions.get(func_name) {
            debug_println(&format!("⚡ 缓存命中，直接调用: {}::{}", lib_name, func_name));
            return Ok(func(args));
        }
    }

    // 🔄 缓存未命中，加载库（这会更新缓存）
    debug_println(&format!("🔄 缓存未命中，加载库: {}", lib_name));
    let functions = load_library(lib_name)?;

    // 查找并调用函数
    match functions.get(func_name) {
        Some(func) => {
            debug_println(&format!("✅ 找到并调用函数: {}::{}", lib_name, func_name));
            Ok(func(args))
        },
        None => Err(format!("库 '{}' 中未找到函数 '{}'", lib_name, func_name)),
    }
}

// 🚀 v0.6.0 新增：性能统计和缓存管理函数

/// 获取库加载性能统计
pub fn get_library_performance_stats() -> (u64, u64, u64, f64) {
    let hits = CACHE_HITS.load(Ordering::Relaxed);
    let misses = CACHE_MISSES.load(Ordering::Relaxed);
    let loads = LIBRARY_LOADS.load(Ordering::Relaxed);
    let hit_rate = if hits + misses > 0 {
        hits as f64 / (hits + misses) as f64 * 100.0
    } else {
        0.0
    };
    (hits, misses, loads, hit_rate)
}

/// 打印库加载性能统计
pub fn print_library_performance_stats() {
    let (hits, misses, loads, hit_rate) = get_library_performance_stats();
    debug_println(&format!("📊 库加载性能统计:"));
    debug_println(&format!("  缓存命中: {}", hits));
    debug_println(&format!("  缓存未命中: {}", misses));
    debug_println(&format!("  库加载次数: {}", loads));
    debug_println(&format!("  缓存命中率: {:.2}%", hit_rate));
    debug_println(&format!("  已缓存库数量: {}", FUNCTION_CACHE.len()));
    debug_println(&format!("  已加载库数量: {}", LOADED_LIBRARIES.len()));
}

/// 清理缓存（用于测试或内存管理）
pub fn clear_library_cache() {
    FUNCTION_CACHE.clear();
    debug_println("🧹 函数缓存已清理");
}

/// 预加载常用库（可选优化）
pub fn preload_common_libraries() -> Result<(), String> {
    let common_libs = ["io", "time", "math"]; // 常用库列表

    debug_println("🚀 开始预加载常用库...");
    for lib_name in &common_libs {
        match load_library(lib_name) {
            Ok(_) => debug_println(&format!("✅ 预加载库成功: {}", lib_name)),
            Err(e) => debug_println(&format!("⚠️ 预加载库失败: {} - {}", lib_name, e)),
        }
    }
    debug_println("🎯 常用库预加载完成");
    Ok(())
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