//! 调用 Machine 层的操作
// 目前还不会用到全部的 SBI 调用，暂时允许未使用的变量或函数
#![allow(unused)]

/// SBI 调用
#[inline(always)]
fn sbi_call(eid: i32,fid: i32,  arg0: usize, arg1: usize, arg2: usize) -> Sbiret {
    let mut error;
    let mut value;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (error), "={x11}" (value)
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (eid), "{x16}" (fid)
            : "memory"      // 如果汇编可能改变内存，则需要加入 memory 选项
            : "volatile"); // 防止编译器做激进的优化（如调换指令顺序等破坏 SBI 调用行为的优化）
    }
    Sbiret{error, value}
}

pub struct Sbiret {
    pub error: i64,
    pub value: i64,
}

const SBI_HSM_STOP_EID:i32 = 0x48534D;
const SBI_HSM_STOP_FID:i32 = 1;

const SBI_SET_TIMER: i32 = 0;
const SBI_CONSOLE_PUTCHAR: i32 = 1;
const SBI_CONSOLE_GETCHAR: i32 = 2;
const SBI_CLEAR_IPI: i32 = 3;
const SBI_SEND_IPI: i32 = 4;
const SBI_REMOTE_FENCE_I: i32 = 5;
const SBI_REMOTE_SFENCE_VMA: i32 = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: i32 = 7;
const SBI_SHUTDOWN: i32 = 8;

/// 向控制台输出一个字符
///
/// 需要注意我们不能直接使用 Rust 中的 char 类型
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, 0, c, 0, 0);
}

/// 从控制台中读取一个字符
///
/// 没有读取到字符则返回 -1
pub fn console_getchar() -> Sbiret {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0, 0)
}

/// 调用 SBI_SHUTDOWN 来关闭操作系统（直接退出 QEMU）
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0,0);
    unreachable!()
}

/// 设置下一次时钟中断的时间
pub fn set_timer(time: usize) {
    sbi_call(SBI_SET_TIMER, 0, time, 0,0);
}

/// 关闭 hart ，等价于 SBI_SHUTDOWN ？
/// TODO: need to verify
pub fn sbi_hart_stop() -> Sbiret {
    sbi_call(SBI_HSM_STOP_EID, SBI_HSM_STOP_FID, 0, 0, 0)
}