use crate::interrupt::{Context, timer};
use riscv::register::{scause::Scause, stval};
use crate::PROCESSOR;
use crate::fs::STDIN;

/// 处理 ebreak 断点
///
/// 继续执行，其中 `sepc` 增加 2 字节，以跳过当前这条 `ebreak` 指令
pub fn breakpoint(context: &mut Context) -> *mut Context {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
    context
}

/// 处理时钟中断
pub fn supervisor_timer(context: &mut Context) -> *mut Context {
    timer::tick();
    PROCESSOR.lock().park_current_thread(context);
    PROCESSOR.lock().prepare_next_thread()
}

/// 处理外部中断，只实现了键盘输入
pub fn supervisor_external(context: &mut Context) -> *mut Context {
    context
}

pub fn supervisor_soft(context: &mut Context) -> *mut Context {
    let mut c = stval::read();
    if c <= 255 {
        if c == '\r' as usize {
            c = '\n' as usize;
        }
        STDIN.push(c as u8);
    }
    unsafe {
        let mut _sip: usize = 0;
        llvm_asm!("csrci sip, 1 << 1" : "=r"(_sip) ::: "volatile");
    }
    context
}
