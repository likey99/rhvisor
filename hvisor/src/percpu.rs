use core::mem;

use crate::arch::riscv::cpu::ArchCpu;
use crate::consts::{INVALID_ADDRESS, PAGE_SIZE, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::memory::addr::VirtAddr;
use crate::zone::Zone;
use crate::{memory, read_csr, CSR_SIE, CSR_SIP};
use crate::{ACTIVATED_CPUS, ENTERED_CPUS};
use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use spin::{Mutex, RwLock};
pub struct PerCpu {
    pub id: usize,
    pub cpu_on_entry: usize,
    pub arch_cpu: ArchCpu,
    pub zone: Option<Arc<RwLock<Zone>>>,
    pub ctrl_lock: Mutex<()>,
    pub boot_cpu: bool,
    //percpu stack
}

impl PerCpu {
    pub fn new<'a>(cpu_id: usize) -> &'a mut Self {
        let _cpu_rank = ENTERED_CPUS.fetch_add(1, Ordering::SeqCst);
        let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
        let ret = unsafe { &mut *(vaddr as *mut Self) };
        *ret = PerCpu {
            id: cpu_id,
            cpu_on_entry: INVALID_ADDRESS,
            arch_cpu: ArchCpu::new(cpu_id),
            zone: None,
            ctrl_lock: Mutex::new(()),
            boot_cpu: false,
        };
        ret
    }
    pub fn cpu_init(&mut self, dtb: usize) {
        log::info!(
            "activating cpu {:#x} {:#x} {:#x}",
            self.id,
            self.cpu_on_entry,
            dtb
        );
        self.arch_cpu.init(self.cpu_on_entry, self.id, dtb);
    }
    pub fn run_vm(&mut self) {
        println!("prepare CPU{} for vm run!", self.id);
        if self.boot_cpu {
            println!("boot vm on CPU{}!", self.id);
            self.arch_cpu.run();
        } else {
            self.arch_cpu.idle();

            self.arch_cpu.run();
        }
    }
}
pub fn get_cpu_data<'a>(cpu_id: usize) -> &'a mut PerCpu {
    let cpu_data: usize = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
    unsafe { &mut *(cpu_data as *mut PerCpu) }
}
