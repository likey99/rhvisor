//! RISC-V timer-related functionality
#![allow(dead_code)]
use crate::arch::riscv::sbi::set_timer;
use riscv::register::time;
pub const CLOCK_FREQ: usize = 12500000;
pub const MEMORY_END: usize = 0x88000000;
const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
///get current time
pub fn get_time() -> usize {
    time::read()
}
/// get current time in microseconds
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}
/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
