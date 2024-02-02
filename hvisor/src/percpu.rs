use core::mem;

use crate::arch::riscv::cpu::ArchCpu;
use crate::consts::{INVALID_ADDRESS, PAGE_SIZE, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::memory;
use crate::memory::addr::VirtAddr;
pub struct PerCpu {
    pub id: usize,
    pub cpu_on_entry: usize,
    pub arch_cpu: ArchCpu,
    //percpu stack
}

impl PerCpu {
    pub fn new<'a>(cpu_id: usize) -> &'a mut Self {
        let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
        let ret = unsafe { &mut *(vaddr as *mut Self) };
        *ret = PerCpu {
            id: cpu_id,
            cpu_on_entry: INVALID_ADDRESS,
            arch_cpu: ArchCpu::new(cpu_id),
        };
        ret
    }
    pub fn cpu_init(&mut self) {
        log::info!("activating cpu {}", self.id);
        self.cpu_on_entry = 0x8020_0000;
        unsafe {
            memory::hv_page_table().read().activate();
        }
        self.arch_cpu.init(self.cpu_on_entry);
        unreachable!()
    }
}
