.text 

.macro VM_EXIT

    csrrw   x31, sscratch, x31
    
    sd   x1, 0*8(x31)
    sd   x2, 1*8(x31)
    sd   x3, 2*8(x31)
    sd   x4, 3*8(x31)
    sd   x5, 4*8(x31)
    sd   x6, 5*8(x31)
    sd   x7, 6*8(x31)
    sd   x8, 7*8(x31)
    sd   x9, 8*8(x31)
    sd   x10, 9*8(x31)
    sd   x11, 10*8(x31)
    sd   x12, 11*8(x31)
    sd   x13, 12*8(x31)
    sd   x14, 13*8(x31)
    sd   x15, 14*8(x31)
    sd   x16, 15*8(x31)
    sd   x17, 16*8(x31)
    sd   x18, 17*8(x31)
    sd   x19, 18*8(x31)
    sd   x20, 19*8(x31)
    sd   x21, 20*8(x31)
    sd   x22, 21*8(x31)
    sd   x23, 22*8(x31)
    sd   x24, 23*8(x31)
    sd   x25, 24*8(x31)
    sd   x26, 25*8(x31)
    sd   x27, 26*8(x31)
    sd   x28, 27*8(x31)
    sd   x29, 28*8(x31)
    sd   x30, 29*8(x31)
    
    mv      sp, x31
    csrrw   x31, sscratch, x31
    sd   x31, 30*8(sp)

    csrr    t0, hstatus
    sd   t0, 31*8(sp)
    csrr    t0, sstatus
    sd   t0, 32*8(sp)
    csrr    t0, sepc
    sd   t0, 33*8(sp)

    ld   sp, 34*8(sp)
    #la    sp, boot_stack_top
    csrr      a0, sscratch
    # .option push
    # .option norelax
    # la  gp, __global_pointer$
    # .option pop   

.endm



.macro VM_ENTRY

    csrr   x31, sscratch

    ld   x1, 31*8(x31)
    csrw   hstatus, x1
    ld   x1, 32*8(x31)
    csrw   sstatus, x1
    ld   x1, 33*8(x31)
    csrw   sepc, x1

    ld   x1, 0*8(x31)
    ld   x2, 1*8(x31)
    ld   x3, 2*8(x31)
    ld   x4, 3*8(x31)
    ld   x5, 4*8(x31)
    ld   x6, 5*8(x31)
    ld   x7, 6*8(x31)
    ld   x8, 7*8(x31)
    ld   x9, 8*8(x31)
    ld   x10, 9*8(x31)
    ld   x11, 10*8(x31)
    ld   x12, 11*8(x31)
    ld   x13, 12*8(x31)
    ld   x14, 13*8(x31)
    ld   x15, 14*8(x31)
    ld   x16, 15*8(x31)
    ld   x17, 16*8(x31)
    ld   x18, 17*8(x31)
    ld   x19, 18*8(x31)
    ld   x20, 19*8(x31)
    ld   x21, 20*8(x31)
    ld   x22, 21*8(x31)
    ld   x23, 22*8(x31)
    ld   x24, 23*8(x31)
    ld   x25, 24*8(x31)
    ld   x26, 25*8(x31)
    ld   x27, 26*8(x31)
    ld   x28, 27*8(x31)
    ld   x29, 28*8(x31)
    ld   x30, 29*8(x31) 
    ld   x31, 30*8(x31)  

    sret
    j   .
.endm

.balign 0x4
.global _hyp_trap_vector	
_hyp_trap_vector:
    VM_EXIT
    csrr    t0, scause
    bltz    t0, 1f
    call    {sync_exception_handler}
    j       2f
1:
    call    {interrupts_arch_handle}
2:
.global vcpu_arch_entry
vcpu_arch_entry:
    VM_ENTRY
