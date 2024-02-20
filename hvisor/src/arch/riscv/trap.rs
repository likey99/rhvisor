use super::cpu::ArchCpu;
use super::plic::PLIC;
use super::sbi::sbi_vs_handler;
use crate::arch::riscv::plic::vplic_hart_emul_handler;
use crate::arch::riscv::timer::{get_time, set_next_trigger};
use crate::arch::riscv::{csr::*, trap};
use crate::memory::{GuestPhysAddr, HostPhysAddr};
use crate::percpu;
use core::arch::{asm, global_asm};
use core::time;
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec;
use riscv::register::{hvip, sie};
use riscv_decode::Instruction;
extern "C" {
    fn _hyp_trap_vector();
    fn boot_stack_top();
}
global_asm!(include_str!("trap.S"),
sync_exception_handler=sym sync_exception_handler,
interrupts_arch_handle=sym interrupts_arch_handle);
pub mod ExceptionType {
    pub const ECALL_VU: usize = 8;
    pub const ECALL_VS: usize = 10;
    pub const LOAD_GUEST_PAGE_FAULT: usize = 21;
    pub const STORE_GUEST_PAGE_FAULT: usize = 23;
}

pub mod InterruptType {
    pub const SSI: usize = 1;
    pub const STI: usize = 5;
    pub const SEI: usize = 9;
}
pub fn init() {
    unsafe {
        // Set the trap vector.
        stvec::write(_hyp_trap_vector as usize, TrapMode::Direct);
    }
}
pub fn sync_exception_handler(current_cpu: &mut ArchCpu) {
    trace!("sync_exception_handler");
    trace!("current_cpu: stack{:#x}", current_cpu.stack_top);
    let trap_code = read_csr!(CSR_SCAUSE);
    trace!("CSR_SCAUSE: {:#x}", trap_code);
    if (read_csr!(CSR_HSTATUS) & (1 << 7)) == 0 {
        //HSTATUS_SPV
        error!("exception from HS mode");
        unreachable!();
    }
    let trap_value = read_csr!(CSR_HTVAL);
    trace!("CSR_HTVAL: {:#x}", trap_value);
    let trap_ins = read_csr!(CSR_HTINST);
    trace!("CSR_HTINST: {:#x}", trap_ins);
    let trap_pc = read_csr!(CSR_SEPC);
    trace!("CSR_SEPC: {:#x}", trap_pc);
    trace!("PC{:#x}", current_cpu.sepc);
    match trap_code {
        ExceptionType::ECALL_VU => {
            error!("ECALL_VU");
        }
        ExceptionType::ECALL_VS => {
            trace!("ECALL_VS");
            sbi_vs_handler(current_cpu);
            current_cpu.sepc += 4;
        }
        ExceptionType::LOAD_GUEST_PAGE_FAULT => {
            trace!("LOAD_GUEST_PAGE_FAULT");
            guest_page_fault_handler(current_cpu);
        }
        ExceptionType::STORE_GUEST_PAGE_FAULT => {
            info!("STORE_GUEST_PAGE_FAULT");
            guest_page_fault_handler(current_cpu);
        }
        _ => {
            error!(
                "unhandled trap {:#x},sepc: {:#x}",
                trap_code, current_cpu.sepc
            );
            unreachable!();
        }
    }
}
pub fn guest_page_fault_handler(current_cpu: &mut ArchCpu) {
    let addr: HostPhysAddr = read_csr!(CSR_HTVAL) << 2;
    trace!("guest page fault at {:#x}", addr);
    if addr >= 0x0c00_0000 && addr < 0x1000_0000 {
        let mut inst: u32 = read_csr!(CSR_HTINST) as u32;
        if inst == 0 {
            let inst_addr: GuestPhysAddr = current_cpu.sepc;
            //TODO: load real ins from guest memmory
            inst = read_inst(inst_addr);
            //let inst = unsafe { core::ptr::read(inst_addr as *const usize) };
        } else if inst == 0x3020 || inst == 0x3000 {
            // TODO: we should reinject this in the guest as a fault access
            error!("fault on 1st stage page table walk");
        } else {
            // If htinst is valid and is not a pseudo instructon make sure
            // the opcode is valid even if it was a compressed instruction,
            // but before save the real instruction size.
            error!("unhandled guest page fault at {:#x}", addr);
        }
        //TODO: decode inst to real instruction
        let (len, inst) = decode_inst(inst);
        if let Some(inst) = inst {
            vplic_hart_emul_handler(current_cpu, addr, inst);
            current_cpu.sepc += len;
        } else {
            error!("Invalid instruction at {:#x}", current_cpu.sepc);
        }
    } else {
        panic!("unmaped memmory");
    }
}
fn read_inst(addr: GuestPhysAddr) -> u32 {
    let mut ins: u32 = 0;
    if addr & 0b1 != 0 {
        error!("trying to read guest unaligned instruction");
    }
    //
    //  Read 16 bits at a time to make sure the access is aligned. If the instruction is not
    //  compressed, read the following 16-bits.
    //
    ins = hlvxhu(addr) as u32;
    if (ins & 0b11) == 3 {
        ins |= (hlvxhu(addr + 2) as u32) << 16;
    }
    ins
}
fn hlvxhu(addr: GuestPhysAddr) -> u64 {
    let mut value: u64;
    unsafe {
        asm!(
            ".insn r 0x73, 0x4, 0x32, {0}, {1}, x3",
            out(reg) value,
            in(reg) addr,
        );
    }
    value
}
/// decode risc-v instruction, return (inst len, inst)
fn decode_inst(inst: u32) -> (usize, Option<Instruction>) {
    let i1 = inst as u16;
    let len = riscv_decode::instruction_length(i1);
    let inst = match len {
        2 => i1 as u32,
        4 => inst as u32,
        _ => unreachable!(),
    };
    (len, riscv_decode::decode(inst).ok())
}
static mut time_irq: usize = 0;
pub fn interrupts_arch_handle(current_cpu: &mut ArchCpu) {
    trace!("interrupts_arch_handle");
    let trap_code: usize;
    trap_code = read_csr!(CSR_SCAUSE);
    trace!("CSR_SCAUSE: {:#x}", trap_code);
    match trap_code & 0xfff {
        InterruptType::STI => {
            trace!("STI");
            // write_csr!(CSR_HVIP, 1 << 6); //inject VSTIP
            unsafe {
                hvip::set_vstip();
                sie::clear_stimer();
                // time_irq += 1;
                // if (time_irq == 100) {
                //     warn!("trigger a external irq");
                //     handle_irq(current_cpu);
                // }
            }
            // write_csr!(CSR_SIE, 1 << 9 | 1 << 1); // clear the timer interrupt pending bit
            trace!("sip{:#x}", read_csr!(CSR_SIP));
            trace!("sie {:#x}", read_csr!(CSR_SIE));
        }
        InterruptType::SSI => {
            panic!("SSI");
        }
        InterruptType::SEI => {
            info!("SEI");
            handle_irq(current_cpu)
        }
        _ => {
            error!(
                "unhandled trap {:#x},sepc: {:#x}",
                trap_code, current_cpu.sepc
            );
            unreachable!();
        }
    }
}

/// handle interrupt request(current only external interrupt)
pub fn handle_irq(current_cpu: &mut ArchCpu) {
    // TODO: handle other irq
    // check external interrupt && handle
    //let host_plic = host_vmm.host_plic.as_mut().unwrap();
    // get current guest context id
    let context_id = 2 * 0 + 1;
    let claim_and_complete_addr = 0x0c00_0000 + 0x0020_0004 + 0x1000 * context_id;
    let mut irq = unsafe { core::ptr::read(claim_and_complete_addr as *const u32) };
    // unsafe {
    //     if (time_irq == 100) {
    //         irq = 10;
    //     }
    // }
    info!("get irq{}@{:#x}", irq, claim_and_complete_addr);
    let mut host_plic = PLIC.get().expect("Uninitialized hypervisor plic!").write();
    host_plic.claim_complete[context_id] = irq;
    drop(host_plic);
    //host_plic.claim_complete[context_id] = irq;

    // set external interrupt pending, which trigger guest interrupt
    unsafe { hvip::set_vseip() };

    // set irq pending in host vmm
    //host_vmm.irq_pending = true;
}
