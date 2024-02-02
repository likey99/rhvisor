use crate::arch::riscv::{csr::*, trap};
use crate::percpu;
use core::arch::{asm, global_asm};
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec;

use super::cpu::ArchCpu;
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
    //trace!("sync_exception_handler");
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
            error!("LOAD_GUEST_PAGE_FAULT");
            unreachable!();
        }
        ExceptionType::STORE_GUEST_PAGE_FAULT => {
            error!("STORE_GUEST_PAGE_FAULT");
            unreachable!();
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
pub fn sbi_vs_handler(current_cpu: &mut ArchCpu) {
    let ret = sbi_call_5(
        current_cpu.x[17],
        current_cpu.x[16],
        current_cpu.x[10],
        current_cpu.x[11],
        current_cpu.x[12],
        current_cpu.x[13],
        current_cpu.x[14],
    );
    current_cpu.sepc += 4;
    current_cpu.x[10] = ret.0;
    current_cpu.x[11] = ret.1;
    trace!("sbi_call_5: error:{:#x}, value:{:#x}", ret.0, ret.1);
}
pub fn sbi_call_5(
    eid: usize,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> (usize, usize) {
    trace!("sbi_call_5: eid:{:#x}, fid:{:#x}", eid, fid);
    let (error, value);
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
        );
    }
    (error, value)
}
pub fn interrupts_arch_handle(current_cpu: &mut ArchCpu) {
    trace!("interrupts_arch_handle");
    let trap_code: usize;
    trap_code = read_csr!(CSR_SCAUSE);
    trace!("CSR_SCAUSE: {:#x}", trap_code);
    match trap_code & 0xff {
        InterruptType::STI => {
            trace!("STI");
            write_csr!(CSR_HVIP, 1 << 6); //VSTIP
            let mut sip: usize = read_csr!(CSR_SIP);
            sip &= !(1 << 5);
            write_csr!(CSR_SIP, sip); //clear STIP
            debug!("STI");
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
