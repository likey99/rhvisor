use crate::arch::riscv::s2pt::Stage2PageTable;
use crate::consts::MAX_CPU_NUM;
use crate::error::HvResult;
use crate::memory::addr::align_up;
use crate::memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet};
use crate::percpu::get_cpu_data;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::char::{decode_utf16, MAX};
use core::mem::{self};
use spin::RwLock;
static ZONE_LIST: RwLock<Vec<Arc<RwLock<Zone>>>> = RwLock::new(vec![]);
/// Add cell to ZONE_LIST
pub fn add_zone(zone: Arc<RwLock<Zone>>) {
    ZONE_LIST.write().push(zone);
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CpuSet {
    max_cpu_id: usize,
    bitmap: usize,
}

impl CpuSet {
    pub fn new(max_cpu_id: usize, bitmap: usize) -> Self {
        Self { max_cpu_id, bitmap }
    }
    pub fn from_cpuset_slice(cpu_set: &[u8]) -> Self {
        if cpu_set.len() != 8 {
            todo!("Cpu_set should be 8 bytes!");
        }
        let cpu_set_long: usize = cpu_set
            .iter()
            .enumerate()
            .fold(0, |acc, (i, x)| acc | (*x as usize) << (i * 8));
        Self::new(cpu_set.len() as usize * 8 - 1, cpu_set_long)
    }
    #[allow(unused)]
    pub fn set_bit(&mut self, id: usize) {
        assert!(id <= self.max_cpu_id);
        self.bitmap |= 1 << id;
    }
    pub fn clear_bit(&mut self, id: usize) {
        assert!(id <= self.max_cpu_id);
        self.bitmap &= !(1 << id);
    }
    pub fn contains_cpu(&self, id: usize) -> bool {
        id <= self.max_cpu_id && (self.bitmap & (1 << id)) != 0
    }
    #[allow(unused)]
    pub fn first_cpu(&self) -> Option<usize> {
        (0..=self.max_cpu_id).find(move |&i| self.contains_cpu(i))
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        (0..=self.max_cpu_id).filter(move |&i| self.contains_cpu(i))
    }
    pub fn iter_except<'a>(&'a self, id: usize) -> impl Iterator<Item = usize> + 'a {
        (0..=self.max_cpu_id).filter(move |&i| self.contains_cpu(i) && i != id)
    }
}

pub struct Zone {
    pub vmid: usize,
    pub gpm: MemorySet<Stage2PageTable>,
    pub cpu_set: CpuSet,
}
impl Zone {
    pub fn new(vmid: usize) -> Self {
        Self {
            vmid,
            gpm: MemorySet::new(),
            cpu_set: CpuSet::new(MAX_CPU_NUM as usize, 0),
        }
    }
    pub fn pt_init(
        &mut self,
        vm_paddr_start: usize,
        fdt: fdt::Fdt,
        guest_dtb: usize,
        dtb_addr: usize,
    ) -> HvResult {
        //debug!("fdt: {:?}", fdt);
        // The first memory region is used to map the guest physical memory.
        let mem_region = fdt.memory().regions().next().unwrap();
        info!("map mem_region: {:?}", mem_region);
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            mem_region.starting_address as GuestPhysAddr,
            vm_paddr_start as HostPhysAddr,
            mem_region.size.unwrap(),
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        ))?;
        // map guest dtb
        info!("map guest dtb: {:#x}", dtb_addr);
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
                info!("map virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
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
                info!("map test addr: {:#x}, size: {:#x}", paddr, size);
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
                info!("map uart addr: {:#x}, size: {:#x}", paddr, size);
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
                info!("map clint addr: {:#x}, size: {:#x}", paddr, size);
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
                println!("map pci addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }
    pub fn gpm_activate(&self) {
        unsafe { self.gpm.activate() }
    }
}
pub fn zone_create(
    vmid: usize,
    vm_paddr_start: usize,
    dtb_ptr: *const u8,
    dtb_addr: usize,
) -> HvResult<Arc<RwLock<Zone>>> {
    // we create the new zone here
    //TODO: create Zone with cpu_set
    let guest_fdt = unsafe { fdt::Fdt::from_ptr(dtb_ptr) }.unwrap();
    let guest_entry = guest_fdt
        .memory()
        .regions()
        .next()
        .unwrap()
        .starting_address as usize;
    let mut zone = Zone::new(vmid);
    zone.pt_init(vm_paddr_start, guest_fdt, dtb_ptr as usize, dtb_addr)
        .unwrap();
    guest_fdt.cpus().for_each(|cpu| {
        let cpu_id = cpu.ids().all().next().unwrap();
        zone.cpu_set.set_bit(cpu_id as usize);
    });
    //TODO:assign cpu according to cpu_set
    //TODO:set cpu entry
    info!("zone cpu_set: {:#b}", zone.cpu_set.bitmap);
    let cpu_set = zone.cpu_set;

    let new_zone_pointer = Arc::new(RwLock::new(zone));
    {
        cpu_set.iter().for_each(|cpuid| {
            let cpu_data = get_cpu_data(cpuid);
            cpu_data.zone = Some(new_zone_pointer.clone());
            //chose boot cpu
            if cpuid == cpu_set.first_cpu().unwrap() {
                cpu_data.boot_cpu = true;
            }
            info!("set cpu{} first_cpu{}", cpuid, cpu_set.first_cpu().unwrap());
            cpu_data.arch_cpu.first_cpu = cpu_set.first_cpu().unwrap();
            cpu_data.cpu_on_entry = guest_entry;
        });
    }
    {}
    add_zone(new_zone_pointer.clone());

    Ok(new_zone_pointer)
}
