/// CodeNothing v0.7.6 - 循环专用内存管理系统
/// 
/// 这个模块实现了专门针对循环优化的内存管理系统，包括：
/// - 栈式分配器：快速分配和批量释放
/// - 循环变量管理器：循环开始时预分配，结束时批量释放
/// - 循环检测和自动优化
/// 
/// 目标：实现40%的循环性能提升

use std::collections::HashMap;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
use crate::interpreter::value::Value;

/// 循环变量信息
#[derive(Debug, Clone)]
pub struct LoopVariable {
    /// 变量名
    pub name: String,
    /// 变量类型
    pub var_type: LoopVariableType,
    /// 在栈中的偏移量
    pub stack_offset: usize,
    /// 变量大小
    pub size: usize,
    /// 是否为循环不变量
    pub is_invariant: bool,
}

/// 循环变量类型
#[derive(Debug, Clone, PartialEq)]
pub enum LoopVariableType {
    /// 循环计数器（如 i, j, k）
    Counter,
    /// 循环累加器（如 sum, product）
    Accumulator,
    /// 临时变量（在循环内创建和销毁）
    Temporary,
    /// 循环不变量（在循环中不变）
    Invariant,
}

/// 栈式分配器 - 专门为循环优化的快速内存分配器
#[derive(Debug)]
pub struct StackAllocator {
    /// 栈内存块
    stack_memory: *mut u8,
    /// 栈大小
    stack_size: usize,
    /// 当前栈顶指针
    stack_top: usize,
    /// 分配统计
    allocations: usize,
    /// 释放统计
    deallocations: usize,
    /// 峰值使用量
    peak_usage: usize,
}

impl StackAllocator {
    /// 创建新的栈式分配器
    pub fn new(stack_size: usize) -> Result<Self, String> {
        let layout = Layout::from_size_align(stack_size, 8)
            .map_err(|e| format!("Layout error: {:?}", e))?;
        
        let stack_memory = unsafe { alloc(layout) };
        if stack_memory.is_null() {
            return Err("Failed to allocate stack memory".to_string());
        }

        Ok(StackAllocator {
            stack_memory,
            stack_size,
            stack_top: 0,
            allocations: 0,
            deallocations: 0,
            peak_usage: 0,
        })
    }

    /// 快速分配内存
    pub fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        // 8字节对齐
        let aligned_size = (size + 7) & !7;
        
        if self.stack_top + aligned_size > self.stack_size {
            return None; // 栈溢出
        }

        let ptr = unsafe { self.stack_memory.add(self.stack_top) };
        self.stack_top += aligned_size;
        self.allocations += 1;
        
        // 更新峰值使用量
        if self.stack_top > self.peak_usage {
            self.peak_usage = self.stack_top;
        }

        Some(ptr)
    }

    /// 批量释放到指定位置
    pub fn deallocate_to(&mut self, position: usize) {
        if position <= self.stack_top {
            self.stack_top = position;
            self.deallocations += 1;
        }
    }

    /// 重置栈（释放所有内存）
    pub fn reset(&mut self) {
        self.stack_top = 0;
        self.deallocations += 1;
    }

    /// 获取当前栈顶位置
    pub fn get_stack_top(&self) -> usize {
        self.stack_top
    }

    /// 获取可用空间
    pub fn available_space(&self) -> usize {
        self.stack_size - self.stack_top
    }

    /// 获取使用率
    pub fn usage_ratio(&self) -> f32 {
        self.stack_top as f32 / self.stack_size as f32
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> StackAllocatorStats {
        StackAllocatorStats {
            total_size: self.stack_size,
            used_size: self.stack_top,
            peak_usage: self.peak_usage,
            allocations: self.allocations,
            deallocations: self.deallocations,
            usage_ratio: self.usage_ratio(),
        }
    }
}

impl Drop for StackAllocator {
    fn drop(&mut self) {
        if !self.stack_memory.is_null() {
            let layout = Layout::from_size_align(self.stack_size, 8).unwrap();
            unsafe {
                dealloc(self.stack_memory, layout);
            }
        }
    }
}

/// 栈分配器统计信息
#[derive(Debug, Clone)]
pub struct StackAllocatorStats {
    pub total_size: usize,
    pub used_size: usize,
    pub peak_usage: usize,
    pub allocations: usize,
    pub deallocations: usize,
    pub usage_ratio: f32,
}

