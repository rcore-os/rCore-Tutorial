//! 中断栈 [`InterruptStack`]
//!
//! 用户态的线程出现中断时，因为运行栈无法保证可用性，中断处理流程必须在中断栈上进行。
//! 所以我们创建一个公用的中断栈，即当发生中断时，会将 Context 写到中断栈顶。
//!
//! ### 线程 [`Context`] 的存放
//! > 1. 线程初始化时，一个 `Context` 放置在中断栈顶，`sp` 指向 `Context` 的位置
//! >   （即栈顶 - `size_of::<Context>()`）
//! > 2. 切换到线程，执行 `__restore` 时，将 `Context` 的数据恢复到寄存器中后，
//! >   会将 `Context` 出栈（即 `sp += size_of::<Context>()`），
//! >   然后保存 `sp` 至 `sscratch`（此时 `sscratch` 即为中断栈顶）
//! > 3. 发生中断时，将 `sscratch` 和 `sp` 互换，入栈一个 `Context` 并保存数据
//!
//! 容易发现，线程的 `Context` 一定保存在中断栈顶。因此，当线程需要运行时，
//! 从 [`Thread`] 中取出 `Context` 然后置于中断栈顶即可

use super::*;
use core::mem::size_of;

/// 中断栈
#[repr(align(16))]
#[repr(C)]
pub struct InterruptStack([u8; INTERRUPT_STACK_SIZE]);

/// 公用的中断栈
pub static mut INTERRUPT_STACK: InterruptStack = InterruptStack([0; INTERRUPT_STACK_SIZE]);

impl InterruptStack {
    /// 在栈顶加入 Context 并且返回新的栈顶指针
    pub fn push_context(&mut self, context: Context) -> *mut Context {
        // 栈顶
        let stack_top = &self.0 as *const _ as usize + size_of::<Self>();
        // Context 的位置
        let push_address = (stack_top - size_of::<Context>()) as *mut Context;
        unsafe {
            *push_address = context;
        }
        push_address
    }
}
