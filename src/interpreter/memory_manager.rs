use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};
use std::cell::RefCell;
use super::value::Value;

// 🚀 v0.6.2 读写锁性能监控（条件编译）
#[cfg(feature = "rwlock-stats")]
static READ_OPERATIONS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "rwlock-stats")]
static WRITE_OPERATIONS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "rwlock-stats")]
static READ_LOCK_TIME: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "rwlock-stats")]
static WRITE_LOCK_TIME: AtomicU64 = AtomicU64::new(0);

// 🚀 v0.6.2 性能监控宏（零开销抽象）
#[cfg(feature = "rwlock-stats")]
macro_rules! track_read_operation {
    ($start_time:expr) => {
        let lock_time = $start_time.elapsed().unwrap().as_nanos() as u64;
        READ_LOCK_TIME.fetch_add(lock_time, Ordering::Relaxed);
        READ_OPERATIONS.fetch_add(1, Ordering::Relaxed);
    };
}

#[cfg(not(feature = "rwlock-stats"))]
macro_rules! track_read_operation {
    ($start_time:expr) => {};
}

#[cfg(feature = "rwlock-stats")]
macro_rules! track_write_operation {
    ($start_time:expr) => {
        let lock_time = $start_time.elapsed().unwrap().as_nanos() as u64;
        WRITE_LOCK_TIME.fetch_add(lock_time, Ordering::Relaxed);
        WRITE_OPERATIONS.fetch_add(1, Ordering::Relaxed);
    };
}

#[cfg(not(feature = "rwlock-stats"))]
macro_rules! track_write_operation {
    ($start_time:expr) => {};
}

/// 内存块信息
#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub address: usize,
    pub size: usize,
    pub value: Value,
    pub is_allocated: bool,
    pub ref_count: usize,
    pub allocation_time: u64, // 分配时间戳
    pub last_access_time: u64, // 最后访问时间
}

/// 指针标记信息，用于跟踪指针生命周期
#[derive(Debug, Clone)]
pub struct PointerTag {
    pub tag_id: u64,
    pub address: usize,
    pub is_valid: bool,
    pub creation_time: u64,
}

/// 内存管理器
#[derive(Debug)]
pub struct MemoryManager {
    memory_blocks: HashMap<usize, MemoryBlock>,
    next_address: usize,
    quarantine_addresses: Vec<(usize, u64)>, // 隔离区：(地址, 释放时间)
    total_allocated: usize,
    max_memory: usize,
    pointer_tags: HashMap<u64, PointerTag>, // 指针标记映射
    next_tag_id: u64,
    quarantine_time_ms: u64, // 隔离时间（毫秒）
    valid_address_ranges: Vec<(usize, usize)>, // 有效地址范围
}

impl MemoryManager {
    pub fn new() -> Self {
        let mut manager = Self {
            memory_blocks: HashMap::new(),
            next_address: 0x1000, // 从较高地址开始，避免与系统地址冲突
            quarantine_addresses: Vec::new(),
            total_allocated: 0,
            max_memory: 1024 * 1024 * 100, // 100MB 限制
            pointer_tags: HashMap::new(),
            next_tag_id: 1,
            quarantine_time_ms: 5000, // 5秒隔离时间
            valid_address_ranges: Vec::new(),
        };

        // 初始化有效地址范围
        manager.valid_address_ranges.push((0x1000, 0x1000 + 1024 * 1024 * 100));
        manager
    }

    /// 获取当前时间戳（毫秒）
    fn current_time_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// 检查地址是否在有效范围内
    fn is_address_in_valid_range(&self, address: usize) -> bool {
        self.valid_address_ranges.iter().any(|(start, end)| {
            address >= *start && address < *end
        })
    }

    /// 分配内存并返回地址和指针标记
    pub fn allocate(&mut self, value: Value) -> Result<(usize, u64), String> {
        let size = self.calculate_size(&value);

        // 检查内存限制
        if self.total_allocated + size > self.max_memory {
            return Err("内存不足".to_string());
        }

        // 清理隔离区中过期的地址
        self.cleanup_quarantine();

        // 分配新地址（不重用，避免悬空指针问题）
        let address = self.next_address;
        self.next_address += size.max(8); // 至少8字节对齐

        // 检查地址是否在有效范围内
        if !self.is_address_in_valid_range(address) {
            return Err("地址超出有效范围".to_string());
        }

        let current_time = Self::current_time_ms();
        let block = MemoryBlock {
            address,
            size,
            value,
            is_allocated: true,
            ref_count: 1,
            allocation_time: current_time,
            last_access_time: current_time,
        };

        // 创建指针标记
        let tag_id = self.next_tag_id;
        self.next_tag_id += 1;

        let tag = PointerTag {
            tag_id,
            address,
            is_valid: true,
            creation_time: current_time,
        };

        self.memory_blocks.insert(address, block);
        self.pointer_tags.insert(tag_id, tag);
        self.total_allocated += size;

        Ok((address, tag_id))
    }

    /// 清理隔离区中过期的地址
    fn cleanup_quarantine(&mut self) {
        let current_time = Self::current_time_ms();
        self.quarantine_addresses.retain(|(_, release_time)| {
            current_time - release_time < self.quarantine_time_ms
        });
    }

