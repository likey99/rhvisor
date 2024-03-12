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
#![feature(naked_functions)]
use core::{arch::global_asm, mem};

use fdt::Fdt;

use crate::{
    arch::riscv::{
        cpu,
        csr::*,
        plic::{self, init_plic},
    },
    config::*,
    consts::{HV_PHY_BASE, MAX_CPU_NUM},
    error::HvResult,
    memory::frame::Frame,
    percpu::PerCpu,
    zone::zone_create,
};
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
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
mod config;
mod consts;
mod lang_items;
mod logging;
mod memory;
mod percpu;
mod zone;
/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
static INITED_CPUS: AtomicU32 = AtomicU32::new(0);
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);
static ACTIVATED_CPUS: AtomicU32 = AtomicU32::new(0);
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);
static INIT_LATE_OK: AtomicU32 = AtomicU32::new(0);
static MASTER_CPU: AtomicI32 = AtomicI32::new(-1);
fn wait_for(condition: impl Fn() -> bool) -> HvResult {
    while condition() {
        core::hint::spin_loop();
    }
    Ok(())
}

fn wait_for_counter(counter: &AtomicU32, max_value: u32) -> HvResult {
    wait_for(|| counter.load(Ordering::Acquire) < max_value)
}

fn primary_init_early(dtb: usize) -> HvResult {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn __core_end(); // end of kernel
        fn gdtb();
    }
    clear_bss();
    logging::init();
    println!("Hello, world!");
    trace!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    debug!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    info!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    error!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    println!("core_end: {:#x}", __core_end as usize);
    println!("gdtb: {:#x}", gdtb as usize);

    memory::init_heap();
    memory::heap::heap_test();
    memory::init_frame_allocator();
    memory::frame::frame_allocator_test();
    info!("host dtb: {:#x}", dtb);
    let host_fdt = unsafe { fdt::Fdt::from_ptr(dtb as *const u8) }.unwrap();
    memory::init_hv_page_table(host_fdt).unwrap();
    let plic_info = host_fdt.find_node("/soc/plic").unwrap();
    init_plic(
        plic_info.reg().unwrap().next().unwrap().starting_address as usize,
        plic_info.reg().unwrap().next().unwrap().size.unwrap(),
    );
    for vmid in 0..GUESTS.len() {
        info!(
            "guest{} addr: {:#x}, dtb addr: {:#x}",
            vmid,
            GUESTS[vmid].0.as_ptr() as usize,
            GUESTS[vmid].1.as_ptr() as usize
        );
        let vm_paddr_start: usize = GUESTS[vmid].0.as_ptr() as usize;
        zone_create(vmid, vm_paddr_start, GUESTS[vmid].1.as_ptr(), DTB_ADDR);
    }

    INIT_EARLY_OK.store(1, Ordering::Release);
    Ok(())
}

fn primary_init_late() {
    info!("Primary CPU init late...");
    INIT_LATE_OK.store(1, Ordering::Release);
}

fn per_cpu_init(cpu: &mut PerCpu) {
    if cpu.zone.is_none() {
        warn!("zone is not created for cpu {}", cpu.id);
    } else {
        unsafe {
            memory::hv_page_table().read().activate();
            cpu.zone.clone().unwrap().read().gpm_activate();
        };
    }

    println!("CPU {} init OK.", cpu.id);
}
fn wakeup_secondary_cpus(this_id: usize, dtb: usize) {
    for cpu_id in 0..MAX_CPU_NUM {
        if cpu_id == this_id {
            continue;
        }
        sbi_rt::hart_start(cpu_id, HV_PHY_BASE, dtb);
    }
}
/// the rust entry-point of os
#[no_mangle]
pub fn rust_main(cpuid: usize, host_dtb: usize) -> () {
    arch::riscv::trap::init();
    let mut is_primary = false;
    if MASTER_CPU.load(Ordering::Acquire) == -1 {
        MASTER_CPU.store(cpuid as i32, Ordering::Release);
        is_primary = true;
    }
    let cpu = PerCpu::new(cpuid);
    println!("Hello from CPU {},dtb {:#x}!", cpuid, host_dtb);
    if is_primary {
        wakeup_secondary_cpus(cpuid as usize, host_dtb);
    }
    wait_for(|| ENTERED_CPUS.load(Ordering::Acquire) < MAX_CPU_NUM as _);
    assert_eq!(ENTERED_CPUS.load(Ordering::Acquire), MAX_CPU_NUM as _);
    println!(
        "{} CPU {} entered.",
        if is_primary { "Primary" } else { "Secondary" },
        cpuid
    );

    if is_primary {
        primary_init_early(host_dtb); // create root cell here
    } else {
        wait_for_counter(&INIT_EARLY_OK, 1).unwrap();
    }
    per_cpu_init(cpu);

    INITED_CPUS.fetch_add(1, Ordering::SeqCst);
    wait_for_counter(&INITED_CPUS, MAX_CPU_NUM as _);
    cpu.cpu_init(DTB_ADDR);

    if is_primary {
        primary_init_late();
    } else {
        wait_for_counter(&INIT_LATE_OK, 1);
    }
    let sie = read_csr!(CSR_SIE);
    println!("CPU{} sie: {:#x}", cpuid, sie);
    cpu.run_vm();
}
