use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::env;
use libloading::{Library, Symbol};
use once_cell::sync::Lazy;

// 已加载库的缓存，使用Lazy静态变量确保线程安全的初始化
static LOADED_LIBRARIES: Lazy<Arc<Mutex<HashMap<String, Arc<Library>>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// 库函数类型定义
pub type LibraryFunction = fn(Vec<String>) -> String;

// 库初始化函数类型
type InitFn = unsafe fn() -> *mut HashMap<String, LibraryFunction>;

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
    
    // 根据操作系统添加不同的扩展名
    #[cfg(target_os = "windows")]
    {
        path.push(format!("{}.dll", lib_name));
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.push(format!("lib{}.so", lib_name));
    }
    
    path
}

// 加载库并返回其函数映射
pub fn load_library(lib_name: &str) -> Result<Arc<HashMap<String, LibraryFunction>>, String> {
    let mut loaded_libs = match LOADED_LIBRARIES.lock() {
        Ok(guard) => guard,
        Err(_) => return Err("无法获取库缓存锁".to_string()),
    };
    
    // 检查库是否已经加载
    if let Some(lib) = loaded_libs.get(lib_name) {
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
            
            // 将原始指针转换为Arc<HashMap>
            let functions = Arc::new(Box::from_raw(functions_ptr).clone());
            return Ok(functions);
        }
    }
    
    // 库尚未加载，尝试加载
    let lib_path = get_library_path(lib_name);
    
    if !lib_path.exists() {
        return Err(format!("找不到库文件: {:?}", lib_path));
    }
    
    unsafe {
        // 加载库
        let lib = match Library::new(&lib_path) {
            Ok(l) => Arc::new(l),
            Err(e) => return Err(format!("无法加载库: {}", e)),
        };
        
        // 获取初始化函数
        let init_fn: Symbol<InitFn> = match lib.get(b"cn_init") {
            Ok(f) => f,
            Err(e) => return Err(format!("无法获取库初始化函数: {}", e)),
        };
        
        // 调用初始化函数获取函数映射
        let functions_ptr = init_fn();
        if functions_ptr.is_null() {
            return Err("库初始化函数返回空指针".to_string());
        }
        
        // 将原始指针转换为Arc<HashMap>
        let functions = Arc::new(Box::from_raw(functions_ptr).clone());
        
        // 将库添加到已加载库缓存
        loaded_libs.insert(lib_name.to_string(), lib);
        
        Ok(functions)
    }
}

// 调用库函数
pub fn call_library_function(lib_name: &str, func_name: &str, args: Vec<String>) -> Result<String, String> {
    // 加载库
    let functions = load_library(lib_name)?;
    
    // 查找函数
    match functions.get(func_name) {
        Some(func) => {
            // 调用函数
            Ok(func(args))
        },
        None => Err(format!("库 '{}' 中未找到函数 '{}'", lib_name, func_name)),
    }
} 