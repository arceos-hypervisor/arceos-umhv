use core::mem::size_of;
use memoffset::offset_of;
use memory_addr::VirtAddr;
use page_table_entry::MappingFlags;
use riscv::register::scause::{self, Exception as E, Trap};
use riscv::register::{hstatus, htinst, htval, stval};

use super::irq::handler_irq;
use super::regs::*;

extern "C" {
    fn vmexit_riscv_handler(state: *mut VmCpuRegisters);
}

static mut EXCEPTION_STACK: [u8; 8192] = [0; 8192];

#[allow(dead_code)]
const fn hyp_gpr_offset(index: GprIndex) -> usize {
    offset_of!(VmCpuRegisters, hyp_regs)
        + offset_of!(HypervisorCpuState, gprs)
        + (index as usize) * size_of::<u64>()
}

#[allow(dead_code)]
const fn guest_gpr_offset(index: GprIndex) -> usize {
    offset_of!(VmCpuRegisters, guest_regs)
        + offset_of!(GuestCpuState, gprs)
        + (index as usize) * size_of::<u64>()
}

#[allow(unused_macros)]
macro_rules! hyp_csr_offset {
    ($reg:tt) => {
        offset_of!(VmCpuRegisters, hyp_regs) + offset_of!(HypervisorCpuState, $reg)
    };
}

#[allow(unused_macros)]
macro_rules! guest_csr_offset {
    ($reg:tt) => {
        offset_of!(VmCpuRegisters, guest_regs) + offset_of!(GuestCpuState, $reg)
    };
}

core::arch::global_asm!(
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<VmCpuRegisters>(),
    hyp_ra = const hyp_gpr_offset(GprIndex::RA),
    hyp_gp = const hyp_gpr_offset(GprIndex::GP),
    hyp_tp = const hyp_gpr_offset(GprIndex::TP),
    hyp_s0 = const hyp_gpr_offset(GprIndex::S0),
    hyp_s1 = const hyp_gpr_offset(GprIndex::S1),
    // hyp_a0 = const hyp_gpr_offset(GprIndex::A0),
    hyp_a1 = const hyp_gpr_offset(GprIndex::A1),
    hyp_a2 = const hyp_gpr_offset(GprIndex::A2),
    hyp_a3 = const hyp_gpr_offset(GprIndex::A3),
    hyp_a4 = const hyp_gpr_offset(GprIndex::A4),
    hyp_a5 = const hyp_gpr_offset(GprIndex::A5),
    hyp_a6 = const hyp_gpr_offset(GprIndex::A6),
    hyp_a7 = const hyp_gpr_offset(GprIndex::A7),
    hyp_s2 = const hyp_gpr_offset(GprIndex::S2),
    hyp_s3 = const hyp_gpr_offset(GprIndex::S3),
    hyp_s4 = const hyp_gpr_offset(GprIndex::S4),
    hyp_s5 = const hyp_gpr_offset(GprIndex::S5),
    hyp_s6 = const hyp_gpr_offset(GprIndex::S6),
    hyp_s7 = const hyp_gpr_offset(GprIndex::S7),
    hyp_s8 = const hyp_gpr_offset(GprIndex::S8),
    hyp_s9 = const hyp_gpr_offset(GprIndex::S9),
    hyp_s10 = const hyp_gpr_offset(GprIndex::S10),
    hyp_s11 = const hyp_gpr_offset(GprIndex::S11),
    hyp_sp = const hyp_gpr_offset(GprIndex::SP),
    hyp_sstatus = const hyp_csr_offset!(sstatus),
    hyp_hstatus = const hyp_csr_offset!(hstatus),
    hyp_scounteren = const hyp_csr_offset!(scounteren),
    // hyp_stvec = const hyp_csr_offset!(stvec),
    hyp_sscratch = const hyp_csr_offset!(sscratch),
    guest_ra = const guest_gpr_offset(GprIndex::RA),
    guest_gp = const guest_gpr_offset(GprIndex::GP),
    guest_tp = const guest_gpr_offset(GprIndex::TP),
    guest_s0 = const guest_gpr_offset(GprIndex::S0),
    guest_s1 = const guest_gpr_offset(GprIndex::S1),
    guest_a0 = const guest_gpr_offset(GprIndex::A0),
    guest_a1 = const guest_gpr_offset(GprIndex::A1),
    guest_a2 = const guest_gpr_offset(GprIndex::A2),
    guest_a3 = const guest_gpr_offset(GprIndex::A3),
    guest_a4 = const guest_gpr_offset(GprIndex::A4),
    guest_a5 = const guest_gpr_offset(GprIndex::A5),
    guest_a6 = const guest_gpr_offset(GprIndex::A6),
    guest_a7 = const guest_gpr_offset(GprIndex::A7),
    guest_s2 = const guest_gpr_offset(GprIndex::S2),
    guest_s3 = const guest_gpr_offset(GprIndex::S3),
    guest_s4 = const guest_gpr_offset(GprIndex::S4),
    guest_s5 = const guest_gpr_offset(GprIndex::S5),
    guest_s6 = const guest_gpr_offset(GprIndex::S6),
    guest_s7 = const guest_gpr_offset(GprIndex::S7),
    guest_s8 = const guest_gpr_offset(GprIndex::S8),
    guest_s9 = const guest_gpr_offset(GprIndex::S9),
    guest_s10 = const guest_gpr_offset(GprIndex::S10),
    guest_s11 = const guest_gpr_offset(GprIndex::S11),
    guest_t0 = const guest_gpr_offset(GprIndex::T0),
    guest_t1 = const guest_gpr_offset(GprIndex::T1),
    guest_t2 = const guest_gpr_offset(GprIndex::T2),
    guest_t3 = const guest_gpr_offset(GprIndex::T3),
    guest_t4 = const guest_gpr_offset(GprIndex::T4),
    guest_t5 = const guest_gpr_offset(GprIndex::T5),
    guest_t6 = const guest_gpr_offset(GprIndex::T6),
    guest_sp = const guest_gpr_offset(GprIndex::SP),

    guest_sstatus = const guest_csr_offset!(sstatus),
    guest_hstatus = const guest_csr_offset!(hstatus),
    guest_scounteren = const guest_csr_offset!(scounteren),
    guest_sepc = const guest_csr_offset!(sepc),
    exception_stack = sym EXCEPTION_STACK,
    exception_stack_size = const 4096,
);