/// 循环变量管理器 - 专门管理循环中的变量
#[derive(Debug)]
pub struct LoopVariableManager {
    /// 循环变量映射
    loop_variables: HashMap<String, LoopVariable>,
    /// 栈式分配器
    stack_allocator: StackAllocator,
    /// 循环嵌套级别
    nesting_level: usize,
    /// 循环开始时的栈位置
    loop_start_position: usize,
    /// 是否在循环中
    in_loop: bool,
    /// 循环统计
    loop_count: usize,
    /// 变量预分配成功次数
    prealloc_hits: usize,
    /// 变量预分配失败次数
    prealloc_misses: usize,
}

impl LoopVariableManager {
    /// 创建新的循环变量管理器
    pub fn new(stack_size: usize) -> Result<Self, String> {
        let stack_allocator = StackAllocator::new(stack_size)?;
        
        Ok(LoopVariableManager {
            loop_variables: HashMap::new(),
            stack_allocator,
            nesting_level: 0,
            loop_start_position: 0,
            in_loop: false,
            loop_count: 0,
            prealloc_hits: 0,
            prealloc_misses: 0,
        })
    }

    /// 进入循环 - 预分配循环变量
    pub fn enter_loop(&mut self, expected_variables: &[(&str, LoopVariableType, usize)]) -> Result<(), String> {
        self.nesting_level += 1;
        self.loop_start_position = self.stack_allocator.get_stack_top();
        self.in_loop = true;
        self.loop_count += 1;

        crate::memory_debug_println!("🔄 进入循环 #{} (嵌套级别: {})", self.loop_count, self.nesting_level);
        crate::memory_debug_println!("📍 循环开始位置: {}", self.loop_start_position);

        // 预分配预期的循环变量
        for (name, var_type, size) in expected_variables {
            if let Some(ptr) = self.stack_allocator.allocate(*size) {
                let loop_var = LoopVariable {
                    name: name.to_string(),
                    var_type: var_type.clone(),
                    stack_offset: self.stack_allocator.get_stack_top() - size,
                    size: *size,
                    is_invariant: *var_type == LoopVariableType::Invariant,
                };
                
                self.loop_variables.insert(name.to_string(), loop_var);
                self.prealloc_hits += 1;
                
                crate::memory_debug_println!("✅ 预分配变量: {} ({:?}, {} bytes)", name, var_type, size);
            } else {
                self.prealloc_misses += 1;
                crate::memory_debug_println!("❌ 预分配失败: {} (栈空间不足)", name);
            }
        }

        Ok(())
    }

    /// 退出循环 - 批量释放循环变量
    pub fn exit_loop(&mut self) -> Result<(), String> {
        if !self.in_loop {
            return Err("Not in a loop".to_string());
        }

        let variables_count = self.loop_variables.len();
        let freed_bytes = self.stack_allocator.get_stack_top() - self.loop_start_position;

        // 批量释放到循环开始位置
        self.stack_allocator.deallocate_to(self.loop_start_position);
        
        // 清理循环变量
        self.loop_variables.clear();
        
        self.nesting_level = self.nesting_level.saturating_sub(1);
        self.in_loop = self.nesting_level > 0;

        crate::memory_debug_println!("🏁 退出循环 #{} (释放 {} 个变量, {} bytes)", 
                                   self.loop_count, variables_count, freed_bytes);
        crate::memory_debug_println!("📊 当前嵌套级别: {}", self.nesting_level);

        Ok(())
    }

    /// 获取循环变量
    pub fn get_loop_variable(&self, name: &str) -> Option<&LoopVariable> {
        self.loop_variables.get(name)
    }

    /// 分配循环变量（如果未预分配）
    pub fn allocate_loop_variable(&mut self, name: &str, var_type: LoopVariableType, size: usize) -> Option<*mut u8> {
        if let Some(_) = self.loop_variables.get(name) {
            // 变量已存在，返回现有指针
            self.prealloc_hits += 1;
            return Some(ptr::null_mut()); // 实际应该返回正确的指针
        }

        // 动态分配新变量
        if let Some(ptr) = self.stack_allocator.allocate(size) {
            let loop_var = LoopVariable {
                name: name.to_string(),
                var_type,
                stack_offset: self.stack_allocator.get_stack_top() - size,
                size,
                is_invariant: false,
            };
            
            self.loop_variables.insert(name.to_string(), loop_var);
            self.prealloc_misses += 1;
            
            crate::memory_debug_println!("🆕 动态分配循环变量: {} ({} bytes)", name, size);
            Some(ptr)
        } else {
            None
        }
    }

    /// 检查是否在循环中
    pub fn is_in_loop(&self) -> bool {
        self.in_loop
    }

