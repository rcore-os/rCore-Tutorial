#![allow(dead_code)]

pub const BOARD_MEMORY_END_ADDRESS: usize = 0x8060_0000;
pub const BOARD_KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const BOARD_STACK_SIZE: usize = 0x8000;
pub const BOARD_KERNEL_STACK_SIZE: usize = 0x8000;

pub const MMIO_INTERVALS: &[(usize, usize)] = &[
    (0x0200_0000, 0x1000), /* CLINT     */
    (0x0C00_0000, 0x1000), /* PLIC      */
    (0x3800_0000, 0x1000), /* UARTHS    */
    (0x3800_1000, 0x1000), /* GPIOHS    */
    (0x5020_0000, 0x1000), /* GPIO      */
    (0x5024_0000, 0x1000), /* SPI_SLAVE */
    (0x502B_0000, 0x1000), /* FPIOA     */
    (0x502D_0000, 0x1000), /* TIMER0    */
    (0x502E_0000, 0x1000), /* TIMER1    */
    (0x502F_0000, 0x1000), /* TIMER2    */
    (0x5044_0000, 0x1000), /* SYSCTL    */
    (0x5200_0000, 0x1000), /* SPI0      */
    (0x5300_0000, 0x1000), /* SPI1      */
    (0x5400_0000, 0x1000), /* SPI2      */
];

pub const RISCV_SPEC_MAJOR: usize = 1;
pub const RISCV_SPEC_MINOR: usize = 9;
pub const RISCV_SPEC_PATCH: usize = 1;