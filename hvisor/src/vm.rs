use core::char::decode_utf16;
use core::mem::{self};

use crate::arch::riscv::s2pt::Stage2PageTable;
use crate::error::HvResult;
use crate::memory::addr::align_up;
use crate::memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet};
use crate::plat::qemu_riscv64_virt::*;
use crate::GUEST_DTB;
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
    pub fn pt_init(&mut self, vm_paddr_start: usize, fdt: fdt::Fdt, dtb_addr: usize) -> HvResult {
        trace!("fdt: {:?}", fdt);
        // The first memory region is used to map the guest physical memory.
        let mem_region = fdt.memory().regions().next().unwrap();
        debug!("map mem_region: {:?}", mem_region);
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            mem_region.starting_address as GuestPhysAddr,
            vm_paddr_start as HostPhysAddr,
            mem_region.size.unwrap(),
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        ))?;
        // map guest dtb
        let guest_dtb = GUEST_DTB.as_ptr() as usize;
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            dtb_addr as GuestPhysAddr,
            guest_dtb as HostPhysAddr,
            align_up(fdt.total_size()),
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        ))?;
        // probe virtio mmio device
        for node in fdt.find_all_nodes("/soc/virtio_mmio") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap();
                debug!("map virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        // probe virt test
        for node in fdt.find_all_nodes("/soc/test") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap() + 0x1000;
                debug!("map test addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
                ))?;
            }
        }

        // probe uart device
        for node in fdt.find_all_nodes("/soc/uart") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = align_up(reg.size.unwrap());
                debug!("map uart addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        // probe clint(core local interrupter)
        for node in fdt.find_all_nodes("/soc/clint") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap();
                debug!("map clint addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        // probe plic
        //TODO: remove plic map from vm
        // for node in fdt.find_all_nodes("/soc/plic") {
        //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
        //         let paddr = reg.starting_address as HostPhysAddr;
        //         //let size = reg.size.unwrap();
        //         let size = PLIC_GLOBAL_SIZE; //
        //         debug!("map plic addr: {:#x}, size: {:#x}", paddr, size);
        //         self.gpm.insert(MemoryRegion::new_with_offset_mapper(
        //             paddr as GuestPhysAddr,
        //             paddr,
        //             size,
        //             MemFlags::READ | MemFlags::WRITE,
        //         ))?;
        //     }
        // }

        for node in fdt.find_all_nodes("/soc/pci") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap();
                debug!("map pci addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }
        debug!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }
}