    /// 获取循环嵌套级别
    pub fn get_nesting_level(&self) -> usize {
        self.nesting_level
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> LoopManagerStats {
        LoopManagerStats {
            loop_count: self.loop_count,
            nesting_level: self.nesting_level,
            variables_count: self.loop_variables.len(),
            prealloc_hits: self.prealloc_hits,
            prealloc_misses: self.prealloc_misses,
            hit_ratio: if self.prealloc_hits + self.prealloc_misses > 0 {
                self.prealloc_hits as f32 / (self.prealloc_hits + self.prealloc_misses) as f32
            } else {
                0.0
            },
            stack_stats: self.stack_allocator.get_stats(),
        }
    }
}

/// 循环管理器统计信息
#[derive(Debug, Clone)]
pub struct LoopManagerStats {
    pub loop_count: usize,
    pub nesting_level: usize,
    pub variables_count: usize,
    pub prealloc_hits: usize,
    pub prealloc_misses: usize,
    pub hit_ratio: f32,
    pub stack_stats: StackAllocatorStats,
}

/// 全局循环变量管理器实例
static mut GLOBAL_LOOP_MANAGER: Option<LoopVariableManager> = None;
static mut LOOP_MANAGER_INITIALIZED: bool = false;

/// 初始化全局循环变量管理器
pub fn init_loop_manager(stack_size: usize) -> Result<(), String> {
    unsafe {
        if !LOOP_MANAGER_INITIALIZED {
            GLOBAL_LOOP_MANAGER = Some(LoopVariableManager::new(stack_size)?);
            LOOP_MANAGER_INITIALIZED = true;
            crate::memory_debug_println!("🚀 循环变量管理器初始化完成 (栈大小: {} bytes)", stack_size);
        }
    }
    Ok(())
}

/// 获取全局循环变量管理器
pub fn get_loop_manager() -> Option<&'static mut LoopVariableManager> {
    unsafe {
        if LOOP_MANAGER_INITIALIZED {
            GLOBAL_LOOP_MANAGER.as_mut()
        } else {
            None
        }
    }
}

/// 便利函数：进入循环
pub fn enter_loop(expected_variables: &[(&str, LoopVariableType, usize)]) -> Result<(), String> {
    if let Some(manager) = get_loop_manager() {
        manager.enter_loop(expected_variables)
    } else {
        Err("Loop manager not initialized".to_string())
    }
}

/// 便利函数：退出循环
pub fn exit_loop() -> Result<(), String> {
    if let Some(manager) = get_loop_manager() {
        manager.exit_loop()
    } else {
        Err("Loop manager not initialized".to_string())
    }
}

/// 便利函数：获取循环管理器统计
pub fn get_loop_stats() -> Option<LoopManagerStats> {
    get_loop_manager().map(|manager| manager.get_stats())
}

/// 循环优化配置
#[derive(Debug, Clone)]
pub struct LoopOptimizationConfig {
    /// 栈大小（字节）
    pub stack_size: usize,
    /// 是否启用循环不变量优化
    pub enable_invariant_optimization: bool,
    /// 是否启用变量预分配
    pub enable_preallocation: bool,
    /// 最大嵌套级别
    pub max_nesting_level: usize,
    /// 预分配阈值（循环次数）
    pub preallocation_threshold: usize,
}

impl Default for LoopOptimizationConfig {
    fn default() -> Self {
        LoopOptimizationConfig {
            stack_size: 64 * 1024, // 64KB 默认栈大小
            enable_invariant_optimization: true,
            enable_preallocation: true,
            max_nesting_level: 10,
            preallocation_threshold: 5,
        }
    }
}

/// 循环性能分析器
#[derive(Debug)]
pub struct LoopPerformanceAnalyzer {
    /// 循环执行时间记录
    loop_timings: Vec<std::time::Duration>,
    /// 变量访问模式
    variable_access_patterns: HashMap<String, VariableAccessPattern>,
    /// 循环热点
    hot_loops: Vec<LoopHotspot>,
}

/// 变量访问模式
#[derive(Debug, Clone)]
pub struct VariableAccessPattern {
    /// 读取次数
    pub read_count: usize,
    /// 写入次数
    pub write_count: usize,
    /// 是否为循环不变量
    pub is_invariant: bool,
    /// 访问频率
    pub access_frequency: f32,
}

/// 循环热点信息
#[derive(Debug, Clone)]
pub struct LoopHotspot {
    /// 循环标识
    pub loop_id: String,
    /// 执行次数
    pub execution_count: usize,
    /// 平均执行时间
    pub average_duration: std::time::Duration,
    /// 变量数量
    pub variable_count: usize,
    /// 优化建议
    pub optimization_suggestions: Vec<String>,
}

