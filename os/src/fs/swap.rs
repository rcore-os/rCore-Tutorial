//! 内存置换文件

use super::*;
use crate::memory::*;
use algorithm::{Allocator, AllocatorImpl};

/// 内存置换文件在硬盘镜像中的路径（在 `user/Makefile` 中预先生成）
const SWAP_FILE_PATH: &str = "SWAP_FILE";
/// 内存置换文件可以存放的页面数量
const SWAP_FILE_CAPACITY: usize = 4096;

lazy_static! {
    /// 内存置换文件
    static ref SWAP: Swap = Swap {
        inode: ROOT_INODE
            .lookup(SWAP_FILE_PATH)
            .expect("cannot find swap file"),
        allocator: Mutex::new(AllocatorImpl::new(4096)),
    };
}

/// 实现对于内存置换文件的一些操作
struct Swap {
    /// 置换文件
    inode: Arc<dyn INode>,
    /// 置换算法
    allocator: Mutex<AllocatorImpl>,
}

impl Swap {
    /// 在置换文件中分配一个页面的位置
    pub(self) fn alloc(&self) -> MemoryResult<usize> {
        self.allocator.lock().alloc().ok_or("swap file full")
    }

    /// 在置换文件中释放一个页面的位置
    pub(self) fn dealloc(&self, index: usize) {
        assert!(index < SWAP_FILE_CAPACITY);
        self.allocator.lock().dealloc(index);
    }

    /// 将数据写入置换文件指定的页面位置
    pub(self) fn write_page(&self, index: usize, data: &[u8; PAGE_SIZE]) {
        assert!(index < SWAP_FILE_CAPACITY);
        self.inode
            .write_at(index * PAGE_SIZE, data)
            .expect("failed to write swap file");
    }

    /// 从置换文件指定的页面中读取数据
    pub(self) fn read_page(&self, index: usize) -> [u8; PAGE_SIZE] {
        assert!(index < SWAP_FILE_CAPACITY);
        let mut data = [0u8; PAGE_SIZE];
        self.inode
            .read_at(index * PAGE_SIZE, &mut data)
            .expect("failed to read swap file");
        data
    }
}

/// 类似于 [`FrameTracker`]，相当于 `Box<置换文件中的一个页面>`
///
/// 内部保存该置换页面在文件中保存的 index
///
/// [`FrameTracker`]: crate::memory::frame::FrameTracker
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SwapTracker(pub(super) usize);

impl SwapTracker {
    /// 从置换文件分配一个页面空间
    pub fn new() -> MemoryResult<Self> {
        SWAP.alloc().map(Self)
    }

    /// 读取页面数据
    pub fn read(&self) -> [u8; PAGE_SIZE] {
        SWAP.read_page(self.0)
    }

    /// 写入页面数据
    pub fn write(&self, data: &[u8; PAGE_SIZE]) {
        SWAP.write_page(self.0, data);
    }
}

impl Drop for SwapTracker {
    fn drop(&mut self) {
        SWAP.dealloc(self.0);
    }
}
