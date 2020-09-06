mod k210;
mod qemu;

#[cfg(feature = "board_k210")]
pub use k210::{config, device_init, interrupt};

#[cfg(feature = "board_qemu")]
pub use qemu::{config, device_init, interrupt};