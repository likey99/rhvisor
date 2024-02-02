use crate::arch::riscv::s2pt::Stage2PageTable;
use crate::memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet};
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CpuSet {
    max_cpu_id: u64,
    bitmap: u64,
}

impl CpuSet {
    pub fn new(max_cpu_id: u64, bitmap: u64) -> Self {
        Self { max_cpu_id, bitmap }
    }
    pub fn from_cpuset_slice(cpu_set: &[u8]) -> Self {
        if cpu_set.len() != 8 {
            todo!("Cpu_set should be 8 bytes!");
        }
        let cpu_set_long: u64 = cpu_set
            .iter()
            .enumerate()
            .fold(0, |acc, (i, x)| acc | (*x as u64) << (i * 8));
        Self::new(cpu_set.len() as u64 * 8 - 1, cpu_set_long)
    }
    #[allow(unused)]
    pub fn set_bit(&mut self, id: u64) {
        assert!(id <= self.max_cpu_id);
        self.bitmap |= 1 << id;
    }
    pub fn clear_bit(&mut self, id: u64) {
        assert!(id <= self.max_cpu_id);
        self.bitmap &= !(1 << id);
    }
    pub fn contains_cpu(&self, id: u64) -> bool {
        id <= self.max_cpu_id && (self.bitmap & (1 << id)) != 0
    }
    #[allow(unused)]
    pub fn first_cpu(&self) -> Option<u64> {
        (0..=self.max_cpu_id).find(move |&i| self.contains_cpu(i))
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u64> + 'a {
        (0..=self.max_cpu_id).filter(move |&i| self.contains_cpu(i))
    }
    pub fn iter_except<'a>(&'a self, id: u64) -> impl Iterator<Item = u64> + 'a {
        (0..=self.max_cpu_id).filter(move |&i| self.contains_cpu(i) && i != id)
    }
}

pub struct Vm {
    pub vmid: u64,
    pub gpm: MemorySet<Stage2PageTable>,
    pub cpu_set: CpuSet,
}
impl Vm {
    pub fn new(vmid: u64) -> Self {
        Self {
            vmid,
            gpm: MemorySet::new(),
            cpu_set: CpuSet::new(1, 1),
        }
    }
    pub fn pt_init(&mut self) {
        let vm_vaddr_start: usize = 0x8020_0000;
        let vm_paddr_start: usize = 0x8040_0000;
        let vm_mem_size: usize = 0x800_0000;
        self.gpm
            .insert(MemoryRegion::new_with_offset_mapper(
                vm_vaddr_start as GuestPhysAddr,
                vm_paddr_start as HostPhysAddr,
                vm_mem_size,
                MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
            ))
            .unwrap();
        debug!("VM stage 2 memory set: {:#x?}", self.gpm);
    }
}
