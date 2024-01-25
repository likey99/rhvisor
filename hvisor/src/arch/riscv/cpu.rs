#![allow(dead_code)]

use crate::arch::riscv::csr::*;
#[repr(C)]
pub struct ArchCpu {
    pub regs: [usize; 31],
    pub hstatus: usize,
    pub sstatus: usize,
    pub sepc: usize,
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
    pub fn init(&mut self) -> usize {
        //self.sepc = guest_test as usize as u64;
        self.sepc = 0x80400000;
        self.hstatus = 1 << 7 | 2 << 32; //HSTATUS_SPV | HSTATUS_VSXL_64
        self.sstatus = 1 << 8; //SPP
        write_csr!(CSR_SSCRARCH, &self.regs as *const _ as usize);
        write_csr!(CSR_SSTATUS, self.sstatus);
        write_csr!(CSR_HSTATUS, self.hstatus);
        write_csr!(CSR_SEPC, self.sepc);
        write_csr!(CSR_HCOUNTEREN, 1 << 1); //HCOUNTEREN_TM
        write_csr!(CSR_HTIMEDELTA, 0);
        write_csr!(CSR_VSSTATUS, 1 << 63 | 3 << 13 | 3 << 15); //SSTATUS_SD | SSTATUS_FS_DIRTY | SSTATUS_XS_DIRTY
        write_csr!(CSR_HIE, 0);
        write_csr!(CSR_VSTVEC, 0);
        write_csr!(CSR_VSSCRATCH, 0);
        write_csr!(CSR_VSEPC, 0);
        write_csr!(CSR_VSCAUSE, 0);
        write_csr!(CSR_VSTVAL, 0);
        write_csr!(CSR_HVIP, 0);
        write_csr!(CSR_VSATP, 0);
        let mut value: usize;
        value = read_csr!(CSR_SEPC);
        info!("CSR_SEPC: {:#x}", value);
        value = read_csr!(CSR_STVEC);
        info!("CSR_STVEC: {:#x}", value);
        value = read_csr!(CSR_VSATP);
        info!("CSR_VSATP: {:#x}", value);
        value = read_csr!(CSR_HGATP);
        info!("CSR_HGATP: {:#x}", value);
        unsafe {
            ::core::arch::asm!("sret",);
        }
        0
    }
}
// fn guest_test(id: usize, args: [usize; 3]) -> isize {
//     let mut ret: isize;
//     unsafe {
//         core::arch::asm!(
//             "ecall",
//             inlateout("x10") args[0] => ret,
//             in("x11") args[1],
//             in("x12") args[2],
//             in("x17") id
//         );
//     }
//     ret
//     //unreachable!();
// }
