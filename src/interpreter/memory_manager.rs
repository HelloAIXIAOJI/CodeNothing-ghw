use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::value::Value;

/// 内存块信息
#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub address: usize,
    pub size: usize,
    pub value: Value,
    pub is_allocated: bool,
    pub ref_count: usize,
}

/// 内存管理器
#[derive(Debug)]
pub struct MemoryManager {
    memory_blocks: HashMap<usize, MemoryBlock>,
    next_address: usize,
    free_addresses: Vec<usize>,
    total_allocated: usize,
    max_memory: usize,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            memory_blocks: HashMap::new(),
            next_address: 0x1000, // 从较高地址开始，避免与系统地址冲突
            free_addresses: Vec::new(),
            total_allocated: 0,
            max_memory: 1024 * 1024 * 100, // 100MB 限制
        }
    }

    /// 分配内存并返回地址
    pub fn allocate(&mut self, value: Value) -> Result<usize, String> {
        let size = self.calculate_size(&value);
        
        // 检查内存限制
        if self.total_allocated + size > self.max_memory {
            return Err("内存不足".to_string());
        }

        // 尝试重用释放的地址
        let address = if let Some(addr) = self.free_addresses.pop() {
            addr
        } else {
            let addr = self.next_address;
            self.next_address += size.max(8); // 至少8字节对齐
            addr
        };

        let block = MemoryBlock {
            address,
            size,
            value,
            is_allocated: true,
            ref_count: 1,
        };

        self.memory_blocks.insert(address, block);
        self.total_allocated += size;

        Ok(address)
    }

    /// 释放内存
    pub fn deallocate(&mut self, address: usize) -> Result<(), String> {
        if let Some(mut block) = self.memory_blocks.remove(&address) {
            if !block.is_allocated {
                return Err("尝试释放已释放的内存".to_string());
            }

            block.is_allocated = false;
            self.total_allocated -= block.size;
            self.free_addresses.push(address);
            
            Ok(())
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 读取内存中的值
    pub fn read(&self, address: usize) -> Result<Value, String> {
        if let Some(block) = self.memory_blocks.get(&address) {
            if !block.is_allocated {
                return Err("尝试访问已释放的内存".to_string());
            }
            Ok(block.value.clone())
        } else {
            Err("无效的内存地址".to_string())
        }
    }

    /// 写入内存
    pub fn write(&mut self, address: usize, value: Value) -> Result<(), String> {
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

    /// 检查是否为悬空指针
    pub fn is_dangling_pointer(&self, address: usize) -> bool {
        if address == 0 {
            return false; // 空指针不是悬空指针
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

    /// 验证指针有效性
    pub fn validate_pointer(&self, address: usize) -> Result<(), String> {
        if self.is_null_pointer(address) {
            return Err("空指针访问".to_string());
        }

        if self.is_dangling_pointer(address) {
            return Err("悬空指针访问".to_string());
        }

        if !self.is_valid_address(address) {
            return Err("无效指针访问".to_string());
        }

        Ok(())
    }

    /// 获取内存块大小
    pub fn get_block_size(&self, address: usize) -> Option<usize> {
        self.memory_blocks.get(&address).map(|block| block.size)
    }

    /// 计算值的内存大小
    fn calculate_size(&self, value: &Value) -> usize {
        match value {
            Value::Int(_) => 4,
            Value::Long(_) => 8,
            Value::Float(_) => 4,
            Value::Bool(_) => 1,
            Value::String(s) => s.len() + 8, // 字符串长度 + 元数据
            Value::Array(arr) => arr.len() * 8 + 16, // 数组元素 + 元数据
            Value::Object(_) => 64, // 对象基础大小
            Value::EnumValue(_) => 32, // 枚举基础大小
            Value::Pointer(_) => 8, // 指针大小
            _ => 8, // 默认大小
        }
    }

    /// 获取内存统计信息
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.total_allocated,
            total_blocks: self.memory_blocks.len(),
            free_addresses: self.free_addresses.len(),
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
pub fn allocate_memory(value: Value) -> Result<usize, String> {
    MEMORY_MANAGER.lock().unwrap().allocate(value)
}

/// 便捷函数：释放内存
pub fn deallocate_memory(address: usize) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().deallocate(address)
}

/// 便捷函数：读取内存
pub fn read_memory(address: usize) -> Result<Value, String> {
    MEMORY_MANAGER.lock().unwrap().read(address)
}

/// 便捷函数：写入内存
pub fn write_memory(address: usize, value: Value) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().write(address, value)
}

/// 便捷函数：检查地址有效性
pub fn is_valid_address(address: usize) -> bool {
    MEMORY_MANAGER.lock().unwrap().is_valid_address(address)
}

/// 便捷函数：检查空指针
pub fn is_null_pointer(address: usize) -> bool {
    MEMORY_MANAGER.lock().unwrap().is_null_pointer(address)
}

/// 便捷函数：检查悬空指针
pub fn is_dangling_pointer(address: usize) -> bool {
    MEMORY_MANAGER.lock().unwrap().is_dangling_pointer(address)
}

/// 便捷函数：验证指针
pub fn validate_pointer(address: usize) -> Result<(), String> {
    MEMORY_MANAGER.lock().unwrap().validate_pointer(address)
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
