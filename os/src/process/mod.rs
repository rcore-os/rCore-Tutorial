//! 管理进程 / 线程

mod config;
mod interrupt_stack;
mod lock;
#[allow(clippy::module_inception)]
mod process;
mod processor;
mod thread;

use crate::interrupt::*;
use crate::memory::*;
use alloc::{sync::Arc, vec, vec::Vec};
use spin::Mutex;

pub use config::*;
pub use interrupt_stack::INTERRUPT_STACK;
pub use lock::Lock;
pub use process::Process;
pub use processor::PROCESSOR;
pub use thread::Thread;
