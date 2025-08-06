// CodeNothing v0.7.5 内存预分配池
// 实现高效的内存管理，减少动态分配开销

pub mod pool_value;

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

/// 自定义内存分配错误
#[derive(Debug, Clone)]
pub struct MemoryAllocError {
    pub message: String,
}

impl std::fmt::Display for MemoryAllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory allocation error: {}", self.message)
    }
}

impl std::error::Error for MemoryAllocError {}

/// 内存块大小类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockSize {
    Small = 8,      // 8字节 - 用于小对象
    Medium = 64,    // 64字节 - 用于中等对象
    Large = 512,    // 512字节 - 用于大对象
    XLarge = 4096,  // 4KB - 用于超大对象
}

impl BlockSize {
    /// 根据请求大小选择合适的块大小
    pub fn from_size(size: usize) -> Self {
        if size <= 8 {
            BlockSize::Small
        } else if size <= 64 {
            BlockSize::Medium
        } else if size <= 512 {
            BlockSize::Large
        } else {
            BlockSize::XLarge
        }
    }

    /// 获取块大小的字节数
    pub fn bytes(self) -> usize {
        self as usize
    }
}

/// 内存块结构
#[derive(Debug)]
pub struct MemoryBlock {
    ptr: *mut u8,
    size: usize,
    in_use: bool,
}

// 为了线程安全，我们需要手动实现Send和Sync
unsafe impl Send for MemoryBlock {}
unsafe impl Sync for MemoryBlock {}

impl MemoryBlock {
    /// 创建新的内存块
    pub fn new(size: usize) -> Result<Self, MemoryAllocError> {
        let layout = Layout::from_size_align(size, 8)
            .map_err(|e| MemoryAllocError { message: format!("Layout error: {:?}", e) })?;

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(MemoryAllocError { message: "Failed to allocate memory".to_string() });
        }

        Ok(MemoryBlock {
            ptr,
            size,
            in_use: false,
        })
    }

    /// 获取内存块指针
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    /// 标记为使用中
    pub fn mark_used(&mut self) {
        self.in_use = true;
    }

    /// 标记为未使用
    pub fn mark_free(&mut self) {
        self.in_use = false;
    }

    /// 检查是否在使用中
    pub fn is_in_use(&self) -> bool {
        self.in_use
    }
}

impl Drop for MemoryBlock {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            let layout = Layout::from_size_align(self.size, 8).unwrap();
            unsafe {
                dealloc(self.ptr, layout);
            }
        }
    }
}

/// 内存池统计信息
#[derive(Debug, Clone, Default)]
pub struct MemoryPoolStats {
    pub total_allocations: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub blocks_allocated: usize,
    pub blocks_freed: usize,
    pub peak_usage: usize,
    pub current_usage: usize,
}

impl MemoryPoolStats {
    /// 计算池命中率
    pub fn hit_rate(&self) -> f64 {
        if self.total_allocations == 0 {
            0.0
        } else {
            self.pool_hits as f64 / self.total_allocations as f64 * 100.0
        }
    }

    /// 打印统计信息
    pub fn print_stats(&self) {
        println!("=== CodeNothing v0.7.5 内存池统计 ===");
        println!("总分配次数: {}", self.total_allocations);
        println!("池命中次数: {}", self.pool_hits);
        println!("池未命中次数: {}", self.pool_misses);
        println!("池命中率: {:.2}%", self.hit_rate());
        println!("已分配块数: {}", self.blocks_allocated);
        println!("已释放块数: {}", self.blocks_freed);
        println!("峰值使用量: {} bytes", self.peak_usage);
        println!("当前使用量: {} bytes", self.current_usage);
        println!("=====================================");
    }
}

