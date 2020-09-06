//! 定义一些进程相关的常量

use crate::board::config::{
    BOARD_STACK_SIZE,
    BOARD_KERNEL_STACK_SIZE,
};

/// 每个线程的运行栈大小 512 KB
pub const STACK_SIZE: usize = BOARD_STACK_SIZE;

/// 共用的内核栈大小 512 KB
pub const KERNEL_STACK_SIZE: usize = BOARD_KERNEL_STACK_SIZE;
