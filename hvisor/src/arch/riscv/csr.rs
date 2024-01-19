#![allow(dead_code)]
pub const CSR_SCAUSE: u64 = 0x142;
pub const CSR_STVEC: u64 = 0x105;
pub const CSR_SEPC: u64 = 0x141;
pub const CSR_SSTATUS: u64 = 0x100;
pub const CSR_SSCRARCH: u64 = 0x140;
pub const CSR_VSSTATUS: u64 = 0x200;
pub const CSR_VSIE: u64 = 0x204;
pub const CSR_VSTVEC: u64 = 0x205;
pub const CSR_VSSCRATCH: u64 = 0x240;
pub const CSR_VSEPC: u64 = 0x241;
pub const CSR_VSCAUSE: u64 = 0x242;
pub const CSR_VSTVAL: u64 = 0x243;
pub const CSR_VSIP: u64 = 0x244;
pub const CSR_VSATP: u64 = 0x280;
/* Sstc Extension */
pub const CSR_VSTIMECMP: u64 = 0x24D;
pub const CSR_VSTIMECMPH: u64 = 0x25D;

pub const CSR_HSTATUS: u64 = 0x600;
pub const CSR_HEDELEG: u64 = 0x602;
pub const CSR_HIDELEG: u64 = 0x603;
pub const CSR_HIE: u64 = 0x604;
pub const CSR_HTIMEDELTA: u64 = 0x605;
pub const CSR_HTIMEDELTAH: u64 = 0x615;
pub const CSR_HCOUNTEREN: u64 = 0x606;
pub const CSR_HGEIE: u64 = 0x607;
pub const CSR_HTVAL: u64 = 0x643;
pub const CSR_HIP: u64 = 0x644;
pub const CSR_HVIP: u64 = 0x645;
pub const CSR_HTINST: u64 = 0x64A;
pub const CSR_HGATP: u64 = 0x680;
pub const CSR_HGEIP: u64 = 0xE07;
/* Hypervisor Configuration */
pub const CSR_HENVCFG: u64 = 0x60A;
pub const CSR_HENVCFGH: u64 = 0x61A;

/* Sstc Extension */
pub const CSR_STIMECMP: u64 = 0x14D;
pub const CSR_STIMECMPH: u64 = 0x15D;

// macro_rules! read_csr {
//     ($csr_number:expr) => {
//         {
//             let mut value: u64;
//             unsafe{
//                 ::core::arch::asm!(
//                 "csrr {value},  $csr_number",
//                 value = out(reg) value,
//                 options(nomem, nostack),
//             );}
//             value
//         }
//     }
// }
// pub(crate) use read_csr;
// macro_rules! write_csr {
//     ($csr_number:expr, $asm_fn: ident) => {
//         /// Writes the CSR
//         #[inline]
//         #[allow(unused_variables)]
//         unsafe fn _write(bits: usize) {
//             match () {
//                 #[cfg(all(riscv, feature = "inline-asm"))]
//                 () => core::arch::asm!("csrrw x0, {1}, {0}", in(reg) bits, const $csr_number),

//                 #[cfg(all(riscv, not(feature = "inline-asm")))]
//                 () => {
//                     extern "C" {
//                         fn $asm_fn(bits: usize);
//                     }

//                     $asm_fn(bits);
//                 }

//                 #[cfg(not(riscv))]
//                 () => unimplemented!(),
//             }
//         }
//     };
// }
// pub(crate) use write_csr;
