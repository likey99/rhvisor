//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.

#![deny(missing_docs)]
//#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(asm_const)]
use core::{arch::global_asm, mem};

use crate::{arch::riscv::cpu, memory::frame::Frame, percpu::PerCpu};
#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;
extern crate buddy_system_allocator;
#[macro_use]
mod error;
#[macro_use]
mod console;
mod arch;
mod consts;
mod lang_items;
mod logging;
mod memory;
mod percpu;
mod vm;
global_asm!(include_str!("arch/riscv/arch_entry.S"));

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main(cpuid: usize) -> ! {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    clear_bss();
    logging::init();
    println!("Hello, world!");
    trace!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    debug!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    info!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    warn!(
        "boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    memory::init_heap();
    memory::heap::heap_test();
    memory::init_frame_allocator();
    memory::frame::frame_allocator_test();
    let mut vm = vm::Vm::new(0);
    vm.pt_init();
    unsafe {
        vm.gpm.activate();
    }
    memory::init_hv_page_table();
    arch::riscv::trap::init();

    let cpu = PerCpu::new(cpuid);
    cpu.cpu_init();

    arch::riscv::sbi::shutdown(false)
}
