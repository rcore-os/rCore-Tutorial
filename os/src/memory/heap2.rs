//! 如果想要尝试自己实现动态分配器，使用此文件替换 heap.rs
//!
//! 具体分配算法需要在 algorithm::allocator 里面实现，
//! 这里将其中的 VectorAllocator 接入 GlobalAlloc，作为全局分配器

use super::config::KERNEL_HEAP_SIZE;
use algorithm::{VectorAllocator, VectorAllocatorImpl};
use core::cell::UnsafeCell;

/// 进行动态内存分配所用的堆空间
///
/// 大小为 [`KERNEL_HEAP_SIZE`]
/// 这段空间编译后会被放在操作系统执行程序的 bss 段
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static HEAP: Heap = Heap(UnsafeCell::new(None));

/// Heap 将分配器封装并放在 static 中。它不安全，但在这个问题中不考虑安全性
struct Heap(UnsafeCell<Option<VectorAllocatorImpl>>);

/// 利用 VectorAllocator 的接口实现全局分配器的 GlobalAlloc trait
unsafe impl alloc::alloc::GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let offset = (*self.0.get())
            .as_mut()
            .unwrap()
            .alloc(layout.size(), layout.align())
            .expect("Heap overflow");
        &mut HEAP_SPACE[offset] as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let offset = ptr as usize - &HEAP_SPACE as *const _ as usize;
        (*self.0.get())
            .as_mut()
            .unwrap()
            .dealloc(offset, layout.size(), layout.align());
    }
}

unsafe impl Sync for Heap {}

/// 初始化操作系统运行时堆空间
pub fn init() {
    // 告诉分配器使用这一段预留的空间作为堆
    unsafe {
        (*HEAP.0.get()).replace(VectorAllocatorImpl::new(KERNEL_HEAP_SIZE));
    }
}

/// 空间分配错误的回调，直接 panic 退出
#[alloc_error_handler]
fn alloc_error_handler(_: alloc::alloc::Layout) -> ! {
    panic!("alloc error")
}
