pub use crate::memory::addr::PAGE_SIZE;
use crate::memory::addr::{align_up, VirtAddr};

/// Size of the hypervisor heap.
pub const HV_HEAP_SIZE: usize = 1024 * 1024; // 1 MB

/// Size of the per-CPU data (stack and other CPU-local data).
pub const PER_CPU_SIZE: usize = 64 * 1024; // 64KB  //may get bigger when dev

/// Size of the per cpu boot stack
pub const PER_CPU_BOOT_SIZE: usize = 1024; // 1KB
/// Start virtual address of the hypervisor memory.
pub const HV_BASE: usize = 0xffffc0200000;
pub fn hv_end() -> usize {
    extern "C" {
        fn __core_end();
    }
    unsafe { __core_end as usize }
}

pub fn mem_pool_start() -> usize {
    hv_end()
}

pub fn mem_pool_end() -> usize {
    hv_end() + HV_HEAP_SIZE
}

pub const INVALID_ADDRESS: usize = usize::MAX;
