//! 一些可能用到，而又不好找库的数据结构
//!
//! 以及有多种实现，会留作业的数据结构
#![no_std]

extern crate alloc;

mod allocator;

pub use allocator::*;
