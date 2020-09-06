#![no_std]
#![no_main]
#[macro_use]
extern crate user_lib;

static mut array: [usize; 256 * 1024] = [0usize; 256 * 1024];
#[no_mangle]
pub unsafe fn main()->usize{
    for i in 0..array.len() {
        array[i] = i;
    }
    for i in 0..array.len() {
        assert_eq!(i, array[i]);
    }
    println!("\x1b[32mtest passed\x1b[0m");
    0
}