/// 内存池实现
pub struct MemoryPool {
    small_blocks: VecDeque<MemoryBlock>,    // 8字节块
    medium_blocks: VecDeque<MemoryBlock>,   // 64字节块
    large_blocks: VecDeque<MemoryBlock>,    // 512字节块
    xlarge_blocks: VecDeque<MemoryBlock>,   // 4KB块
    stats: MemoryPoolStats,
    max_blocks_per_size: usize,
}

impl MemoryPool {
    /// 创建新的内存池
    pub fn new() -> Self {
        Self {
            small_blocks: VecDeque::new(),
            medium_blocks: VecDeque::new(),
            large_blocks: VecDeque::new(),
            xlarge_blocks: VecDeque::new(),
            stats: MemoryPoolStats::default(),
            max_blocks_per_size: 100, // 每种大小最多缓存100个块
        }
    }

    /// 预分配内存块
    pub fn preallocate(&mut self, small_count: usize, medium_count: usize, large_count: usize, xlarge_count: usize) -> Result<(), MemoryAllocError> {
        // 预分配小块
        for _ in 0..small_count {
            let block = MemoryBlock::new(BlockSize::Small.bytes())?;
            self.small_blocks.push_back(block);
        }

        // 预分配中等块
        for _ in 0..medium_count {
            let block = MemoryBlock::new(BlockSize::Medium.bytes())?;
            self.medium_blocks.push_back(block);
        }

        // 预分配大块
        for _ in 0..large_count {
            let block = MemoryBlock::new(BlockSize::Large.bytes())?;
            self.large_blocks.push_back(block);
        }

        // 预分配超大块
        for _ in 0..xlarge_count {
            let block = MemoryBlock::new(BlockSize::XLarge.bytes())?;
            self.xlarge_blocks.push_back(block);
        }

        self.stats.blocks_allocated += small_count + medium_count + large_count + xlarge_count;

        // 只在启用内存调试时显示预分配信息
        crate::memory_debug_println!("内存池预分配完成: {} small, {} medium, {} large, {} xlarge",
                small_count, medium_count, large_count, xlarge_count);

        Ok(())
    }

    /// 从池中分配内存
    pub fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        self.stats.total_allocations += 1;
        let block_size = BlockSize::from_size(size);

        let blocks = match block_size {
            BlockSize::Small => &mut self.small_blocks,
            BlockSize::Medium => &mut self.medium_blocks,
            BlockSize::Large => &mut self.large_blocks,
            BlockSize::XLarge => &mut self.xlarge_blocks,
        };

        // 尝试从池中获取空闲块
        for block in blocks.iter_mut() {
            if !block.is_in_use() {
                block.mark_used();
                self.stats.pool_hits += 1;
                self.stats.current_usage += block_size.bytes();
                if self.stats.current_usage > self.stats.peak_usage {
                    self.stats.peak_usage = self.stats.current_usage;
                }
                return Some(block.as_ptr());
            }
        }

        // 池中没有可用块，尝试创建新块
        if blocks.len() < self.max_blocks_per_size {
            if let Ok(mut new_block) = MemoryBlock::new(block_size.bytes()) {
                new_block.mark_used();
                let ptr = new_block.as_ptr();
                blocks.push_back(new_block);
                self.stats.pool_misses += 1;
                self.stats.blocks_allocated += 1;
                self.stats.current_usage += block_size.bytes();
                if self.stats.current_usage > self.stats.peak_usage {
                    self.stats.peak_usage = self.stats.current_usage;
                }
                return Some(ptr);
            }
        }

