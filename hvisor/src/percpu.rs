use crate::arch::riscv::cpu::ArchCpu;
use crate::consts::{INVALID_ADDRESS, PAGE_SIZE, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::memory::addr::VirtAddr;
pub struct PerCpu {
    pub id: u64,
    pub cpu_on_entry: u64,
    pub arch_cpu: ArchCpu,
}

impl PerCpu {
    pub fn new(cpu_id: u64) -> Self {
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
