use core::ops::Add;

use crate::arch::riscv::csr::*;
use crate::{cpu::ArchCpu, memory::GuestPhysAddr};
use aarch64_cpu::registers::CCSIDR_EL1::AssociativityWithCCIDX::Value;
use riscv::register::{hvip, sie};
use riscv_decode::Instruction;
use spin::{Once, RwLock};
// PLIC Memory Map
//  base + 0x000000: Reserved (interrupt source 0 does not exist)
//  base + 0x000004: Interrupt source 1 priority
//  base + 0x000008: Interrupt source 2 priority
//  ...
//  base + 0x000FFC: Interrupt source 1023 priority
//  base + 0x001000: Interrupt Pending bit 0-31
//  base + 0x00107C: Interrupt Pending bit 992-1023
//  ...
//  base + 0x002000: Enable bits for sources 0-31 on context 0
//  base + 0x002004: Enable bits for sources 32-63 on context 0
//  ...
//  base + 0x00207C: Enable bits for sources 992-1023 on context 0
//  base + 0x002080: Enable bits for sources 0-31 on context 1
//  base + 0x002084: Enable bits for sources 32-63 on context 1
//  ...
//  base + 0x0020FC: Enable bits for sources 992-1023 on context 1
//  base + 0x002100: Enable bits for sources 0-31 on context 2
//  base + 0x002104: Enable bits for sources 32-63 on context 2
//  ...
//  base + 0x00217C: Enable bits for sources 992-1023 on context 2
//  ...
//  base + 0x1F1F80: Enable bits for sources 0-31 on context 15871
//  base + 0x1F1F84: Enable bits for sources 32-63 on context 15871
//  base + 0x1F1FFC: Enable bits for sources 992-1023 on context 15871
//  ...
//  base + 0x1FFFFC: Reserved
//  base + 0x200000: Priority threshold for context 0
//  base + 0x200004: Claim/complete for context 0
//  base + 0x200008: Reserved
//  ...
//  base + 0x200FFC: Reserved
//  base + 0x201000: Priority threshold for context 1
//  base + 0x201004: Claim/complete for context 1
//  ...
//  base + 0x3FFF000: Priority threshold for context 15871
//  base + 0x3FFF004: Claim/complete for context 15871
//  base + 0x3FFF008: Reserved
//  ...
//  base + 0x3FFFFFC: Reserved
/// Plic used for Hypervisor.
pub const PLIC_PRIORITY_BASE: usize = 0x0000;
pub const PLIC_PENDING_BASE: usize = 0x1000;
pub const PLIC_ENABLE_BASE: usize = 0x2000;
pub const PLIC_GLOBAL_SIZE: usize = 0x200000;
pub const PLIC_TOTAL_SIZE: usize = 0x400000;
pub const PLIC_MAX_CONTEXT: usize = 64;
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
    pub fn set_priority(&self, irq_id: usize, priority: u32) {
        let addr = self.base + PLIC_PRIORITY_BASE + irq_id * 4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, priority);
        }
    }
    pub fn read_enable(&self, context: usize, irq_base: usize) -> u32 {
        let addr = self.base + PLIC_ENABLE_BASE + context * 0x80 + irq_base;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn set_enable(&self, context: usize, irq_base: usize, value: u32) {
        let addr = self.base + PLIC_ENABLE_BASE + context * 0x80 + irq_base;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }
    pub fn set_threshold(&self, context: usize, value: u32) {
        let addr = self.base + PLIC_GLOBAL_SIZE + context * 0x1000;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }
    ///TODO:move to vplic
    pub fn emul_claim(&self, context: usize) -> u32 {
        self.claim_complete[context]
    }
    pub fn emul_complete(&mut self, context: usize, irq_id: u32) {
        let addr = self.base + PLIC_GLOBAL_SIZE + 0x1000 * context + 0x4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, irq_id as u32);
        }

        self.claim_complete[context] = 0;
        unsafe {
            hvip::clear_vseip();
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
    if offset >= PLIC_PRIORITY_BASE && offset < PLIC_ENABLE_BASE {
        // priority/pending
        match inst {
            Instruction::Sw(i) => {
                // guest write irq priority
                //TODO:check irq id for vm
                let irq_id = offset / 4;
                let value = current_cpu.x[i.rs2() as usize] as u32;
                host_plic.write().set_priority(irq_id, value);
                info!(
                    "PLIC set priority write addr@{:#x} irq id {} valuse{:#x}",
                    addr, irq_id, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= PLIC_ENABLE_BASE && offset < PLIC_GLOBAL_SIZE {
        //enable
        match inst {
            Instruction::Lw(i) => {
                // guest read
                let vcontext = (offset - 0x002000) / 0x80;
                let context = vcontext + current_cpu.first_cpu * 2;
                let irq_base = (offset - 0x002000) % 0x80;
                let value = host_plic.read().read_enable(context, irq_base);
                current_cpu.x[i.rd() as usize] = value as usize;
                info!(
                    "PLIC set enable read addr@{:#x} -> context {}=>{}  irq_base {}~{} value {:#x}",
                    addr,
                    vcontext,
                    context,
                    irq_base * 8,
                    irq_base * 8 + 31,
                    value
                );
            }
            Instruction::Sw(i) => {
                // guest write irq enable
                let vcontext = (offset - 0x002000) / 0x80;
                let context = vcontext + current_cpu.first_cpu * 2;
                let irq_base = (offset - 0x002000) % 0x80;
                let value = current_cpu.x[i.rs2() as usize] as u32;
                host_plic.write().set_enable(context, irq_base, value);

                info!(
                    "PLIC set enable write addr@{:#x} -> context{}=>{}  irq_base {}~{} value {:#x}",
                    addr,
                    vcontext,
                    context,
                    irq_base * 8,
                    irq_base * 8 + 31,
                    value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else {
        panic!("Invalid address: {:#x}", addr);
    }
}
pub fn vplic_hart_emul_handler(current_cpu: &mut ArchCpu, addr: GuestPhysAddr, inst: Instruction) {
    trace!("handle PLIC access addr@{:#x}", addr);
    let host_plic = host_plic();
    let offset = addr.wrapping_sub(host_plic.read().base);
    // threshold/claim/complete
    if offset >= PLIC_GLOBAL_SIZE && offset < PLIC_TOTAL_SIZE {
        let vcontext = (offset - PLIC_GLOBAL_SIZE) / 0x1000;
        let context = vcontext + current_cpu.first_cpu * 2;
        let index = ((offset - PLIC_GLOBAL_SIZE) & 0xfff);
        if index == 0 {
            // threshold
            match inst {
                Instruction::Sw(i) => {
                    // guest write threshold register to plic core
                    let value = current_cpu.x[i.rs2() as usize] as u32;
                    host_plic.write().set_threshold(context, value);
                    info!(
                        "PLIC set threshold write addr@{:#x} context{} -> {:#x}",
                        addr, context, value
                    );
                }
                _ => panic!("Unexpected instruction threshold {:?}", inst),
            }
        } else if index == 0x4 {
            // claim/complete
            // htracking!("claim/complete");
            match inst {
                Instruction::Lw(i) => {
                    // guest read claim from plic core
                    current_cpu.x[i.rd() as usize] = host_plic.read().emul_claim(context) as usize;
                    debug!(
                        "PLIC claim read addr@{:#x} context{} -> {:#x}",
                        addr,
                        context,
                        host_plic.read().claim_complete[context]
                    );
                }
                Instruction::Sw(i) => {
                    // guest write complete to plic core
                    let value = current_cpu.x[i.rs2() as usize] as u32;
                    host_plic.write().emul_complete(context, value);
                    // todo: guest pa -> host pa
                    debug!(
                        "PLIC complete write addr@:{:#x} context {} -> {:#x}",
                        addr, context, value
                    );
                }
                _ => panic!("Unexpected instruction claim/complete {:?}", inst),
            }
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    } else {
        panic!("Invalid address: {:#x}", addr);
    }
}
