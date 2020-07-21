//! 文件相关的内核功能

use super::*;
use core::slice::from_raw_parts_mut;

/// 从指定的文件中读取字符
///
/// 如果缓冲区暂无数据，返回 0；出现错误返回 -1
pub(super) fn sys_read(fd: usize, buffer: *mut u8, size: usize) -> SyscallResult {
    // 从进程中获取 inode，注意避免锁
    let thread = PROCESSOR.get().current_thread();
    let process = thread.process.read();
    if let Some(inode) = process.descriptors.get(fd) {
        let buffer = unsafe { from_raw_parts_mut(buffer, size) };
        if let Ok(ret) = inode.read_at(0, buffer) {
            let ret = ret as isize;
            if ret > 0 {
                return SyscallResult::Proceed(ret);
            }
            if ret == 0 {
                return SyscallResult::Park(ret);
            }
        }
    }
    SyscallResult::Proceed(-1)
}

/// 将字符写入指定的文件
pub(super) fn sys_write(fd: usize, buffer: *mut u8, size: usize) -> SyscallResult {
    // 从进程中获取 inode，注意避免锁
    let thread = PROCESSOR.get().current_thread();
    let process = thread.process.read();
    if let Some(inode) = process.descriptors.get(fd) {
        let buffer = unsafe { from_raw_parts_mut(buffer, size) };
        if let Ok(ret) = inode.write_at(0, buffer) {
            let ret = ret as isize;
            if ret >= 0 {
                return SyscallResult::Proceed(ret);
            }
        }
    }
    SyscallResult::Proceed(-1)
}
