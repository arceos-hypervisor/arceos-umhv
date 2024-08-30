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
.endif

.endm

/* U mode and S mode use, guests use run_guest to resume guests. */
.macro RESTORE_STATE, from_user
.if \from_user == 1
    ld    gp, ({guest_gp})(sp)          
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

.section .text
.balign 4
.global trap_base
trap_base:
    // sscratch == 0: trap from S mode
    // sscratch != 0: trap from U mode or guest
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
    call    trap_handler
    RESTORE_STATE 1
    sret


