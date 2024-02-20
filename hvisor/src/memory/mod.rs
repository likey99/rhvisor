pub mod addr;
pub mod frame;
pub mod heap;
mod mapper;
mod mm;
mod paging;

use crate::arch::riscv::s1pt::Stage1PageTable;
use crate::consts::{hv_end, HV_BASE, HV_PHY_BASE};
use crate::error::HvResult;
use aarch64_cpu::registers::SCTLR_EL3::M;
pub use addr::{
    virt_to_phys, GuestPhysAddr, GuestVirtAddr, HostPhysAddr, HostVirtAddr, PhysAddr, VirtAddr,
    PHYS_VIRT_OFFSET,
};
use core::ops::{Deref, DerefMut};

use bitflags::bitflags;
use spin::{Once, RwLock};

use crate::memory::addr::align_up;
pub use frame::Frame;
pub use mm::{MemoryRegion, MemorySet, PARKING_INST_PAGE};
pub use paging::{
    npages, GenericPageTable, GenericPageTableImmut, Level3PageTable, Level3PageTableImmut,
};
pub use paging::{GenericPTE, PagingInstr};
pub const PAGE_SIZE: usize = paging::PageSize::Size4K as usize;
pub const TEMPORARY_MAPPING_BASE: usize = 0x80_0000_0000;
pub const NUM_TEMPORARY_PAGES: usize = 16;
#[repr(align(4096))]
pub struct AlignedPage([u8; PAGE_SIZE]);

impl AlignedPage {
    pub const fn new() -> Self {
        Self([0; PAGE_SIZE])
    }
}

impl Deref for AlignedPage {
    type Target = [u8; PAGE_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AlignedPage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct MemFlags: u64 {
        const READ          = 1 << 0;
        const WRITE         = 1 << 1;
        const EXECUTE       = 1 << 2;
        const DMA           = 1 << 3;
        const IO            = 1 << 4;
        const COMMUNICATION = 1 << 5;
        const LOADABLE      = 1 << 6;
        const ROOTSHARED    = 1 << 7;
        const NO_HUGEPAGES  = 1 << 8;
        const USER          = 1 << 9;
    }
}

/// Page table used for hypervisor.
static HV_PT: Once<RwLock<MemorySet<Stage1PageTable>>> = Once::new();

pub fn hv_page_table<'a>() -> &'a RwLock<MemorySet<Stage1PageTable>> {
    HV_PT.get().expect("Uninitialized hypervisor page table!")
}
pub fn init_heap() {
    heap::init();
}
pub fn init_frame_allocator() {
    frame::init();
}
pub fn init_hv_page_table(fdt: fdt::Fdt) -> HvResult {
    let mut hv_pt: MemorySet<Stage1PageTable> = MemorySet::new();
    // let _ = hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     0x8000_0000 as HostVirtAddr,
    //     hv_phys_start as HostPhysAddr,
    //     (hv_phys_end - hv_phys_start) as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    // ));
    trace!("fdt: {:?}", fdt);
    // The first memory region is used to map the guest physical memory.
    let mem_region = fdt.memory().regions().next().unwrap();
    debug!("map mem_region: {:?}", mem_region);
    hv_pt.insert(MemoryRegion::new_with_offset_mapper(
        mem_region.starting_address as GuestPhysAddr,
        mem_region.starting_address as HostPhysAddr,
        mem_region.size.unwrap(),
        MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    ))?;
    // probe virtio mmio device
    for node in fdt.find_all_nodes("/soc/virtio_mmio") {
        if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
            let paddr = reg.starting_address as HostPhysAddr;
            let size = reg.size.unwrap();
            debug!("map virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
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
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
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
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
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
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
        }
    }

    // probe plic
    for node in fdt.find_all_nodes("/soc/plic") {
        if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
            let paddr = reg.starting_address as HostPhysAddr;
            let size = reg.size.unwrap();
            debug!("map plic addr: {:#x}, size: {:#x}", paddr, size);
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
        }
    }

    for node in fdt.find_all_nodes("/soc/pci") {
        if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
            let paddr = reg.starting_address as HostPhysAddr;
            let size = reg.size.unwrap();
            debug!("map pci addr: {:#x}, size: {:#x}", paddr, size);
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
        }
    }
    info!("Hypervisor page table init end.");
    debug!("Hypervisor virtual memory set: {:#x?}", hv_pt);

    HV_PT.call_once(|| RwLock::new(hv_pt));

    Ok(())
}
