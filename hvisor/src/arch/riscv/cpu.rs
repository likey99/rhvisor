#![allow(dead_code)]
use crate::arch::riscv::csr::*;
#[repr(C)]
pub struct ArchCpu {
    pub regs: [u64; 31],
    pub hstatus: u64,
    pub sstatus: u64,
    pub sepc: u64,
}
impl ArchCpu {
    pub fn new() -> Self {
        ArchCpu {
            regs: [0; 31],
            hstatus: 0,
            sstatus: 0,
            sepc: 0,
        }
    }
    pub fn init(&mut self) -> u64 {
        //self.sepc = guest_test as usize as u64;
        self.sepc = 0x80400000;
        self.hstatus = 1 << 7 | 2 << 32; //HSTATUS_SPV | HSTATUS_VSXL_64
        self.sstatus = 1 << 8; //SPP
        unsafe {
            ::core::arch::asm!(
                "csrw {csr}, {value}",
                value = in(reg) &self.regs as *const _ as u64,
                csr = const CSR_SSCRARCH,
                options(nomem, nostack),
            );
        }
        unsafe {
            ::core::arch::asm!(
                "csrw {csr}, {value}",
                value = in(reg) self.sstatus,
                csr = const CSR_SSTATUS,
                options(nomem, nostack),
            );
        }
        unsafe {
            ::core::arch::asm!(
                "csrw {csr}, {value}",
                value = in(reg) self.hstatus,
                csr = const CSR_HSTATUS,
                options(nomem, nostack),
            );
        }
        unsafe {
            ::core::arch::asm!(
                "csrw {csr}, {value}",
                value = in(reg) self.sepc,
                csr = const CSR_SEPC,
                options(nomem, nostack),
            );
        }
        let mut value: u64;
        unsafe {
            ::core::arch::asm!("csrr {value}, {csr}",
        value = out(reg) value,
        csr = const CSR_SEPC,);
        }
        log::info!("CSR_SEPC: {:#x}", value);
        unsafe {
            ::core::arch::asm!("csrr {value}, {csr}",
        value = out(reg) value,
        csr = const CSR_STVEC,);
        }
        log::info!("CSR_STVEC: {:#x}", value);
        unsafe {
            ::core::arch::asm!("sret",);
        }
        0
    }
}
fn guest_test(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
    //unreachable!();
}
