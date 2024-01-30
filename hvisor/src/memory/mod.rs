pub mod addr;
pub mod frame;
pub mod heap;
//mod mapper;
//mod mm;
//mod paging;

use crate::{
    consts::{hv_end, HV_BASE, HV_PHY_BASE, PAGE_SIZE},
    memory::addr::{virt_to_phys, PhysAddr, VirtAddr},
};
pub fn init_heap() {
    heap::init();
}
pub fn init_frame_allocator() {
    frame::init();
}

pub fn init_hv_page_table() -> Result<(), usize> {
    let hv_phys_start: PhysAddr = HV_PHY_BASE;
    let hv_phys_size: VirtAddr = virt_to_phys(hv_end());
    // let mut hv_pt: MemorySet<Stage1PageTable> = MemorySet::new();

    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     HV_BASE as GuestPhysAddr,
    //     hv_phys_start as HostPhysAddr,
    //     hv_phys_size as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::NO_HUGEPAGES,
    // ))?;

    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     trampoline_page as GuestPhysAddr,
    //     trampoline_page as HostPhysAddr,
    //     PAGE_SIZE as usize,
    //     MemFlags::READ | MemFlags::WRITE,
    // ))?;

    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     UART_BASE_VIRT,
    //     sys_config.debug_console.address as PhysAddr,
    //     sys_config.debug_console.size as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    // ))?;

    // // add gicd memory map
    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     gicd_base as GuestPhysAddr,
    //     gicd_base as HostPhysAddr,
    //     GICD_SIZE as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    // ))?;
    // //add gicr memory map
    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     gicr_base as GuestPhysAddr,
    //     gicr_base as HostPhysAddr,
    //     gicr_size as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    // ))?;
    // // Map pci region. Jailhouse doesn't map pci region to el2.
    // // Now we simplify the complex pci handler and just map it.
    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     mmcfg_start as GuestPhysAddr,
    //     mmcfg_start as HostPhysAddr,
    //     mmcfg_size as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    // ))?;

    // // add virtio map
    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     0xa000000 as GuestPhysAddr,
    //     0xa000000 as HostPhysAddr,
    //     0x4000 as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    // ))?;

    // // add virt gic its
    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     0x8080000 as GuestPhysAddr,
    //     0x8080000 as HostPhysAddr,
    //     0x20000 as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    // ))?;

    // info!("Hypervisor page table init end.");
    // debug!("Hypervisor virtual memory set: {:#x?}", hv_pt);

    // HV_PT.call_once(|| RwLock::new(hv_pt));

    Ok(())
}
