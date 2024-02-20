use crate::arch::riscv::csr::*;
use crate::plat::qemu_riscv64_virt::{PLIC_GLOBAL_SIZE, PLIC_TOTAL_SIZE};
use crate::{cpu::ArchCpu, memory::GuestPhysAddr};
use riscv::register::{hvip, sie};
use riscv_decode::Instruction;
use spin::{Once, RwLock};
/// Plic used for Hypervisor.
pub static PLIC: Once<RwLock<Plic>> = Once::new();

pub fn hv_plic<'a>() -> &'a RwLock<Plic> {
    PLIC.get().expect("Uninitialized hypervisor plic!")
}
pub fn init_plic(plic_base: usize) {
    let plic = Plic::new(plic_base);
    PLIC.call_once(|| RwLock::new(plic));
}
pub struct Plic {
    base: usize,
    pub claim_complete: [u32; 32],
}
impl Plic {
    pub fn new(base: usize) -> Self {
        Self {
            base,
            claim_complete: [0u32; 32],
        }
    }
}
// struct Vplic {
//     plic: Plic,
//     vcpu: usize,
// }
// impl Vplic {
//     pub fn new(base: usize, vcpu: usize) -> Self {
//         Self {
//             plic: Plic::new(base),
//             vcpu,
//         }
//     }
//     pub fn set_threshold(&mut self, target: usize, threshold: usize) {
//         self.plic.set_threshold(target, threshold);
//     }
//     pub fn set_enable(&mut self, irq: usize, enable: bool) {
//         self.plic.set_enable(irq, enable);
//     }
//     pub fn set_priority(&mut self, irq: usize, priority: usize) {
//         self.plic.set_priority(irq, priority);
//     }
//     pub fn claim(&mut self) -> usize {
//         self.plic.claim()
//     }
//     pub fn complete(&mut self, irq: usize) {
//         self.plic.complete(irq);
//     }
// }
pub fn vplic_hart_emul_handler(current_cpu: &mut ArchCpu, addr: GuestPhysAddr, inst: Instruction) {
    trace!("handle PLIC access addr@{:#x}", addr);
    let host_plic = PLIC.get().expect("Uninitialized hypervisor plic!").read();
    let offset = addr.wrapping_sub(host_plic.base);
    drop(host_plic);
    // threshold/claim/complete
    if offset >= PLIC_GLOBAL_SIZE && offset < PLIC_TOTAL_SIZE {
        let hart = (offset - PLIC_GLOBAL_SIZE) / 0x1000;
        let index = ((offset - PLIC_GLOBAL_SIZE) & 0xfff) >> 2;
        if index == 0 {
            // threshold
            //inst lw=>true   sw=>false
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
                    let host_plic = PLIC.get().expect("Uninitialized hypervisor plic!").read();
                    // guest read claim from plic core
                    debug!(
                        "PLIC read addr@{:#x} -> {:#x}",
                        addr, host_plic.claim_complete[hart]
                    );
                    current_cpu.x[i.rd() as usize] = host_plic.claim_complete[hart] as usize;
                    drop(host_plic);
                }
                Instruction::Sw(i) => {
                    // guest write complete to plic core
                    let value = current_cpu.x[i.rs2() as usize] as u32;
                    debug!("Write plic addr@:{:#x} -> {:#x}", addr, value);
                    // todo: guest pa -> host pa
                    unsafe {
                        core::ptr::write_volatile(addr as *mut u32, value);
                    }
                    let mut host_plic = PLIC.get().expect("Uninitialized hypervisor plic!").write();
                    host_plic.claim_complete[hart] = 0;
                    drop(host_plic);
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
