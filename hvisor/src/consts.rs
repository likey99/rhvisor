use crate::memory::addr::{align_up, PhysAddr, VirtAddr};
pub use crate::memory::PAGE_SIZE;

/// Size of the hypervisor heap.
pub const HV_HEAP_SIZE: usize = 1024 * 1024; // 1 MB

pub const PER_CPU_ARRAY_PTR: *mut VirtAddr = __core_end as _;
/// Size of the per-CPU data (stack and other CPU-local data).
pub const PER_CPU_SIZE: usize = 64 * 1024; // 64KB  //may get bigger when dev
pub const MAX_CPU_NUM: usize = 8;

/// Size of the per cpu boot stack
pub const PER_CPU_BOOT_SIZE: usize = 1024; // 1KB
/// Start virtual address of the hypervisor memory.
pub const HV_BASE: VirtAddr = 0x80200000;
pub const HV_PHY_BASE: PhysAddr = 0x80200000;

extern "C" {
    fn __core_end();
}
pub fn core_end() -> VirtAddr {
    __core_end as _
}

pub fn mem_pool_start() -> VirtAddr {
    core_end() + MAX_CPU_NUM * PER_CPU_SIZE
}
pub fn hv_end() -> VirtAddr {
    mem_pool_start() + HV_HEAP_SIZE
}
pub const INVALID_ADDRESS: usize = usize::MAX;
