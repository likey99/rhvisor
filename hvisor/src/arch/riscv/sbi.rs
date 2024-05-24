//! SBI call wrappers

#![allow(unused)]
use crate::percpu::get_cpu_data;

use super::cpu::ArchCpu;
use crate::arch::riscv::csr::*;
use riscv::register::{hvip, sie};
pub mod SBI_EID {
    pub const BASE_EXTID: usize = 0x10;
    pub const SET_TIMER: usize = 0x54494D45;
    pub const EXTID_HSM: usize = 0x48534D;
    pub const SEND_IPI: usize = 0x735049;
    pub const RFENCE: usize = 0x52464E43;
    pub const PMU: usize = 0x504D55;
}
pub const SBI_SUCCESS: i64 = 0;
pub const SBI_ERR_FAILURE: i64 = -1;
pub const SBI_ERR_NOT_SUPPORTED: i64 = -2;
pub const SBI_ERR_INVALID_PARAM: i64 = -3;
pub const SBI_ERR_DENIED: i64 = -4;
pub const SBI_ERR_INVALID_ADDRESS: i64 = -5;
pub const SBI_ERR_ALREADY_AVAILABLE: i64 = -6;
pub struct SbiRet {
    error: i64,
    value: i64,
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
    let eid: usize = current_cpu.x[17];
    let fid: usize = current_cpu.x[16];
    let sbi_ret;

    match eid {
        //SBI_EXTID_BASE => sbi_ret = sbi_base_handler(fid, current_cpu),
        SBI_EID::BASE_EXTID => {
            trace!("SBI_EID::BASE,fid:{:#x}", fid);
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
        SBI_EID::SET_TIMER => {
            //debug!("SBI_EID::SET_TIMER on CPU {}", current_cpu.hartid);
            sbi_ret = sbi_time_handler(fid, current_cpu);
        }
        SBI_EID::EXTID_HSM => {
            warn!("SBI_EID::EXTID_HSM on CPU {}", current_cpu.hartid);
            sbi_ret = sbi_hsm_handler(fid, current_cpu);
        }
        SBI_EID::SEND_IPI => {
            trace!("SBI_EID::SEND_IPI on CPU {}", current_cpu.hartid);
            trace!(
                "SBI_EID::SEND_IPI,hartid:{:#x},mask:{:#x}",
                current_cpu.x[10],
                current_cpu.x[11]
            );
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
        SBI_EID::RFENCE => {
            trace!("SBI_EID::RFENCE,mask:{:#x}", current_cpu.x[10]);
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
        SBI_EID::PMU => {
            trace!("SBI_EID::PMU,fid:{:#x}", fid);
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
        //_ => sbi_ret = sbi_dummy_handler(),
        _ => {
            warn!(
                "Pass through SBI call eid {:#x} fid:{:#x} on CPU {}",
                eid, fid, current_cpu.hartid
            );
            sbi_ret = sbi_call_5(
                eid,
                fid,
                current_cpu.x[10],
                current_cpu.x[11],
                current_cpu.x[12],
                current_cpu.x[13],
                current_cpu.x[14],
            );
        }
    }
    current_cpu.x[10] = sbi_ret.error as usize;
    current_cpu.x[11] = sbi_ret.value as usize;
}

pub fn sbi_call_5(
    eid: usize,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> SbiRet {
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
    SbiRet { error, value }
}

pub fn sbi_time_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    let stime = current_cpu.x[10];
    warn!("SBI_SET_TIMER stime: {:#x}", stime);
    if current_cpu.sstc {
        write_csr!(CSR_VSTIMECMP, stime);
    } else {
        set_timer(stime);
        unsafe {
            // clear guest timer interrupt pending
            hvip::clear_vstip();
            // enable timer interrupt
            sie::set_stimer();
        }
    }
    //debug!("SBI_SET_TIMER stime: {:#x}", stime);
    return sbi_ret;
}
pub fn sbi_hsm_handler(fid: usize, current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    match fid {
        0 => {
            // hsm start
            sbi_ret = sbi_hsm_start_handler(current_cpu);
        }
        _ => {
            error!("Unsupported HSM function {:#x}", fid);
        }
    }
    sbi_ret
}
pub fn sbi_hsm_start_handler(current_cpu: &mut ArchCpu) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    let hartid = current_cpu.x[10];

    if (hartid == current_cpu.hartid) {
        sbi_ret.error = SBI_ERR_ALREADY_AVAILABLE;
    } else {
        //TODO:add sbi conext in archcpu
        let hartid = current_cpu.x[10];
        let start_addr = current_cpu.x[11];
        let opaque = current_cpu.x[12];
        let target_cpu = get_cpu_data(hartid);
        target_cpu.cpu_on_entry = start_addr;
        target_cpu.arch_cpu.sepc = start_addr;
        target_cpu.arch_cpu.x[11] = opaque;
        warn!(
            "@CPU{} hartid: {:#x}, start_addr: {:#x}, opaque: {:#x}",
            current_cpu.hartid, hartid, start_addr, opaque
        );
        let _ret = sbi_rt::send_ipi(1 << hartid, 0);
        warn!(
            "send ipi to CPU{} ret: {} {}",
            hartid, _ret.error, _ret.value
        );
    }
    sbi_ret
}
