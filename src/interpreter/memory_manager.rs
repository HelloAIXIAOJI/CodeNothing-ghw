use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use super::value::Value;

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

/// 全局内存管理器实例
lazy_static::lazy_static! {
    pub static ref MEMORY_MANAGER: Arc<Mutex<MemoryManager>> = Arc::new(Mutex::new(MemoryManager::new()));
}

/// 便捷函数：分配内存
pub fn allocate_memory(value: Value) -> Result<(usize, u64), String> {
    MEMORY_MANAGER.lock().unwrap().allocate(value)
}

/// 便捷函数：释放内存
pub fn deallocate_memory(address: usize) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().deallocate(address)
}

/// 便捷函数：读取内存
pub fn read_memory(address: usize) -> Result<Value, String> {
    MEMORY_MANAGER.lock().unwrap().read(address, None)
}

/// 便捷函数：安全读取内存（带标记验证）
pub fn read_memory_safe(address: usize, tag_id: u64) -> Result<Value, String> {
    MEMORY_MANAGER.lock().unwrap().read(address, Some(tag_id))
}

/// 便捷函数：写入内存
pub fn write_memory(address: usize, value: Value) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().write(address, value, None)
}

/// 便捷函数：安全写入内存（带标记验证）
pub fn write_memory_safe(address: usize, value: Value, tag_id: u64) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().write(address, value, Some(tag_id))
}

/// 便捷函数：检查地址有效性
pub fn is_valid_address(address: usize) -> bool {
    MEMORY_MANAGER.lock().unwrap().is_valid_address(address)
}

/// 便捷函数：检查空指针
pub fn is_null_pointer(address: usize) -> bool {
    MEMORY_MANAGER.lock().unwrap().is_null_pointer(address)
}

/// 便捷函数：检查悬空指针（使用标记）
pub fn is_dangling_pointer(tag_id: u64) -> bool {
    MEMORY_MANAGER.lock().unwrap().is_dangling_pointer(tag_id)
}

/// 便捷函数：检查悬空指针（传统方式）
pub fn is_dangling_pointer_by_address(address: usize) -> bool {
    MEMORY_MANAGER.lock().unwrap().is_dangling_pointer_by_address(address)
}

/// 便捷函数：验证指针
pub fn validate_pointer(address: usize) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().validate_pointer(address, None)
}

/// 便捷函数：安全验证指针（带标记）
pub fn validate_pointer_safe(address: usize, tag_id: u64) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().validate_pointer(address, Some(tag_id))
}

/// 便捷函数：安全指针算术
pub fn safe_pointer_arithmetic(address: usize, offset: isize, element_size: usize, tag_id: Option<u64>) -> Result<usize, String> {
    MEMORY_MANAGER.lock().unwrap().safe_pointer_arithmetic(address, offset, element_size, tag_id)
}

/// 便捷函数：检查边界
pub fn check_memory_bounds(address: usize, offset: usize) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().check_bounds(address, offset)
}

/// 便捷函数：检测内存泄漏
pub fn detect_memory_leaks() -> Vec<usize> {
    MEMORY_MANAGER.lock().unwrap().detect_memory_leaks()
}

/// 便捷函数：垃圾回收
pub fn garbage_collect() -> usize {
    MEMORY_MANAGER.lock().unwrap().garbage_collect()
}
