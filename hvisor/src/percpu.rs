use crate::arch::riscv::cpu::ArchCpu;
use crate::consts::{INVALID_ADDRESS, PAGE_SIZE, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::memory::addr::VirtAddr;
pub struct PerCpu {
    pub id: usize,
    pub cpu_on_entry: usize,
    pub arch_cpu: ArchCpu,
}

impl PerCpu {
    pub fn new(cpu_id: usize) -> Self {
        PerCpu {
            id: cpu_id,
            cpu_on_entry: INVALID_ADDRESS,
            arch_cpu: ArchCpu::new(),
        }
    }
    pub fn cpu_init(&mut self) {
        log::info!("activating cpu {}", self.id);
        self.cpu_on_entry = INVALID_ADDRESS;
        self.arch_cpu.init();
        unreachable!()
    }
}
