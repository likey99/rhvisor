use crate::arch::riscv::csr::*;
use aarch64_cpu::registers::CCSIDR_EL1::AssociativityWithCCIDX::Value;
use core::arch::{asm, global_asm};
use log::info;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};
pub fn init() {
    unsafe {
        // Set the trap vector.
        stvec::write(trap_handler as usize, TrapMode::Direct);
    }
}
pub fn trap_handler() {
    let mut value: u64;
    unsafe {
        ::core::arch::asm!("csrr {value}, {csr}",
    value = out(reg) value,
    csr = const CSR_SCAUSE,);
    }
    log::info!("CSR_SCAUSE: {:#x}", value);
    if value == 0xa {
        info!("ecall from VS mode");
    } else {
        log::error!("trap unimplemented!");
    }

    unreachable!();
}
