use crate::arch::riscv::csr::*;
use crate::plat::qemu_riscv64_virt::{PLIC_GLOBAL_SIZE, PLIC_MAX_CONTEXT, PLIC_TOTAL_SIZE};
use crate::{cpu::ArchCpu, memory::GuestPhysAddr};
use riscv::register::{hvip, sie};
use riscv_decode::Instruction;
use spin::{Once, RwLock};
/// Plic used for Hypervisor.
pub static PLIC: Once<RwLock<Plic>> = Once::new();

pub fn host_plic<'a>() -> &'a RwLock<Plic> {
    PLIC.get().expect("Uninitialized hypervisor plic!")
}
pub fn init_plic(plic_base: usize, plic_size: usize) {
    let plic = Plic::new(plic_base, plic_size);
    PLIC.call_once(|| RwLock::new(plic));
}
pub struct Plic {
    pub base: usize,
    pub size: usize,
    pub claim_complete: [u32; PLIC_MAX_CONTEXT],
}
impl Plic {
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
            claim_complete: [0u32; PLIC_MAX_CONTEXT],
        }
    }
}
pub fn vplic_global_emul_handler(
    current_cpu: &mut ArchCpu,
    addr: GuestPhysAddr,
    inst: Instruction,
) {
    //TODO:check irq id for vm
    let host_plic = host_plic();
    let offset = addr.wrapping_sub(host_plic.read().base);
    // priority/pending/enable
    if offset >= 0x000000 && offset < 0x002000 {
        // priority
        match inst {
            Instruction::Sw(i) => {
                // guest write irq priority
                //TODO:check irq id for vm
                let irq_id = offset / 4;
                let value = current_cpu.x[i.rs2() as usize] as u32;
                debug!("PLIC write addr@{:#x} -> {:#x}", addr, value);
                unsafe {
                    core::ptr::write_volatile(addr as *mut u32, value);
                }
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= 0x002000 && offset < 0x004000 {
        //enable
        match inst {
            Instruction::Lw(i) => {
                // guest read
                let value = unsafe { core::ptr::read_volatile(addr as *const u32) };
                debug!("PLIC read addr@{:#x} -> {:#x}", addr, value);
                current_cpu.x[i.rd() as usize] = value as usize;
            }
            Instruction::Sw(i) => {
                // guest write irq enable
                let context = (offset - 0x002000) / 0x80;
                let irq_id = (offset - 0x002000) % 0x80;
                let value = current_cpu.x[i.rs2() as usize] as u32;
                debug!("PLIC write addr@{:#x} -> {:#x}", addr, value);
                unsafe {
                    core::ptr::write_volatile(addr as *mut u32, value);
                }
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    }
}
pub fn vplic_hart_emul_handler(current_cpu: &mut ArchCpu, addr: GuestPhysAddr, inst: Instruction) {
    trace!("handle PLIC access addr@{:#x}", addr);
    let host_plic = host_plic();
    let offset = addr.wrapping_sub(host_plic.read().base);
    // threshold/claim/complete
    if offset >= PLIC_GLOBAL_SIZE && offset < PLIC_TOTAL_SIZE {
        let context = (offset - PLIC_GLOBAL_SIZE) / 0x1000;
        let index = ((offset - PLIC_GLOBAL_SIZE) & 0xfff) >> 2;
        if index == 0 {
            // threshold
            match inst {
                Instruction::Sw(i) => {
                    // guest write threshold register to plic core
                    let value = current_cpu.x[i.rs2() as usize] as u32;
                    debug!("PLIC write addr@{:#x} -> {:#x}", addr, value);
                    // todo: guest pa -> host pa
                    // htracking!(
                    //     "write PLIC threshold reg, addr: {:#x}, value: {:#x}",
                    //     guest_pa,
                    //     value
                    // );
                    unsafe {
                        core::ptr::write_volatile(addr as *mut u32, value);
                    }
                }
                _ => panic!("Unexpected instruction threshold {:?}", inst),
            }
        } else if index == 1 {
            // claim/complete
            // htracking!("claim/complete");
            match inst {
                Instruction::Lw(i) => {
                    // guest read claim from plic core
                    debug!(
                        "PLIC read addr@{:#x} -> {:#x}",
                        addr,
                        host_plic.read().claim_complete[context]
                    );
                    current_cpu.x[i.rd() as usize] =
                        host_plic.read().claim_complete[context] as usize;
                }
                Instruction::Sw(i) => {
                    // guest write complete to plic core
                    let value = current_cpu.x[i.rs2() as usize] as u32;
                    debug!("Write plic addr@:{:#x} -> {:#x}", addr, value);
                    // todo: guest pa -> host pa
                    unsafe {
                        core::ptr::write_volatile(addr as *mut u32, value);
                    }
                    host_plic.write().claim_complete[context] = 0;
                    unsafe {
                        hvip::clear_vseip();
                    }
                }
                _ => panic!("Unexpected instruction claim/complete {:?}", inst),
            }
        }
    } else {
        panic!("Invalid address: {:#x}", addr);
    }
}
