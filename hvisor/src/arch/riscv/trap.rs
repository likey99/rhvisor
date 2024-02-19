use super::cpu::ArchCpu;
use super::sbi::sbi_vs_handler;
use crate::arch::riscv::timer::{get_time, set_next_trigger};
use crate::arch::riscv::{csr::*, trap};
use crate::memory::HostPhysAddr;
use crate::percpu;
use core::arch::{asm, global_asm};
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec;
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
    let trap_code: usize;
    trap_code = read_csr!(CSR_SCAUSE);
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
        }
        ExceptionType::LOAD_GUEST_PAGE_FAULT => {
            info!("LOAD_GUEST_PAGE_FAULT");
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
    info!("guest page fault at {:#x}", addr);
    unreachable!();
}
pub fn interrupts_arch_handle(current_cpu: &mut ArchCpu) {
    trace!("interrupts_arch_handle");

    let trap_code: usize;
    trap_code = read_csr!(CSR_SCAUSE);
    trace!("CSR_SCAUSE: {:#x}", trap_code);
    match trap_code & 0xfff {
        InterruptType::STI => {
            trace!("STI");
            write_csr!(CSR_HVIP, 1 << 6); //inject VSTIP
            let sip: usize = read_csr!(CSR_SIP);
            debug!("sip: {:#x}", sip);
            write_csr!(CSR_SIE, 1 << 9 | 1 << 1); // clear the timer interrupt pending bit
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
