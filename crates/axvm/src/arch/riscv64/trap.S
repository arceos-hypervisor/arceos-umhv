.macro SAVE_STATE, from_user
    addi    sp, sp, -{trapframe_size}
    sd   ra, ({guest_ra})(sp)
    sd   s0, ({guest_s0})(sp)
    sd   s1, ({guest_s1})(sp)
	sd   a0, ({guest_a0})(sp)
    sd   a1, ({guest_a1})(sp)
    sd   a2, ({guest_a2})(sp)
    sd   a3, ({guest_a3})(sp)
    sd   a4, ({guest_a4})(sp)
    sd   a5, ({guest_a5})(sp)
    sd   a6, ({guest_a6})(sp)
    sd   a7, ({guest_a7})(sp)
    sd   s2, ({guest_s2})(sp)
    sd   s3, ({guest_s3})(sp)
    sd   s4, ({guest_s4})(sp)
    sd   s5, ({guest_s5})(sp)
    sd   s6, ({guest_s6})(sp)
    sd   s7, ({guest_s7})(sp)
    sd   s8, ({guest_s8})(sp)
    sd   s9, ({guest_s9})(sp)
    sd   s10, ({guest_s10})(sp)
    sd   s11, ({guest_s11})(sp)
    sd   t0, ({guest_t0})(sp)
    sd   t1, ({guest_t1})(sp)
    sd   t2, ({guest_t2})(sp)
    sd   t3, ({guest_t3})(sp)
    sd   t4, ({guest_t4})(sp)
    sd   t5, ({guest_t5})(sp)
    sd   t6, ({guest_t6})(sp)

    csrr  t0, sepc
	sd    t0, ({guest_sepc})(sp)
    csrr  t0, sstatus
	sd    t0, ({guest_sstatus})(sp)
	csrr  t0, hstatus
    sd    t0, ({guest_hstatus})(sp)
    csrr  t0, scounteren
    sd    t0, ({guest_scounteren})(sp)

	csrrw t0, sscratch, zero          
	sd    t0, ({guest_sp})(sp)

.if \from_user == 1
    sd    gp, ({guest_gp})(sp)        
    sd    tp, ({guest_tp})(sp)  
    ; ld    tp, ({hyp_tp})(sp)
.endif

.endm

/* U mode and S mode use, guests use run_guest to resume guests. */
.macro RESTORE_STATE, from_user
.if \from_user == 1
    ld    gp, ({guest_gp})(sp) 
    ; sd    tp, ({hyp_tp})(sp)          
    ld    tp, ({guest_tp})(sp)
    addi  t0, sp, {trapframe_size}     
    csrw  sscratch, t0
.endif

    ld    t1, ({guest_sstatus})(sp)
    csrw  sstatus, t1
    ld    t1, ({guest_hstatus})(sp)
    csrw  hstatus, t1
    ld    t1, ({guest_scounteren})(sp)
    csrw  scounteren, t1
    ld    t1, ({guest_sepc})(sp)
    csrw  sepc, t1

    ld    ra, ({guest_ra})(sp)
    ld    s0, ({guest_s0})(sp)
    ld    s1, ({guest_s1})(sp)
	ld    a0, ({guest_a0})(sp)
    ld    a1, ({guest_a1})(sp)
    ld    a2, ({guest_a2})(sp)
    ld    a3, ({guest_a3})(sp)
    ld    a4, ({guest_a4})(sp)
    ld    a5, ({guest_a5})(sp)
    ld    a6, ({guest_a6})(sp)
    ld    a7, ({guest_a7})(sp)
    ld    s2, ({guest_s2})(sp)
    ld    s3, ({guest_s3})(sp)
    ld    s4, ({guest_s4})(sp)
    ld    s5, ({guest_s5})(sp)
    ld    s6, ({guest_s6})(sp)
    ld    s7, ({guest_s7})(sp)
    ld    s8, ({guest_s8})(sp)
    ld    s9, ({guest_s9})(sp)
    ld    s10, ({guest_s10})(sp)
    ld    s11, ({guest_s11})(sp)
    ld    t0, ({guest_t0})(sp)
    ld    t1, ({guest_t1})(sp)
    ld    t2, ({guest_t2})(sp)
    ld    t3, ({guest_t3})(sp)
    ld    t4, ({guest_t4})(sp)
    ld    t5, ({guest_t5})(sp)
    ld    t6, ({guest_t6})(sp)
    ld    sp, ({guest_sp})(sp)
.endm

/* this macro is used for hypervisor to restore host state */
.macro RESTORE_HOST_STATE
	ld   ra, ({hyp_ra})(a0)
    ld   gp, ({hyp_gp})(a0)
    ld   tp, ({hyp_tp})(a0)
    ld   s0, ({hyp_s0})(a0)
    ld   s1, ({hyp_s1})(a0)
    ld   a1, ({hyp_a1})(a0)
    ld   a2, ({hyp_a2})(a0)
    ld   a3, ({hyp_a3})(a0)
    ld   a4, ({hyp_a4})(a0)
    ld   a5, ({hyp_a5})(a0)
    ld   a6, ({hyp_a6})(a0)
    ld   a7, ({hyp_a7})(a0)
    ld   s2, ({hyp_s2})(a0)
    ld   s3, ({hyp_s3})(a0)
    ld   s4, ({hyp_s4})(a0)
    ld   s5, ({hyp_s5})(a0)
    ld   s6, ({hyp_s6})(a0)
    ld   s7, ({hyp_s7})(a0)
    ld   s8, ({hyp_s8})(a0)
    ld   s9, ({hyp_s9})(a0)
    ld   s10, ({hyp_s10})(a0)
    ld   s11, ({hyp_s11})(a0)
    ld   sp, ({hyp_sp})(a0)

    
    ld    t1, ({hyp_hstatus})(a0)
    csrw  hstatus, t1
    ld    t1, ({hyp_scounteren})(a0)
    csrw  scounteren, t1
    ld    t1, ({hyp_sscratch})(a0)
    csrw  sscratch, t1
    ld    t1, ({hyp_sstatus})(a0)
    csrw  sstatus, t1