fn handle_breakpoint(sepc: &mut usize) {
    debug!("Exception(Breakpoint) @ {:#x} ", sepc);
    *sepc += 2
}

fn handle_page_fault(tf: &VmCpuRegisters, access_flags: MappingFlags, is_user: bool) {
    let vaddr = VirtAddr::from(stval::read());

    panic!(
        "Unhandled {} Page Fault @ {:#x}, fault_vaddr={:#x} ({:?}):\n{:#x?}",
        if is_user { "User" } else { "Supervisor" },
        tf.guest_regs.sepc,
        vaddr,
        access_flags,
        tf.guest_regs,
    );
}

#[no_mangle]
fn trap_handler(tf: &mut VmCpuRegisters, from_user: bool) {
    let hstatus = hstatus::read();

    match hstatus.spv() {
        true => {
            // from V = 1
            // info!("trap from guest!");
            tf.trap_csrs.scause = scause::read().bits();
            // info!("scause:{:x}", scause::read().bits());
            tf.trap_csrs.stval = stval::read();
            tf.trap_csrs.htval = htval::read();
            tf.trap_csrs.htinst = htinst::read();

            unsafe {
                vmexit_riscv_handler(tf);
            }
        }
        _ => {
            // from V = 0

            let scause = scause::read();
            // info!("trap not from guest! scause: {:?}", scause.cause());
            match scause.cause() {
                Trap::Exception(E::LoadPageFault) => {
                    handle_page_fault(tf, MappingFlags::READ, from_user)
                }
                Trap::Exception(E::StorePageFault) => {
                    handle_page_fault(tf, MappingFlags::WRITE, from_user)
                }
                Trap::Exception(E::InstructionPageFault) => {
                    handle_page_fault(tf, MappingFlags::EXECUTE, from_user)
                }
                Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.guest_regs.sepc),
                Trap::Interrupt(_) => {
                    // handle_trap!(IRQ, scause.bits());
                    handler_irq(scause.bits());
                }
                _ => {
                    panic!(
                        "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
                        scause.cause(),
                        tf.guest_regs.sepc,
                        tf.guest_regs
                    );
                }
            }
        }
    }
}
