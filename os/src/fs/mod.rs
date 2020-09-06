//! 文件系统
//!
//! 将读取第一个块设备作为根文件系统

use crate::drivers::{
    block::BlockDevice,
    driver::{DeviceType, DRIVERS},
    block::sdcard::Sd_card,
};
use crate::kernel::Condvar;
use alloc::{sync::Arc, vec::Vec};
use core::any::Any;
use lazy_static::lazy_static;
use rcore_fs_sfs::SimpleFileSystem;
use spin::Mutex;

mod config;
mod inode_ext;
mod stdin;
mod stdout;
mod swap;

pub use config::*;
pub use inode_ext::INodeExt;
pub use rcore_fs::{dev::block_cache::BlockCache, vfs::*};
pub use stdin::STDIN;
pub use stdout::STDOUT;
pub use swap::SwapTracker;

#[cfg(feature = "board_qemu")]
lazy_static! {
    /// 根文件系统的根目录的 INode
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        // 选择第一个块设备
        for driver in DRIVERS.read().iter() {
            if driver.device_type() == DeviceType::Block {
                let device = BlockDevice(driver.clone());
                // 动态分配一段内存空间作为设备 Cache
                let device_with_cache = Arc::new(BlockCache::new(device, BLOCK_CACHE_CAPACITY));
                return SimpleFileSystem::open(device_with_cache)
                    .expect("failed to open SFS")
                    .root_inode();
            }
        }
        panic!("failed to load fs")
    };
}

#[cfg(feature = "board_k210")]
lazy_static! {
    /// 根文件系统的根目录的 INode
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        let device = Arc::new(Sd_card::new());
        let sfs = SimpleFileSystem::open(device).expect("failed to open SFS");
        sfs.root_inode()
    };
}

/// 触发 [`static@ROOT_INODE`] 的初始化并打印根目录内容
pub fn init() {
    ROOT_INODE.ls();
    println!("mod fs initialized");
}