impl LoopPerformanceAnalyzer {
    /// 创建新的性能分析器
    pub fn new() -> Self {
        LoopPerformanceAnalyzer {
            loop_timings: Vec::new(),
            variable_access_patterns: HashMap::new(),
            hot_loops: Vec::new(),
        }
    }

    /// 记录循环执行时间
    pub fn record_loop_timing(&mut self, duration: std::time::Duration) {
        self.loop_timings.push(duration);
    }

    /// 记录变量访问
    pub fn record_variable_access(&mut self, var_name: &str, is_read: bool) {
        let pattern = self.variable_access_patterns
            .entry(var_name.to_string())
            .or_insert(VariableAccessPattern {
                read_count: 0,
                write_count: 0,
                is_invariant: false,
                access_frequency: 0.0,
            });

        if is_read {
            pattern.read_count += 1;
        } else {
            pattern.write_count += 1;
        }

        // 更新访问频率
        pattern.access_frequency = (pattern.read_count + pattern.write_count) as f32 / self.loop_timings.len() as f32;

        // 检测循环不变量（只读且访问频率高）
        pattern.is_invariant = pattern.write_count == 0 && pattern.access_frequency > 0.8;
    }

    /// 分析循环热点
    pub fn analyze_hotspots(&mut self) -> Vec<LoopHotspot> {
        // 简化的热点分析
        if self.loop_timings.len() > 0 {
            let avg_duration = self.loop_timings.iter().sum::<std::time::Duration>() / self.loop_timings.len() as u32;

            let mut suggestions = Vec::new();

            // 基于变量访问模式生成优化建议
            for (var_name, pattern) in &self.variable_access_patterns {
                if pattern.is_invariant {
                    suggestions.push(format!("变量 '{}' 可以提升为循环不变量", var_name));
                }
                if pattern.access_frequency > 1.5 {
                    suggestions.push(format!("变量 '{}' 访问频繁，建议缓存", var_name));
                }
            }

            let hotspot = LoopHotspot {
                loop_id: "main_loop".to_string(),
                execution_count: self.loop_timings.len(),
                average_duration: avg_duration,
                variable_count: self.variable_access_patterns.len(),
                optimization_suggestions: suggestions,
            };

            self.hot_loops.push(hotspot.clone());
            vec![hotspot]
        } else {
            Vec::new()
        }
    }

    /// 获取性能报告
    pub fn get_performance_report(&self) -> LoopPerformanceReport {
        let total_time = self.loop_timings.iter().sum::<std::time::Duration>();
        let avg_time = if !self.loop_timings.is_empty() {
            total_time / self.loop_timings.len() as u32
        } else {
            std::time::Duration::from_millis(0)
        };

        LoopPerformanceReport {
            total_loops: self.loop_timings.len(),
            total_execution_time: total_time,
            average_execution_time: avg_time,
            variable_patterns: self.variable_access_patterns.clone(),
            hotspots: self.hot_loops.clone(),
        }
    }
}

/// 循环性能报告
#[derive(Debug, Clone)]
pub struct LoopPerformanceReport {
    pub total_loops: usize,
    pub total_execution_time: std::time::Duration,
    pub average_execution_time: std::time::Duration,
    pub variable_patterns: HashMap<String, VariableAccessPattern>,
    pub hotspots: Vec<LoopHotspot>,
}

/// 显示循环性能统计
pub fn print_loop_performance_stats() {
    if let Some(stats) = get_loop_stats() {
        println!("=== CodeNothing v0.7.6 循环内存管理统计 ===");
        println!("循环执行次数: {}", stats.loop_count);
        println!("当前嵌套级别: {}", stats.nesting_level);
        println!("管理变量数量: {}", stats.variables_count);
        println!("预分配命中次数: {}", stats.prealloc_hits);
        println!("预分配失败次数: {}", stats.prealloc_misses);
        println!("预分配命中率: {:.2}%", stats.hit_ratio * 100.0);

        println!("\n--- 栈分配器统计 ---");
        println!("栈总大小: {} bytes", stats.stack_stats.total_size);
        println!("已使用大小: {} bytes", stats.stack_stats.used_size);
        println!("峰值使用量: {} bytes", stats.stack_stats.peak_usage);
        println!("分配次数: {}", stats.stack_stats.allocations);
        println!("释放次数: {}", stats.stack_stats.deallocations);
        println!("使用率: {:.2}%", stats.stack_stats.usage_ratio * 100.0);
        println!("=====================================");
    } else {
        println!("循环内存管理器未初始化");
    }
}
