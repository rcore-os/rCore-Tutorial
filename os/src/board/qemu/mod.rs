pub mod config;
pub mod interrupt;

use crate::drivers::init;
use crate::memory::address::PhysicalAddress;

pub fn device_init(dtb_pa: PhysicalAddress) {
    unsafe {
        // 在 OpenSBI 中开启外部中断
        (0x0C00_2080 as *mut u32).write_volatile(1u32 << 10);
        // 在 OpenSBI 中开启串口
        (0x1000_0004 as *mut u8).write_volatile(0x0bu8);
        (0x1000_0001 as *mut u8).write_volatile(0x01u8);
        // 其他一些外部中断相关魔数
        (0x0C00_0028 as *mut u32).write_volatile(0x07u32);
        (0x0C20_1000 as *mut u32).write_volatile(0);
    }
    init(dtb_pa);
}