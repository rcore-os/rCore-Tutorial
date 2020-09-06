use crate::interrupt::timer::read_time;
use super::sysctl;

pub fn cycle_sleep(n: usize) {
    //let start = mcycle::read();
    let start = read_time();
    while (/*mcycle::read()*/read_time().wrapping_sub(start)) < n {
        // IDLE
    }
}

pub fn usleep(n: usize) {
    let freq = sysctl::clock_get_freq(sysctl::clock::CPU) as usize;
    cycle_sleep(freq * n / 1000000);
}