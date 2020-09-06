use crate::memory::address::PhysicalAddress;

pub mod config;
pub mod interrupt;

pub fn device_init(_: PhysicalAddress) {
    // after RustSBI, txen = rxen = 1, rxie = 1, rxcnt = 0
    // start UART interrupt configuration
    // disable external interrupt on hart1 by setting threshold
    let hart0_m_threshold: *mut u32 = 0x0c20_0000 as *mut u32;
    let hart1_m_threshold: *mut u32 = 0x0c20_2000 as *mut u32;
    unsafe {
        hart0_m_threshold.write_volatile(0u32);
        hart1_m_threshold.write_volatile(1u32);
    }
    // now using UARTHS whose IRQID = 33
    // assure that its priority equals 1
    let uarths_irq_priority: *mut u32 = (0x0c00_0000 + 33 * 4) as *mut u32;
    assert_eq!(unsafe{ uarths_irq_priority.read_volatile() }, 1);
    // open interrupt enable register on PLIC
    let hart0_m_int_enable_hi: *mut u32 = 0x0c00_2004 as *mut u32;
    unsafe {
        hart0_m_int_enable_hi.write_volatile(1 << 0x1);
    }
    // now, we can receive UARTHS interrupt on hart0!

    crate::drivers::soc::sleep::usleep(1000000);
}