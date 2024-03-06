#![allow(dead_code)]

use crate::arch::riscv::csr::*;
use crate::consts::{INVALID_ADDRESS, PAGE_SIZE, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::memory::addr::VirtAddr;
use riscv::register::sie;
#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub x: [usize; 32], //x0~x31
    pub hstatus: usize,
    pub sstatus: usize,
    pub sepc: usize,
    pub stack_top: usize,
    pub hartid: usize,
}
impl ArchCpu {
    pub fn new(hartid: usize) -> Self {
        ArchCpu {
            x: [0; 32],
            hstatus: 0,
            sstatus: 0,
            sepc: 0,
            stack_top: 0,
            hartid,
        }
    }
    pub fn get_hartid(&self) -> usize {
        self.hartid
    }
    pub fn stack_top(&self) -> VirtAddr {
        PER_CPU_ARRAY_PTR as VirtAddr + (self.get_hartid() + 1) as usize * PER_CPU_SIZE - 8
    }
    pub fn init(&mut self, entry: usize, cpu_id: usize, dtb: usize) -> usize {
        //self.sepc = guest_test as usize as u64;
        write_csr!(CSR_SSCRARCH, self as *const _ as usize); //arch cpu pointer
        self.sepc = entry;
        self.hstatus = 1 << 7 | 2 << 32; //HSTATUS_SPV | HSTATUS_VSXL_64
        self.sstatus = 1 << 8 | 1 << 63 | 3 << 13 | 3 << 15; //SPP
        self.stack_top = self.stack_top() as usize;
        self.x[10] = cpu_id; //cpu id
        self.x[11] = dtb; //dtb addr
        trace!("stack_top: {:#x}", self.stack_top);

        // write_csr!(CSR_SSTATUS, self.sstatus);
        // write_csr!(CSR_HSTATUS, self.hstatus);
        // write_csr!(CSR_SEPC, self.sepc);
        write_csr!(CSR_HIDELEG, 1 << 2 | 1 << 6 | 1 << 10); //HIDELEG_VSSI | HIDELEG_VSTI | HIDELEG_VSEI
        write_csr!(CSR_HEDELEG, 1 << 8 | 1 << 12 | 1 << 13 | 1 << 15); //HEDELEG_ECU | HEDELEG_IPF | HEDELEG_LPF | HEDELEG_SPF
        write_csr!(CSR_HCOUNTEREN, 1 << 1); //HCOUNTEREN_TM
                                            //In VU-mode, a counter is not readable unless the applicable bits are set in both hcounteren and scounteren.
        write_csr!(CSR_SCOUNTEREN, 1 << 1);
        write_csr!(CSR_HTIMEDELTA, 0);
        write_csr!(CSR_HENVCFG, 1 << 63);
        //write_csr!(CSR_VSSTATUS, 1 << 63 | 3 << 13 | 3 << 15); //SSTATUS_SD | SSTATUS_FS_DIRTY | SSTATUS_XS_DIRTY
        //write_csr!(CSR_SIE, 1 << 9 | 1 << 5 | 1 << 1); //SEIE STIE SSIE
        // enable all interupts
        unsafe {
            sie::set_sext();
            sie::set_ssoft();
            sie::set_stimer();
        }
        // write_csr!(CSR_HIE, 1 << 12 | 1 << 10 | 1 << 6 | 1 << 2); //SGEIE VSEIE VSTIE VSSIE
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
        //unreachable!();
        0
    }
    pub fn run(&mut self) {
        extern "C" {
            fn vcpu_arch_entry();
        }
        unsafe {
            vcpu_arch_entry();
        }
    }
    pub fn idle(&self) {
        unsafe {
            core::arch::asm!("wfi");
        }
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
