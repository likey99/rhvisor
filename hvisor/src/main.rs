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
extern crate fdt;
#[macro_use]
mod error;
#[macro_use]
mod console;
mod arch;
mod consts;
mod device;
mod lang_items;
mod logging;
mod memory;
mod percpu;
mod vm;
#[link_section = ".dtb"]
/// the guest dtb file
pub static GUEST_DTB: [u8; include_bytes!("../../guests/linux.dtb").len()] =
    *include_bytes!("../../guests/linux.dtb");
#[link_section = ".initrd"]
static GUEST: [u8; include_bytes!("../../guests/Image-62").len()] =
    *include_bytes!("../../guests/Image-62");

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
pub fn rust_main(cpuid: usize, dtb: usize) -> ! {
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
        fn __core_end(); // end of kernel
        fn gdtb();
        fn vmimg();
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
    println!("core_end: {:#x}", __core_end as usize);
    println!("gdtb: {:#x}", gdtb as usize);
    println!("vmimg: {:#x}", vmimg as usize);

    memory::init_heap();
    memory::heap::heap_test();
    memory::init_frame_allocator();
    memory::frame::frame_allocator_test();
    debug!("host dtb: {:#x}", dtb);
    let host_fdt = unsafe { fdt::Fdt::from_ptr(dtb as *const u8) }.unwrap();
    //let vm_vaddr_start: usize = 0x8020_0000;
    //let vm_paddr_start: usize = 0x8040_0000;
    let vm_paddr_start: usize = GUEST.as_ptr() as usize;
    //let vm_mem_size: usize = 0x0080_0000;
    let guest_fdt = unsafe { fdt::Fdt::from_ptr(GUEST_DTB.as_ptr()) }.unwrap();
    let mut vm = vm::Vm::new(0);
    vm.pt_init(vm_paddr_start, guest_fdt, dtb).unwrap();
    unsafe {
        vm.gpm.activate();
    }
    //unreachable!();
    memory::init_hv_page_table(host_fdt).unwrap();
    unsafe {
        memory::hv_page_table().read().activate();
    }
    arch::riscv::trap::init();

    let cpu = PerCpu::new(cpuid);
    debug!(
        "guest entry: {:#x}, guest size: {:#x}",
        GUEST.as_ptr() as usize,
        GUEST.len()
    );
    let guest_entry = guest_fdt
        .memory()
        .regions()
        .next()
        .unwrap()
        .starting_address as usize;
    cpu.cpu_init(guest_entry, dtb);

    arch::riscv::sbi::shutdown(false)
}