        // 无法分配
        self.stats.pool_misses += 1;
        None
    }

    /// 释放内存到池中
    pub fn deallocate(&mut self, ptr: *mut u8, size: usize) {
        let block_size = BlockSize::from_size(size);

        let blocks = match block_size {
            BlockSize::Small => &mut self.small_blocks,
            BlockSize::Medium => &mut self.medium_blocks,
            BlockSize::Large => &mut self.large_blocks,
            BlockSize::XLarge => &mut self.xlarge_blocks,
        };

        // 查找对应的块并标记为未使用
        for block in blocks.iter_mut() {
            if block.as_ptr() == ptr {
                block.mark_free();
                self.stats.blocks_freed += 1;
                self.stats.current_usage = self.stats.current_usage.saturating_sub(block_size.bytes());
                return;
            }
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &MemoryPoolStats {
        &self.stats
    }

    /// 清理未使用的块
    pub fn cleanup(&mut self) {
        let cleanup_blocks = |blocks: &mut VecDeque<MemoryBlock>| {
            blocks.retain(|block| block.is_in_use());
        };

        cleanup_blocks(&mut self.small_blocks);
        cleanup_blocks(&mut self.medium_blocks);
        cleanup_blocks(&mut self.large_blocks);
        cleanup_blocks(&mut self.xlarge_blocks);
    }
}

/// 全局内存池
static GLOBAL_MEMORY_POOL: std::sync::OnceLock<Arc<Mutex<MemoryPool>>> = std::sync::OnceLock::new();

/// 获取全局内存池
pub fn get_global_memory_pool() -> &'static Arc<Mutex<MemoryPool>> {
    GLOBAL_MEMORY_POOL.get_or_init(|| {
        let mut pool = MemoryPool::new();
        // 预分配一些常用大小的块
        if let Err(e) = pool.preallocate(50, 30, 20, 10) {
            eprintln!("警告: 内存池预分配失败: {:?}", e);
        }
        Arc::new(Mutex::new(pool))
    })
}

/// 便捷的内存分配函数
pub fn pool_allocate(size: usize) -> Option<*mut u8> {
    if let Ok(mut pool) = get_global_memory_pool().lock() {
        pool.allocate(size)
    } else {
        None
    }
}

/// 便捷的内存释放函数
pub fn pool_deallocate(ptr: *mut u8, size: usize) {
    if let Ok(mut pool) = get_global_memory_pool().lock() {
        pool.deallocate(ptr, size);
    }
}

/// 打印内存池统计信息
pub fn print_memory_pool_stats() {
    if let Ok(pool) = get_global_memory_pool().lock() {
        pool.get_stats().print_stats();
    }
}

/// 清理内存池
pub fn cleanup_memory_pool() {
    if let Ok(mut pool) = get_global_memory_pool().lock() {
        pool.cleanup();
    }
}

/// 智能指针包装器，自动管理内存池分配的内存
pub struct PoolPtr<T> {
    ptr: *mut T,
    size: usize,
}

impl<T> PoolPtr<T> {
    /// 从内存池分配新对象
    pub fn new(value: T) -> Option<Self> {
        let size = std::mem::size_of::<T>();
        if let Some(raw_ptr) = pool_allocate(size) {
            let ptr = raw_ptr as *mut T;
            unsafe {
                std::ptr::write(ptr, value);
                Some(PoolPtr {
                    ptr,
                    size,
                })
            }
        } else {
            None
        }
    }

    /// 获取原始指针
    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    /// 获取引用
    pub fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }

    /// 获取可变引用
    pub fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for PoolPtr<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                // 调用析构函数
                std::ptr::drop_in_place(self.ptr);
                // 释放内存到池中
                pool_deallocate(self.ptr as *mut u8, self.size);
            }
        }
    }
}

impl<T> std::ops::Deref for PoolPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> std::ops::DerefMut for PoolPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

/// 内存池分配器宏
#[macro_export]
macro_rules! pool_alloc {
    ($value:expr) => {
        $crate::memory_pool::PoolPtr::new($value)
    };
}

/// 批量分配宏
#[macro_export]
macro_rules! pool_alloc_vec {
    ($value:expr; $count:expr) => {{
        let mut vec = Vec::with_capacity($count);
        for _ in 0..$count {
            if let Some(ptr) = $crate::memory_pool::PoolPtr::new($value.clone()) {
                vec.push(ptr);
            } else {
                break;
            }
        }
        vec
    }};
}
