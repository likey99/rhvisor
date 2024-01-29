use crate::arch::riscv::csr::*;
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
pub fn init() {
    unsafe {
        // Set the trap vector.
        stvec::write(_hyp_trap_vector as usize, TrapMode::Direct);
    }
}
pub fn sync_exception_handler(current_cpu: &mut ArchCpu) {
    //info!("sync_exception_handler");
    trace!("current_cpu: stack{:#x}", current_cpu.stack_top);
    let trap_code: usize;
    trap_code = read_csr!(CSR_SCAUSE);
    info!("CSR_SCAUSE: {:#x}", trap_code);
    if (read_csr!(CSR_HSTATUS) & (1 << 7)) == 0 {
        //HSTATUS_SPV
        error!("exception from HS mode");
        unreachable!();
    }
    match trap_code {
        ExceptionType::ECALL_VU => {
            info!("ECALL_VU");
        }
        ExceptionType::ECALL_VS => {
            info!("ECALL_VS");
            sbi_vs_handler(current_cpu);
        }
        ExceptionType::LOAD_GUEST_PAGE_FAULT => {
            info!("LOAD_GUEST_PAGE_FAULT");
        }
        ExceptionType::STORE_GUEST_PAGE_FAULT => {
            info!("STORE_GUEST_PAGE_FAULT");
        }
        _ => {
            error!("unhandled trap");
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
pub fn interrupts_arch_handle() {
    info!("interrupts_arch_handle");
    let trap_code: usize;
    trap_code = read_csr!(CSR_SCAUSE);
    info!("CSR_SCAUSE: {:#x}", trap_code);
    unreachable!();
}
