// CodeNothing v0.7.4 调试配置模块
// 提供细粒度的调试输出控制

use std::sync::atomic::{AtomicBool, Ordering};

/// 全局调试配置
pub struct DebugConfig {
    /// 是否启用JIT编译调试输出
    pub jit_debug: AtomicBool,
    /// 是否启用生命周期分析调试输出
    pub lifetime_debug: AtomicBool,
    /// 是否启用表达式求值调试输出
    pub expression_debug: AtomicBool,
    /// 是否启用函数调用调试输出
    pub function_debug: AtomicBool,
    /// 是否启用变量访问调试输出
    pub variable_debug: AtomicBool,
    /// 是否启用所有调试输出
    pub all_debug: AtomicBool,
}

impl DebugConfig {
    /// 创建默认配置（所有调试输出关闭）
    pub fn new() -> Self {
        Self {
            jit_debug: AtomicBool::new(false),
            lifetime_debug: AtomicBool::new(false),
            expression_debug: AtomicBool::new(false),
            function_debug: AtomicBool::new(false),
            variable_debug: AtomicBool::new(false),
            all_debug: AtomicBool::new(false),
        }
    }

    /// 启用JIT调试输出
    pub fn enable_jit_debug(&self) {
        self.jit_debug.store(true, Ordering::Relaxed);
    }

    /// 禁用JIT调试输出
    pub fn disable_jit_debug(&self) {
        self.jit_debug.store(false, Ordering::Relaxed);
    }

    /// 检查是否启用JIT调试输出
    pub fn is_jit_debug_enabled(&self) -> bool {
        self.jit_debug.load(Ordering::Relaxed) || self.all_debug.load(Ordering::Relaxed)
    }

    /// 启用生命周期分析调试输出
    pub fn enable_lifetime_debug(&self) {
        self.lifetime_debug.store(true, Ordering::Relaxed);
    }

    /// 禁用生命周期分析调试输出
    pub fn disable_lifetime_debug(&self) {
        self.lifetime_debug.store(false, Ordering::Relaxed);
    }

    /// 检查是否启用生命周期分析调试输出
    pub fn is_lifetime_debug_enabled(&self) -> bool {
        self.lifetime_debug.load(Ordering::Relaxed) || self.all_debug.load(Ordering::Relaxed)
    }

    /// 启用表达式求值调试输出
    pub fn enable_expression_debug(&self) {
        self.expression_debug.store(true, Ordering::Relaxed);
    }

    /// 禁用表达式求值调试输出
    pub fn disable_expression_debug(&self) {
        self.expression_debug.store(false, Ordering::Relaxed);
    }

    /// 检查是否启用表达式求值调试输出
    pub fn is_expression_debug_enabled(&self) -> bool {
        self.expression_debug.load(Ordering::Relaxed) || self.all_debug.load(Ordering::Relaxed)
    }

    /// 启用函数调用调试输出
    pub fn enable_function_debug(&self) {
        self.function_debug.store(true, Ordering::Relaxed);
    }

    /// 禁用函数调用调试输出
    pub fn disable_function_debug(&self) {
        self.function_debug.store(false, Ordering::Relaxed);
    }

    /// 检查是否启用函数调用调试输出
    pub fn is_function_debug_enabled(&self) -> bool {
        self.function_debug.load(Ordering::Relaxed) || self.all_debug.load(Ordering::Relaxed)
    }

    /// 启用变量访问调试输出
    pub fn enable_variable_debug(&self) {
        self.variable_debug.store(true, Ordering::Relaxed);
    }

    /// 禁用变量访问调试输出
    pub fn disable_variable_debug(&self) {
        self.variable_debug.store(false, Ordering::Relaxed);
    }

    /// 检查是否启用变量访问调试输出
    pub fn is_variable_debug_enabled(&self) -> bool {
        self.variable_debug.load(Ordering::Relaxed) || self.all_debug.load(Ordering::Relaxed)
    }

    /// 启用所有调试输出
    pub fn enable_all_debug(&self) {
        self.all_debug.store(true, Ordering::Relaxed);
    }