.endm

.section .text
.balign 4
.global trap_base
trap_base:
    // sscratch == 0: trap from S mode
    // sscratch != 0: trap from guest
    csrrw   sp, sscratch, sp            
    bnez    sp, .entry_u

    csrr    sp, sscratch               
    j       .entry_s

.entry_s:
    SAVE_STATE 0
    mv      a0, sp
    li      a1, 0
    call    trap_handler
    RESTORE_STATE 0
    sret

.entry_u:
    SAVE_STATE 1
    mv      a0, sp
    li      a1, 1
    la      sp, {exception_stack}
    li      t0, {exception_stack_size}
    add     sp, sp, t0
    call    trap_handler
    RESTORE_STATE 1
    sret


/// Enter the guest given in `VmCpuRegisters` from `a0`
.global _run_guest
_run_guest:
    /* Save hypervisor state */

    /* Save hypervisor GPRs (except T0-T6 and a0, which is GuestInfo and stashed in sscratch) */
    sd   ra, ({hyp_ra})(a0)
    sd   gp, ({hyp_gp})(a0)
    sd   tp, ({hyp_tp})(a0)
    sd   s0, ({hyp_s0})(a0)
    sd   s1, ({hyp_s1})(a0)
    sd   a1, ({hyp_a1})(a0)
    sd   a2, ({hyp_a2})(a0)
    sd   a3, ({hyp_a3})(a0)
    sd   a4, ({hyp_a4})(a0)
    sd   a5, ({hyp_a5})(a0)
    sd   a6, ({hyp_a6})(a0)
    sd   a7, ({hyp_a7})(a0)
    sd   s2, ({hyp_s2})(a0)
    sd   s3, ({hyp_s3})(a0)
    sd   s4, ({hyp_s4})(a0)
    sd   s5, ({hyp_s5})(a0)
    sd   s6, ({hyp_s6})(a0)
    sd   s7, ({hyp_s7})(a0)
    sd   s8, ({hyp_s8})(a0)
    sd   s9, ({hyp_s9})(a0)
    sd   s10, ({hyp_s10})(a0)
    sd   s11, ({hyp_s11})(a0)
    sd   sp, ({hyp_sp})(a0)

    /* Swap in guest CSRs. */
    ld    t1, ({guest_sstatus})(a0)
    csrrw t1, sstatus, t1
    sd    t1, ({hyp_sstatus})(a0)

    ld    t1, ({guest_hstatus})(a0)
    csrrw t1, hstatus, t1
    sd    t1, ({hyp_hstatus})(a0)

    ld    t1, ({guest_scounteren})(a0)
    csrrw t1, scounteren, t1
    sd    t1, ({hyp_scounteren})(a0)

    ld    t1, ({guest_sepc})(a0)
    csrw  sepc, t1

    /* Save sscratch and replace with pointer to GuestInfo. */
    addi  t1, a0, {trapframe_size}
    csrrw t1, sscratch, t1
    sd    t1, ({hyp_sscratch})(a0)

    /* Restore the gprs from this GuestInfo */
    ld   ra, ({guest_ra})(a0)
    ld   gp, ({guest_gp})(a0)       
    ld   tp, ({guest_tp})(a0)
    ld   s0, ({guest_s0})(a0)
    ld   s1, ({guest_s1})(a0)
    ld   a1, ({guest_a1})(a0)
    ld   a2, ({guest_a2})(a0)
    ld   a3, ({guest_a3})(a0)
    ld   a4, ({guest_a4})(a0)
    ld   a5, ({guest_a5})(a0)
    ld   a6, ({guest_a6})(a0)
    ld   a7, ({guest_a7})(a0)
    ld   s2, ({guest_s2})(a0)
    ld   s3, ({guest_s3})(a0)
    ld   s4, ({guest_s4})(a0)
    ld   s5, ({guest_s5})(a0)
    ld   s6, ({guest_s6})(a0)
    ld   s7, ({guest_s7})(a0)
    ld   s8, ({guest_s8})(a0)
    ld   s9, ({guest_s9})(a0)
    ld   s10, ({guest_s10})(a0)
    ld   s11, ({guest_s11})(a0)
    ld   t0, ({guest_t0})(a0)
    ld   t1, ({guest_t1})(a0)
    ld   t2, ({guest_t2})(a0)
    ld   t3, ({guest_t3})(a0)
    ld   t4, ({guest_t4})(a0)
    ld   t5, ({guest_t5})(a0)
    ld   t6, ({guest_t6})(a0)
    ld   sp, ({guest_sp})(a0)
    ld   a0, ({guest_a0})(a0)

    sret

.global vmexit_riscv_handler
vmexit_riscv_handler:
    RESTORE_HOST_STATE
    ret