    /// 释放内存（使用隔离机制）
    pub fn deallocate(&mut self, address: usize) -> Result<(), String> {
        if let Some(block) = self.memory_blocks.get_mut(&address) {
            if !block.is_allocated {
                return Err("尝试释放已释放的内存".to_string());
            }

            // 标记为已释放
            block.is_allocated = false;
            self.total_allocated -= block.size;

            // 将地址放入隔离区而不是立即重用
            let current_time = Self::current_time_ms();
            self.quarantine_addresses.push((address, current_time));

            // 使所有指向此地址的标记失效
            self.invalidate_pointer_tags_for_address(address);

            Ok(())
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 使指向特定地址的所有指针标记失效
    fn invalidate_pointer_tags_for_address(&mut self, address: usize) {
        for tag in self.pointer_tags.values_mut() {
            if tag.address == address {
                tag.is_valid = false;
            }
        }
    }

    /// 读取内存中的值（带指针标记验证）
    pub fn read(&mut self, address: usize, tag_id: Option<u64>) -> Result<Value, String> {
        // 验证指针标记
        if let Some(tag_id) = tag_id {
            if let Some(tag) = self.pointer_tags.get(&tag_id) {
                if !tag.is_valid || tag.address != address {
                    return Err("指针标记无效或地址不匹配".to_string());
                }
            } else {
                return Err("指针标记不存在".to_string());
            }
        }

        if let Some(block) = self.memory_blocks.get_mut(&address) {
            if !block.is_allocated {
                return Err("尝试访问已释放的内存".to_string());
            }

            // 更新最后访问时间
            block.last_access_time = Self::current_time_ms();
            Ok(block.value.clone())
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 🚀 v0.6.2 只读内存访问（不更新访问时间，支持并发读取）
    pub fn read_only(&self, address: usize, tag_id: Option<u64>) -> Result<Value, String> {
        // 验证指针标记
        if let Some(tag_id) = tag_id {
            if let Some(tag) = self.pointer_tags.get(&tag_id) {
                if !tag.is_valid || tag.address != address {
                    return Err("指针标记无效或地址不匹配".to_string());
                }
            } else {
                return Err("指针标记不存在".to_string());
            }
        }

        if let Some(block) = self.memory_blocks.get(&address) {
            if !block.is_allocated {
                return Err("尝试访问已释放的内存".to_string());
            }

            // 注意：只读访问不更新last_access_time，以支持并发读取
            Ok(block.value.clone())
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 写入内存（带指针标记验证）
    pub fn write(&mut self, address: usize, value: Value, tag_id: Option<u64>) -> Result<(), String> {
        // 验证指针标记
        if let Some(tag_id) = tag_id {
            if let Some(tag) = self.pointer_tags.get(&tag_id) {
                if !tag.is_valid || tag.address != address {
                    return Err("指针标记无效或地址不匹配".to_string());
                }
            } else {
                return Err("指针标记不存在".to_string());
            }
        }

        // 先计算新值大小，避免借用冲突
        let new_size = self.calculate_size(&value);

        if let Some(block) = self.memory_blocks.get_mut(&address) {
            if !block.is_allocated {
                return Err("尝试写入已释放的内存".to_string());
            }

            if new_size > block.size {
                return Err("新值大小超过分配的内存块".to_string());
            }

            block.value = value;
            block.last_access_time = Self::current_time_ms();
            Ok(())
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 增加引用计数
    pub fn add_ref(&mut self, address: usize) -> Result<(), String> {
        if let Some(block) = self.memory_blocks.get_mut(&address) {
            block.ref_count += 1;
            Ok(())
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 减少引用计数
    pub fn remove_ref(&mut self, address: usize) -> Result<bool, String> {
        if let Some(block) = self.memory_blocks.get_mut(&address) {
            if block.ref_count > 0 {
                block.ref_count -= 1;
                Ok(block.ref_count == 0)
            } else {
                Err("引用计数已为0".to_string())
            }
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 检查地址是否有效
    pub fn is_valid_address(&self, address: usize) -> bool {
        self.memory_blocks.contains_key(&address) && 
        self.memory_blocks[&address].is_allocated
    }

    /// 检查是否为空指针
    pub fn is_null_pointer(&self, address: usize) -> bool {
        address == 0
    }

    /// 检查是否为悬空指针（使用指针标记）
    pub fn is_dangling_pointer(&self, tag_id: u64) -> bool {
        if let Some(tag) = self.pointer_tags.get(&tag_id) {
            if !tag.is_valid {
                return true; // 标记已失效
            }

            // 检查地址是否仍然有效
            if let Some(block) = self.memory_blocks.get(&tag.address) {
                !block.is_allocated
            } else {
                true // 内存块不存在
            }
        } else {
            true // 标记不存在
        }
    }

    /// 检查是否为悬空指针（传统方式，用于向后兼容）
    pub fn is_dangling_pointer_by_address(&self, address: usize) -> bool {
        if address == 0 {
            return false; // 空指针不是悬空指针
        }

        // 检查地址是否在隔离区中
        if self.quarantine_addresses.iter().any(|(addr, _)| *addr == address) {
            return true;
        }

        // 检查地址是否曾经被分配但现在已释放
        !self.memory_blocks.contains_key(&address) ||
        !self.memory_blocks[&address].is_allocated
    }

    /// 检查内存边界
    pub fn check_bounds(&self, address: usize, offset: usize) -> Result<(), String> {
        if let Some(block) = self.memory_blocks.get(&address) {
            if !block.is_allocated {
                return Err("访问已释放的内存".to_string());
            }

            if offset >= block.size {
                return Err(format!("内存访问越界：偏移 {} 超出块大小 {}", offset, block.size));
            }

            Ok(())
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 检测内存泄漏
    pub fn detect_memory_leaks(&self) -> Vec<usize> {
        let mut leaks = Vec::new();

        for (address, block) in &self.memory_blocks {
            if block.is_allocated && block.ref_count == 0 {
                leaks.push(*address);
            }
        }

        leaks
    }

    /// 验证指针有效性（使用指针标记）
    pub fn validate_pointer(&self, address: usize, tag_id: Option<u64>) -> Result<(), String> {
        if self.is_null_pointer(address) {
            return Err("空指针访问".to_string());
        }

        // 检查地址是否在有效范围内
        if !self.is_address_in_valid_range(address) {
            return Err("地址超出有效范围".to_string());
        }

        // 如果有标记，验证标记
        if let Some(tag_id) = tag_id {
            if self.is_dangling_pointer(tag_id) {
                return Err("悬空指针访问".to_string());
            }
        } else {
            // 没有标记时使用传统方式检查
            if self.is_dangling_pointer_by_address(address) {
                return Err("悬空指针访问".to_string());
            }
        }

        if !self.is_valid_address(address) {
            return Err("无效指针访问".to_string());
        }

        Ok(())
    }

    /// 安全的指针算术运算（带边界检查）
    pub fn safe_pointer_arithmetic(&self, address: usize, offset: isize, element_size: usize, tag_id: Option<u64>) -> Result<usize, String> {
        // 验证原指针
        self.validate_pointer(address, tag_id)?;

        // 计算新地址，检查溢出
        let new_address = if offset >= 0 {
            address.checked_add((offset as usize).checked_mul(element_size).ok_or("乘法溢出")?)
                .ok_or("地址加法溢出")?
        } else {
            address.checked_sub(((-offset) as usize).checked_mul(element_size).ok_or("乘法溢出")?)
                .ok_or("地址减法下溢")?
        };

        // 检查新地址是否在有效范围内
        if !self.is_address_in_valid_range(new_address) {
            return Err("指针算术结果超出有效范围".to_string());
        }

        Ok(new_address)
    }

    /// 获取内存块大小
    pub fn get_block_size(&self, address: usize) -> Option<usize> {
        self.memory_blocks.get(&address).map(|block| block.size)
    }

    /// 计算值的内存大小（平台无关）
    fn calculate_size(&self, value: &Value) -> usize {
        match value {
            Value::Int(_) => std::mem::size_of::<i32>(),
            Value::Long(_) => std::mem::size_of::<i64>(),
            Value::Float(_) => std::mem::size_of::<f64>(),
            Value::Bool(_) => std::mem::size_of::<bool>(),
            Value::String(s) => {
                // 字符串内容 + 长度信息 + 容量信息
                s.len() + std::mem::size_of::<usize>() * 2
            },
            Value::Array(arr) => {
                // 数组元素大小 + 长度信息 + 容量信息
                let element_size = if arr.is_empty() {
                    std::mem::size_of::<usize>() // 默认元素大小
                } else {
                    self.calculate_size(&arr[0]) // 使用第一个元素的大小
                };
                arr.len() * element_size + std::mem::size_of::<usize>() * 2
            },
            Value::Object(_) => std::mem::size_of::<usize>() * 8, // 对象基础大小
            Value::EnumValue(_) => std::mem::size_of::<usize>() * 4, // 枚举基础大小
            Value::Pointer(_) => std::mem::size_of::<usize>(), // 指针大小
            Value::ArrayPointer(array_ptr) => {
                // 数组指针大小：指针本身 + 数组元数据
                std::mem::size_of::<usize>() + std::mem::size_of::<usize>() * 2
            },
            Value::PointerArray(ptr_array) => {
                // 指针数组大小：指针数量 * 指针大小 + 元数据
                ptr_array.pointers.len() * std::mem::size_of::<usize>() + std::mem::size_of::<usize>() * 2
            },
            Value::FunctionPointer(_) => std::mem::size_of::<usize>(), // 函数指针大小
            Value::LambdaFunctionPointer(_) => std::mem::size_of::<usize>(), // Lambda函数指针大小
            Value::Lambda(_, _) => std::mem::size_of::<usize>() * 2, // Lambda表达式大小
            Value::LambdaBlock(_, _) => std::mem::size_of::<usize>() * 2, // Lambda块大小
            Value::FunctionReference(_) => std::mem::size_of::<usize>(), // 函数引用大小
            Value::Map(map) => {
                // 映射大小：键值对数量 * (键大小 + 值大小) + 元数据
                let pair_size = map.iter().map(|(k, v)| {
                    k.len() + std::mem::size_of::<usize>() + self.calculate_size(v)
                }).sum::<usize>();
                pair_size + std::mem::size_of::<usize>() * 2
            },
            Value::None => std::mem::size_of::<usize>(), // None值大小
        }
    }

    /// 获取内存统计信息
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.total_allocated,
            total_blocks: self.memory_blocks.len(),
            free_addresses: self.quarantine_addresses.len(),
            max_memory: self.max_memory,
        }
    }

    /// 垃圾回收
    pub fn garbage_collect(&mut self) -> usize {
        let mut collected = 0;
        let mut to_remove = Vec::new();

        for (address, block) in &self.memory_blocks {
            if block.ref_count == 0 && block.is_allocated {
                to_remove.push(*address);
            }
        }

        for address in to_remove {
            if self.deallocate(address).is_ok() {
                collected += 1;
            }
        }

        collected
    }
}

/// 内存统计信息
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_blocks: usize,
    pub free_addresses: usize,
    pub max_memory: usize,
}

/// 🚀 v0.6.2 全局内存管理器实例 - 使用RwLock优化并发性能
lazy_static::lazy_static! {
    pub static ref MEMORY_MANAGER: Arc<RwLock<MemoryManager>> = Arc::new(RwLock::new(MemoryManager::new()));
}

/// 🚀 v0.6.2 快速内存操作：支持读写锁的批量操作
pub fn batch_memory_operations<F, R>(f: F) -> R
where
    F: FnOnce(&mut MemoryManager) -> R,
{
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let mut manager = MEMORY_MANAGER.write().unwrap();
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    track_write_operation!(start_time);
    f(&mut manager)
}

/// 🚀 v0.6.2 新增：只读内存操作，支持并发读取
pub fn batch_memory_read_operations<F, R>(f: F) -> R
where
    F: FnOnce(&MemoryManager) -> R,
{
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    f(&manager)
}

/// 🚀 v0.6.3 简单类型快速分配函数 - 跳过复杂安全检查
fn allocate_simple_type_fast(value: Value) -> Result<(usize, u64), String> {
    let mut manager = MEMORY_MANAGER.write().unwrap();

    // 计算简单类型大小（内联计算，避免函数调用开销）
    let size = match &value {
        Value::Int(_) => std::mem::size_of::<i32>(),
        Value::Long(_) => std::mem::size_of::<i64>(),
        Value::Float(_) => std::mem::size_of::<f64>(),
        Value::Bool(_) => std::mem::size_of::<bool>(),
        _ => unreachable!("allocate_simple_type_fast只应用于简单类型"),
    };

    // 快速内存限制检查
    if manager.total_allocated + size > manager.max_memory {
        return Err("内存不足".to_string());
    }

    // 直接分配地址，跳过隔离区清理
    let address = manager.next_address;
    manager.next_address += size.max(8); // 8字节对齐

    // 简化的地址范围检查
    if address >= 0x1000 + 1024 * 1024 * 100 {
        return Err("地址超出有效范围".to_string());
    }

    let current_time = MemoryManager::current_time_ms();

    // 创建简化的内存块（跳过一些字段的初始化）
    let block = MemoryBlock {
        address,
        size,
        value,
        is_allocated: true,
        ref_count: 1,
        allocation_time: current_time,
        last_access_time: current_time,
    };

    // 简化的标记创建
    let tag_id = manager.next_tag_id;
    manager.next_tag_id += 1;

    let tag = PointerTag {
        tag_id,
        address,
        is_valid: true,
        creation_time: current_time,
    };

    manager.memory_blocks.insert(address, block);
    manager.pointer_tags.insert(tag_id, tag);
    manager.total_allocated += size;

    Ok((address, tag_id))
}

/// 🚀 v0.6.3 智能内存分配 - 根据类型选择快速或安全路径
pub fn allocate_memory_smart(value: Value) -> Result<(usize, u64), String> {
    match &value {
        Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Long(_) => {
            // 简单类型使用快速路径
            allocate_simple_type_fast(value)
        },
        _ => {
            // 复杂类型使用完整的安全路径
            #[cfg(feature = "rwlock-stats")]
            #[cfg(feature = "rwlock-stats")]
            let start_time = SystemTime::now();
            let mut manager = MEMORY_MANAGER.write().unwrap();
            #[cfg(feature = "rwlock-stats")]
            #[cfg(feature = "rwlock-stats")]
            track_write_operation!(start_time);
            manager.allocate(value)
        }
    }
}

/// 🚀 v0.6.2 便捷函数：分配内存（读写锁优化版）
pub fn allocate_memory(value: Value) -> Result<(usize, u64), String> {
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let mut manager = MEMORY_MANAGER.write().unwrap();
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    track_write_operation!(start_time);
    manager.allocate(value)
}

/// 🚀 v0.6.2 便捷函数：释放内存（写锁）
pub fn deallocate_memory(address: usize) -> Result<(), String> {
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let mut manager = MEMORY_MANAGER.write().unwrap();
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    track_write_operation!(start_time);
    manager.deallocate(address)
}

/// 🚀 v0.6.2 便捷函数：读取内存（读锁优化版）
pub fn read_memory(address: usize) -> Result<Value, String> {
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.read_only(address, None)
}

/// 🚀 v0.6.2 便捷函数：安全读取内存（读锁优化版）
pub fn read_memory_safe(address: usize, tag_id: u64) -> Result<Value, String> {
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.read_only(address, Some(tag_id))
}

/// 🚀 v0.6.2 便捷函数：写入内存（写锁）
pub fn write_memory(address: usize, value: Value) -> Result<(), String> {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let mut manager = MEMORY_MANAGER.write().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_write_operation!(start_time);
    manager.write(address, value, None)
}

/// 🚀 v0.6.2 便捷函数：安全写入内存（写锁）
pub fn write_memory_safe(address: usize, value: Value, tag_id: u64) -> Result<(), String> {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let mut manager = MEMORY_MANAGER.write().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_write_operation!(start_time);
    manager.write(address, value, Some(tag_id))
}

/// 🚀 v0.6.2 便捷函数：检查地址有效性（读锁）
pub fn is_valid_address(address: usize) -> bool {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.is_valid_address(address)
}

/// 🚀 v0.6.2 便捷函数：检查空指针（读锁）
pub fn is_null_pointer(address: usize) -> bool {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.is_null_pointer(address)
}

/// 🚀 v0.6.2 便捷函数：检查悬空指针（读锁）
pub fn is_dangling_pointer(tag_id: u64) -> bool {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.is_dangling_pointer(tag_id)
}

/// 🚀 v0.6.2 便捷函数：检查悬空指针（读锁）
pub fn is_dangling_pointer_by_address(address: usize) -> bool {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.is_dangling_pointer_by_address(address)
}

/// 🚀 v0.6.2 便捷函数：验证指针（读锁）
pub fn validate_pointer(address: usize) -> Result<(), String> {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.validate_pointer(address, None)
}

/// 🚀 v0.6.2 便捷函数：安全验证指针（读锁）
pub fn validate_pointer_safe(address: usize, tag_id: u64) -> Result<(), String> {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.validate_pointer(address, Some(tag_id))
}

/// 🚀 v0.6.2 便捷函数：安全指针算术（读锁）
pub fn safe_pointer_arithmetic(address: usize, offset: isize, element_size: usize, tag_id: Option<u64>) -> Result<usize, String> {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.safe_pointer_arithmetic(address, offset, element_size, tag_id)
}

/// 🚀 v0.6.2 便捷函数：检查边界（读锁）
pub fn check_memory_bounds(address: usize, offset: usize) -> Result<(), String> {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.check_bounds(address, offset)
}

/// 🚀 v0.6.2 便捷函数：检测内存泄漏（读锁）
pub fn detect_memory_leaks() -> Vec<usize> {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let manager = MEMORY_MANAGER.read().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_read_operation!(start_time);
    manager.detect_memory_leaks()
}

/// 🚀 v0.6.2 便捷函数：垃圾回收（写锁）
pub fn garbage_collect() -> usize {
    #[cfg(feature = "rwlock-stats")]
    let start_time = SystemTime::now();
    let mut manager = MEMORY_MANAGER.write().unwrap();
    #[cfg(feature = "rwlock-stats")]
    track_write_operation!(start_time);
    manager.garbage_collect()
}

/// 🚀 v0.6.2 新增：读写锁性能统计
#[derive(Debug, Clone)]
pub struct RwLockStats {
    pub read_operations: u64,
    pub write_operations: u64,
    pub avg_read_lock_time_ns: u64,
    pub avg_write_lock_time_ns: u64,
    pub total_read_lock_time_ns: u64,
    pub total_write_lock_time_ns: u64,
}

/// 🚀 v0.6.2 获取读写锁性能统计
#[cfg(feature = "rwlock-stats")]
pub fn get_rwlock_performance_stats() -> RwLockStats {
    let read_ops = READ_OPERATIONS.load(Ordering::Relaxed);
    let write_ops = WRITE_OPERATIONS.load(Ordering::Relaxed);
    let total_read_time = READ_LOCK_TIME.load(Ordering::Relaxed);
    let total_write_time = WRITE_LOCK_TIME.load(Ordering::Relaxed);

    RwLockStats {
        read_operations: read_ops,
        write_operations: write_ops,
        avg_read_lock_time_ns: if read_ops > 0 { total_read_time / read_ops } else { 0 },
        avg_write_lock_time_ns: if write_ops > 0 { total_write_time / write_ops } else { 0 },
        total_read_lock_time_ns: total_read_time,
        total_write_lock_time_ns: total_write_time,
    }
}

#[cfg(not(feature = "rwlock-stats"))]
pub fn get_rwlock_performance_stats() -> RwLockStats {
    RwLockStats {
        read_operations: 0,
        write_operations: 0,
        avg_read_lock_time_ns: 0,
        avg_write_lock_time_ns: 0,
        total_read_lock_time_ns: 0,
        total_write_lock_time_ns: 0,
    }
}

/// 🚀 v0.6.2 打印读写锁性能统计
pub fn print_rwlock_performance_stats() {
    #[cfg(feature = "rwlock-stats")]
    {
        let stats = get_rwlock_performance_stats();
        println!("🚀 v0.6.2 读写锁性能统计:");
        println!("  📖 读操作: {} 次", stats.read_operations);
        println!("  ✏️  写操作: {} 次", stats.write_operations);
        println!("  ⏱️  平均读锁时间: {} ns", stats.avg_read_lock_time_ns);
        println!("  ⏱️  平均写锁时间: {} ns", stats.avg_write_lock_time_ns);
        println!("  📊 总读锁时间: {} ns", stats.total_read_lock_time_ns);
        println!("  📊 总写锁时间: {} ns", stats.total_write_lock_time_ns);

        let total_ops = stats.read_operations + stats.write_operations;
        if total_ops > 0 {
            let read_ratio = (stats.read_operations as f64 / total_ops as f64) * 100.0;
            let write_ratio = (stats.write_operations as f64 / total_ops as f64) * 100.0;
            println!("  📈 读写比例: {:.1}% 读 / {:.1}% 写", read_ratio, write_ratio);
        }
    }

    #[cfg(not(feature = "rwlock-stats"))]
    {
        println!("🚀 v0.6.2 读写锁性能统计: 已禁用（编译时优化）");
        println!("  💡 使用 --features rwlock-stats 重新编译以启用统计");
    }
}

/// 🚀 v0.6.2 清除读写锁性能统计
pub fn clear_rwlock_performance_stats() {
    #[cfg(feature = "rwlock-stats")]
    {
        READ_OPERATIONS.store(0, Ordering::Relaxed);
        WRITE_OPERATIONS.store(0, Ordering::Relaxed);
        READ_LOCK_TIME.store(0, Ordering::Relaxed);
        WRITE_LOCK_TIME.store(0, Ordering::Relaxed);
    }
}

/// 🚀 v0.6.10 批量内存操作扩展 - 循环优化专用
impl MemoryManager {
    /// 批量分配多个值，减少锁获取次数
    pub fn batch_allocate(&mut self, values: Vec<Value>) -> Result<Vec<(usize, u64)>, String> {
        let mut results = Vec::with_capacity(values.len());

        for value in values {
            match self.allocate(value) {
                Ok(result) => results.push(result),
                Err(e) => return Err(format!("批量分配失败: {}", e)),
            }
        }

        Ok(results)
    }

    /// 批量读取多个地址的值
    pub fn batch_read(&self, addresses: &[(usize, u64)]) -> Result<Vec<Value>, String> {
        let mut results = Vec::with_capacity(addresses.len());

        for &(address, tag) in addresses {
            match self.read_only(address, Some(tag)) {
                Ok(value) => results.push(value),
                Err(e) => return Err(format!("批量读取失败 地址{}: {}", address, e)),
            }
        }

        Ok(results)
    }

    /// 批量写入多个地址的值
    pub fn batch_write(&mut self, operations: Vec<(usize, u64, Value)>) -> Result<(), String> {
        for (address, tag, value) in operations {
            if let Err(e) = self.write(address, value, Some(tag)) {
                return Err(format!("批量写入失败 地址{}: {}", address, e));
            }
        }

        Ok(())
    }

    /// 批量释放多个地址
    pub fn batch_deallocate(&mut self, addresses: Vec<(usize, u64)>) -> Result<(), String> {
        for (address, _tag) in addresses {
            // deallocate方法不需要tag参数，只需要地址
            if let Err(e) = self.deallocate(address) {
                return Err(format!("批量释放失败 地址{}: {}", address, e));
            }
        }

        Ok(())
    }

    /// 🚀 v0.6.10 循环专用批量操作 - 合并多次锁获取
    pub fn batch_operations<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        // 在单次锁获取内执行所有操作
        f(self)
    }
}

/// 🚀 v0.6.10 全局批量内存操作API - 循环优化专用
/// 批量分配操作，减少锁获取次数
pub fn batch_allocate_values(values: Vec<Value>) -> Result<Vec<(usize, u64)>, String> {
    batch_memory_operations(|manager| {
        manager.batch_allocate(values)
    })
}

/// 批量读取操作，减少锁获取次数
pub fn batch_read_values(addresses: Vec<(usize, u64)>) -> Result<Vec<Value>, String> {
    batch_memory_read_operations(|manager| {
        manager.batch_read(&addresses)
    })
}

/// 批量写入操作，减少锁获取次数
pub fn batch_write_values(operations: Vec<(usize, u64, Value)>) -> Result<(), String> {
    batch_memory_operations(|manager| {
        manager.batch_write(operations)
    })
}

/// 批量释放操作，减少锁获取次数
pub fn batch_deallocate_values(addresses: Vec<(usize, u64)>) -> Result<(), String> {
    batch_memory_operations(|manager| {
        manager.batch_deallocate(addresses)
    })
}

/// 🚀 v0.6.10 循环优化专用：批量处理循环体内的内存操作
pub fn optimize_loop_memory_operations<F, R>(operations: F) -> R
where
    F: FnOnce() -> R,
{
    // 为循环体提供优化的内存操作环境
    // 这里可以添加循环特定的优化逻辑
    operations()
}

// 🚀 v0.6.11 线程本地内存池系统

/// 线程本地内存池配置
#[derive(Debug, Clone)]
pub struct LocalMemoryPoolConfig {
    /// 初始池大小
    pub initial_pool_size: usize,
    /// 最大池大小
    pub max_pool_size: usize,
    /// 块大小（字节）
    pub block_size: usize,
    /// 预分配块数量
    pub prealloc_blocks: usize,
    /// 自动扩展阈值
    pub expand_threshold: f32,
}

impl Default for LocalMemoryPoolConfig {
    fn default() -> Self {
        Self {
            initial_pool_size: 1024 * 1024,      // 1MB初始大小
            max_pool_size: 16 * 1024 * 1024,     // 16MB最大大小
            block_size: 64,                      // 64字节块大小
            prealloc_blocks: 1000,               // 预分配1000个块
            expand_threshold: 0.8,               // 80%使用率时扩展
        }
    }
}

/// 内存块元数据
#[derive(Debug, Clone)]
struct LocalMemoryBlockMeta {
    address: usize,
    size: usize,
    is_free: bool,
    allocation_time: u64,
    thread_id: std::thread::ThreadId,
}

/// 线程本地内存管理器
#[derive(Debug)]
pub struct LocalMemoryManager {
    /// 配置
    config: LocalMemoryPoolConfig,
    /// 空闲块列表
    free_blocks: Vec<LocalMemoryBlockMeta>,
    /// 已分配块映射
    allocated_blocks: HashMap<usize, LocalMemoryBlockMeta>,
    /// 内存池基地址
    pool_base: usize,
    /// 当前池大小
    current_pool_size: usize,
    /// 下一个可用地址
    next_address: usize,
    /// 分配统计
    allocation_count: u64,
    /// 释放统计
    deallocation_count: u64,
    /// 线程ID
    thread_id: std::thread::ThreadId,
}

impl LocalMemoryManager {
    /// 创建新的线程本地内存管理器
    pub fn new() -> Self {
        Self::with_config(LocalMemoryPoolConfig::default())
    }

    /// 使用指定配置创建内存管理器
    pub fn with_config(config: LocalMemoryPoolConfig) -> Self {
        let thread_id = std::thread::current().id();
        let pool_base = Self::allocate_pool_memory(config.initial_pool_size);

        let mut manager = Self {
            config: config.clone(),
            free_blocks: Vec::with_capacity(config.prealloc_blocks),
            allocated_blocks: HashMap::new(),
            pool_base,
            current_pool_size: config.initial_pool_size,
            next_address: pool_base,
            allocation_count: 0,
            deallocation_count: 0,
            thread_id,
        };

        // 预分配空闲块
        manager.preallocate_blocks();
        manager
    }

    /// 分配池内存（模拟）
    fn allocate_pool_memory(size: usize) -> usize {
        // 在实际实现中，这里会调用系统内存分配
        // 这里我们使用一个模拟的地址空间
        static NEXT_POOL_ADDRESS: AtomicU64 = AtomicU64::new(0x10000000);
        NEXT_POOL_ADDRESS.fetch_add(size as u64, Ordering::SeqCst) as usize
    }

    /// 预分配空闲块
    fn preallocate_blocks(&mut self) {
        let block_count = self.config.prealloc_blocks;
        let block_size = self.config.block_size;

        for i in 0..block_count {
            let address = self.pool_base + i * block_size;
            let block = LocalMemoryBlockMeta {
                address,
                size: block_size,
                is_free: true,
                allocation_time: 0,
                thread_id: self.thread_id,
            };
            self.free_blocks.push(block);
        }

        self.next_address = self.pool_base + block_count * block_size;
    }

    /// 分配内存
    pub fn allocate(&mut self, value: Value) -> Result<(usize, u64), String> {
        // 尝试从空闲块列表分配
        if let Some(block_index) = self.find_suitable_free_block(&value) {
            return self.allocate_from_free_block(block_index, value);
        }

        // 空闲块不足，尝试扩展池
        if self.should_expand_pool() {
            self.expand_pool()?;
            // 重试分配
            if let Some(block_index) = self.find_suitable_free_block(&value) {
                return self.allocate_from_free_block(block_index, value);
            }
        }

        // 从池中分配新块
        self.allocate_new_block(value)
    }

    /// 查找合适的空闲块
    fn find_suitable_free_block(&self, value: &Value) -> Option<usize> {
        let required_size = self.calculate_value_size(value);

        for (index, block) in self.free_blocks.iter().enumerate() {
            if block.is_free && block.size >= required_size {
                return Some(index);
            }
        }

        None
    }

    /// 计算值所需的内存大小
    fn calculate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::Int(_) => 8,
            Value::Long(_) => 8,
            Value::Float(_) => 8,
            Value::Bool(_) => 1,
            Value::String(s) => s.len() + 8, // 字符串长度 + 元数据
            Value::Array(arr) => arr.len() * 8 + 16, // 数组元素 + 元数据
            _ => self.config.block_size, // 默认块大小
        }
    }

    /// 从空闲块分配
    fn allocate_from_free_block(&mut self, block_index: usize, value: Value) -> Result<(usize, u64), String> {
        let mut block = self.free_blocks.remove(block_index);
        block.is_free = false;
        block.allocation_time = self.get_current_time();

        let address = block.address;
        let tag_id = self.generate_tag_id();

        // 将块移动到已分配映射
        self.allocated_blocks.insert(address, block);
        self.allocation_count += 1;

        Ok((address, tag_id))
    }

    /// 分配新块
    fn allocate_new_block(&mut self, value: Value) -> Result<(usize, u64), String> {
        let required_size = self.calculate_value_size(&value);
        let block_size = required_size.max(self.config.block_size);

        // 检查是否有足够空间
        if self.next_address + block_size > self.pool_base + self.current_pool_size {
            return Err("线程本地内存池空间不足".to_string());
        }

        let address = self.next_address;
        let tag_id = self.generate_tag_id();

        let block = LocalMemoryBlockMeta {
            address,
            size: block_size,
            is_free: false,
            allocation_time: self.get_current_time(),
            thread_id: self.thread_id,
        };

        self.allocated_blocks.insert(address, block);
        self.next_address += block_size;
        self.allocation_count += 1;

        Ok((address, tag_id))
    }

    /// 释放内存
    pub fn deallocate(&mut self, address: usize) -> Result<(), String> {
        if let Some(mut block) = self.allocated_blocks.remove(&address) {
            block.is_free = true;
            self.free_blocks.push(block);
            self.deallocation_count += 1;
            Ok(())
        } else {
            Err(format!("无效的内存地址: 0x{:x}", address))
        }
    }

    /// 读取内存
    pub fn read(&self, address: usize, _tag_id: Option<u64>) -> Result<Value, String> {
        if self.allocated_blocks.contains_key(&address) {
            // 在实际实现中，这里会从内存中读取实际数据
            // 这里返回一个模拟值
            Ok(Value::Int(42))
        } else {
            Err(format!("无效的内存地址: 0x{:x}", address))
        }
    }

    /// 写入内存
    pub fn write(&mut self, address: usize, _value: Value, _tag_id: Option<u64>) -> Result<(), String> {
        if self.allocated_blocks.contains_key(&address) {
            // 在实际实现中，这里会将数据写入内存
            Ok(())
        } else {
            Err(format!("无效的内存地址: 0x{:x}", address))
        }
    }

    /// 检查是否应该扩展池
    fn should_expand_pool(&self) -> bool {
        let used_space = self.next_address - self.pool_base;
        let usage_ratio = used_space as f32 / self.current_pool_size as f32;

        usage_ratio > self.config.expand_threshold &&
        self.current_pool_size < self.config.max_pool_size
    }

    /// 扩展内存池
    fn expand_pool(&mut self) -> Result<(), String> {
        let new_size = (self.current_pool_size * 2).min(self.config.max_pool_size);
        if new_size <= self.current_pool_size {
            return Err("内存池已达到最大大小".to_string());
        }

        // 在实际实现中，这里会重新分配更大的内存池
        // 这里我们只是更新大小
        self.current_pool_size = new_size;

        // 预分配更多空闲块
        let additional_blocks = self.config.prealloc_blocks / 2;
        for i in 0..additional_blocks {
            let address = self.next_address + i * self.config.block_size;
            let block = LocalMemoryBlockMeta {
                address,
                size: self.config.block_size,
                is_free: true,
                allocation_time: 0,
                thread_id: self.thread_id,
            };
            self.free_blocks.push(block);
        }

        Ok(())
    }

    /// 获取当前时间戳
    fn get_current_time(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// 生成标签ID
    fn generate_tag_id(&self) -> u64 {
        static NEXT_TAG_ID: AtomicU64 = AtomicU64::new(1);
        NEXT_TAG_ID.fetch_add(1, Ordering::SeqCst)
    }

    /// 获取内存统计信息
    pub fn get_stats(&self) -> LocalMemoryStats {
        LocalMemoryStats {
            thread_id: self.thread_id,
            total_allocations: self.allocation_count,
            total_deallocations: self.deallocation_count,
            active_allocations: self.allocated_blocks.len(),
            free_blocks: self.free_blocks.len(),
            pool_size: self.current_pool_size,
            used_space: self.next_address - self.pool_base,
            fragmentation_ratio: self.calculate_fragmentation(),
        }
    }

    /// 计算碎片化率
    fn calculate_fragmentation(&self) -> f32 {
        if self.free_blocks.is_empty() {
            return 0.0;
        }

        let total_free_space: usize = self.free_blocks.iter().map(|b| b.size).sum();
        let largest_free_block = self.free_blocks.iter().map(|b| b.size).max().unwrap_or(0);

        if total_free_space == 0 {
            0.0
        } else {
            1.0 - (largest_free_block as f32 / total_free_space as f32)
        }
    }
}

/// 线程本地内存统计信息
#[derive(Debug, Clone)]
pub struct LocalMemoryStats {
    pub thread_id: std::thread::ThreadId,
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub active_allocations: usize,
    pub free_blocks: usize,
    pub pool_size: usize,
    pub used_space: usize,
    pub fragmentation_ratio: f32,
}

// 🚀 v0.6.11 线程本地内存池全局接口

thread_local! {
    /// 线程本地内存池实例
    static LOCAL_MEMORY_POOL: RefCell<LocalMemoryManager> =
        RefCell::new(LocalMemoryManager::new());
}

/// 使用线程本地内存池分配内存
pub fn local_allocate_memory(value: Value) -> Result<(usize, u64), String> {
    LOCAL_MEMORY_POOL.with(|pool| {
        pool.borrow_mut().allocate(value)
    })
}

/// 使用线程本地内存池释放内存
pub fn local_deallocate_memory(address: usize) -> Result<(), String> {
    LOCAL_MEMORY_POOL.with(|pool| {
        pool.borrow_mut().deallocate(address)
    })
}

/// 使用线程本地内存池读取内存
pub fn local_read_memory(address: usize, tag_id: Option<u64>) -> Result<Value, String> {
    LOCAL_MEMORY_POOL.with(|pool| {
        pool.borrow().read(address, tag_id)
    })
}

/// 使用线程本地内存池写入内存
pub fn local_write_memory(address: usize, value: Value, tag_id: Option<u64>) -> Result<(), String> {
    LOCAL_MEMORY_POOL.with(|pool| {
        pool.borrow_mut().write(address, value, tag_id)
    })
}

/// 获取线程本地内存池统计信息
pub fn get_local_memory_stats() -> LocalMemoryStats {
    LOCAL_MEMORY_POOL.with(|pool| {
        pool.borrow().get_stats()
    })
}

/// 批量线程本地内存操作
pub fn local_batch_memory_operations<F, R>(f: F) -> R
where
    F: FnOnce(&mut LocalMemoryManager) -> R,
{
    LOCAL_MEMORY_POOL.with(|pool| {
        f(&mut pool.borrow_mut())
    })
}

/// 🚀 v0.6.11 智能内存分配策略
/// 根据值类型和大小选择最优的分配策略
pub fn smart_allocate_memory(value: Value) -> Result<(usize, u64), String> {
    // 分析值的特征
    let value_size = calculate_smart_value_size(&value);
    let is_temporary = is_temporary_value(&value);

    // 选择分配策略
    if is_temporary && value_size <= 64 {
        // 小型临时值：使用线程本地池
        local_allocate_memory(value)
    } else if value_size > 1024 * 1024 {
        // 大型值：使用全局内存管理器
        allocate_memory_smart(value)
    } else {
        // 中等大小值：优先使用线程本地池
        match local_allocate_memory(value.clone()) {
            Ok(result) => Ok(result),
            Err(_) => {
                // 线程本地池失败，回退到全局管理器
                allocate_memory_smart(value)
            }
        }
    }
}

/// 计算智能值大小
fn calculate_smart_value_size(value: &Value) -> usize {
    match value {
        Value::Int(_) => 8,
        Value::Long(_) => 8,
        Value::Float(_) => 8,
        Value::Bool(_) => 1,
        Value::String(s) => s.len() + 16,
        Value::Array(arr) => arr.len() * 8 + 32,
        _ => 64, // 默认大小
    }
}

/// 判断是否为临时值
fn is_temporary_value(value: &Value) -> bool {
    match value {
        Value::Int(_) | Value::Long(_) | Value::Float(_) | Value::Bool(_) => true,
        Value::String(s) => s.len() < 256, // 短字符串视为临时值
        Value::Array(arr) => arr.len() < 10, // 小数组视为临时值
        _ => false,
    }
}