    /// 禁用所有调试输出
    pub fn disable_all_debug(&self) {
        self.all_debug.store(false, Ordering::Relaxed);
        self.jit_debug.store(false, Ordering::Relaxed);
        self.lifetime_debug.store(false, Ordering::Relaxed);
        self.expression_debug.store(false, Ordering::Relaxed);
        self.function_debug.store(false, Ordering::Relaxed);
        self.variable_debug.store(false, Ordering::Relaxed);
    }

    /// 从命令行参数解析调试配置
    pub fn from_args(&self, args: &[String]) {
        for arg in args {
            match arg.as_str() {
                "--debug-jit" => self.enable_jit_debug(),
                "--debug-lifetime" => self.enable_lifetime_debug(),
                "--debug-expression" => self.enable_expression_debug(),
                "--debug-function" => self.enable_function_debug(),
                "--debug-variable" => self.enable_variable_debug(),
                "--debug-all" => self.enable_all_debug(),
                "--no-debug" => self.disable_all_debug(),
                _ => {}
            }
        }
    }

    /// 打印调试配置状态
    pub fn print_status(&self) {
        println!("=== CodeNothing v0.7.4 调试配置状态 ===");
        println!("JIT编译调试: {}", if self.is_jit_debug_enabled() { "启用" } else { "禁用" });
        println!("生命周期分析调试: {}", if self.is_lifetime_debug_enabled() { "启用" } else { "禁用" });
        println!("表达式求值调试: {}", if self.is_expression_debug_enabled() { "启用" } else { "禁用" });
        println!("函数调用调试: {}", if self.is_function_debug_enabled() { "启用" } else { "禁用" });
        println!("变量访问调试: {}", if self.is_variable_debug_enabled() { "启用" } else { "禁用" });
        println!("所有调试: {}", if self.all_debug.load(Ordering::Relaxed) { "启用" } else { "禁用" });
        println!("=====================================");
    }
}

/// 全局调试配置实例
static DEBUG_CONFIG: std::sync::OnceLock<DebugConfig> = std::sync::OnceLock::new();

/// 获取全局调试配置
pub fn get_debug_config() -> &'static DebugConfig {
    DEBUG_CONFIG.get_or_init(|| DebugConfig::new())
}

/// 初始化调试配置
pub fn init_debug_config(args: &[String]) {
    let config = get_debug_config();
    config.from_args(args);
}

/// JIT调试输出宏
#[macro_export]
macro_rules! jit_debug_println {
    ($($arg:tt)*) => {
        if $crate::debug_config::get_debug_config().is_jit_debug_enabled() {
            println!($($arg)*);
        }
    };
}

/// 生命周期调试输出宏
#[macro_export]
macro_rules! lifetime_debug_println {
    ($($arg:tt)*) => {
        if $crate::debug_config::get_debug_config().is_lifetime_debug_enabled() {
            println!($($arg)*);
        }
    };
}

/// 表达式调试输出宏
#[macro_export]
macro_rules! expression_debug_println {
    ($($arg:tt)*) => {
        if $crate::debug_config::get_debug_config().is_expression_debug_enabled() {
            println!($($arg)*);
        }
    };
}

/// 函数调试输出宏
#[macro_export]
macro_rules! function_debug_println {
    ($($arg:tt)*) => {
        if $crate::debug_config::get_debug_config().is_function_debug_enabled() {
            println!($($arg)*);
        }
    };
}

/// 变量调试输出宏
#[macro_export]
macro_rules! variable_debug_println {
    ($($arg:tt)*) => {
        if $crate::debug_config::get_debug_config().is_variable_debug_enabled() {
            println!($($arg)*);
        }
    };
}

/// 打印调试帮助信息
pub fn print_debug_help() {
    println!("CodeNothing v0.7.4 调试选项:");
    println!("  --debug-jit        启用JIT编译调试输出");
    println!("  --debug-lifetime   启用生命周期分析调试输出");
    println!("  --debug-expression 启用表达式求值调试输出");
    println!("  --debug-function   启用函数调用调试输出");
    println!("  --debug-variable   启用变量访问调试输出");
    println!("  --debug-all        启用所有调试输出");
    println!("  --no-debug         禁用所有调试输出");
    println!();
    println!("示例:");
    println!("  ./CodeNothing program.cn --debug-jit");
    println!("  ./CodeNothing program.cn --debug-lifetime --debug-jit");
    println!("  ./CodeNothing program.cn --debug-all");
}
