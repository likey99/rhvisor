//! SBI call wrappers

#![allow(unused)]
use super::cpu::ArchCpu;
use crate::arch::riscv::csr::*;
pub mod SBI_EID {
    pub const SET_TIMER: usize = 0x54494D45;
}
/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

/// use sbi call to set timer
pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
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
    if eid == 0x54494D45 {
        debug!("VS set timer");
        write_csr!(CSR_HVIP, 0); //VSTIP
        write_csr!(CSR_SIE, 1 << 9 | 1 << 5 | 1 << 1);
    }
    match eid {
        SBI_EID::SET_TIMER => {
            {
                debug!("VS set timer");
                write_csr!(CSR_HVIP, 0); //VSTIP
                write_csr!(CSR_SIE, 1 << 9 | 1 << 5 | 1 << 1);
            }
        }
        //_ => sbi_ret = sbi_dummy_handler(),
        _ => warn!("Pass through SBI call id {:#x}", eid),
    }
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
