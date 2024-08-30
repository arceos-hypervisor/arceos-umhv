use axerrno::AxResult;
use core::marker::PhantomData;
use riscv::register::{hstatus, htinst, htval, hvip, scause, sie, sstatus, stval};

use super::irq;
use super::sbi::{BaseFunction, PmuFunction, RemoteFenceFunction, SbiMessage};
use crate::AxVMHal;
use axaddrspace::HostPhysAddr;
use axvcpu::AxArchVCpuExitReason;

use super::regs::*;
use super::timers;
use super::timers::{register_timer, TimerEventFn};
use sbi_rt::{pmu_counter_get_info, pmu_counter_stop};
use core::mem::size_of;
use riscv::addr::BitField;
use memoffset::offset_of;

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
    include_str!("guest.S"),
    // trapframe_size = const core::mem::size_of::<VmCpuRegisters>(),
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
    hyp_stvec = const hyp_csr_offset!(stvec),
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
);

extern "C" {
    // fn _test_guest(state: *mut VmCpuRegisters);
    fn _run_guest(state: *mut VmCpuRegisters);
}

#[derive(Default)]
/// A virtual CPU within a guest
pub struct VCpu<H: AxVMHal> {
    regs: VmCpuRegisters,
    marker: PhantomData<H>,
}

impl<H: AxVMHal> axvcpu::AxArchVCpu for VCpu<H> {
    type CreateConfig = ();

    type SetupConfig = ();

    fn new(_config: Self::CreateConfig) -> AxResult<Self> {
        let mut regs = VmCpuRegisters::default();
        // Set hstatus
        let mut hstatus = hstatus::read();
        hstatus.set_spv(true);
        // Set SPVP bit in order to accessing VS-mode memory from HS-mode.
        hstatus.set_spvp(true);
        unsafe {
            hstatus.write();
        }
        regs.guest_regs.hstatus = hstatus.bits();

        // Set sstatus
        let mut sstatus = sstatus::read();
        sstatus.set_spp(sstatus::SPP::Supervisor);
        regs.guest_regs.sstatus = sstatus.bits();

        regs.guest_regs.gprs.set_reg(GprIndex::A0, 0);
        // TODO:from _config
        regs.guest_regs.gprs.set_reg(GprIndex::A1, 0x9000_0000);

        Ok(Self {
            regs: regs,
            marker: core::marker::PhantomData,
        })
    }

    fn setup(&mut self, _config: Self::SetupConfig) -> AxResult {
        Ok(())
    }

    fn set_entry(&mut self, entry: usize) -> AxResult {
        let regs = &mut self.regs;
        regs.guest_regs.sepc = entry;
        Ok(())
    }

    fn set_ept_root(&mut self, ept_root: HostPhysAddr) -> AxResult {
        self.regs.virtual_hs_csrs.hgatp = 8usize << 60 | usize::from(ept_root) >> 12;
        unsafe {
            core::arch::asm!(
                "csrw hgatp, {hgatp}",
                hgatp = in(reg) self.regs.virtual_hs_csrs.hgatp,
            );
            core::arch::riscv64::hfence_gvma_all();
        }
        Ok(())
    }

    fn run(&mut self) -> AxResult<AxArchVCpuExitReason> {
        let regs = &mut self.regs;

        unsafe {
            // Safe to run the guest as it only touches memory assigned to it by being owned
            // by its page table
            // _test_guest(regs);
            // info!("before running guest!");
            // info!("{:p} {:#x?}",regs, regs);
            _run_guest(regs);
        }
        // info!("vmexit_handler!!!");
        // info!("{:p} {:#x?}",regs, regs);
        self.vmexit_handler()
    }

    fn bind(&mut self) -> AxResult {
        // unimplemented!()
        Ok(())
    }

    fn unbind(&mut self) -> AxResult {
        // unimplemented!()
        Ok(())
    }
}

impl<H: AxVMHal> VCpu<H> {
    /// Gets one of the vCPU's general purpose registers.
    pub fn get_gpr(&self, index: GprIndex) -> usize {
        self.regs.guest_regs.gprs.reg(index)
    }

    /// Set one of the vCPU's general purpose register.
    pub fn set_gpr(&mut self, index: GprIndex, val: usize) {
        self.regs.guest_regs.gprs.set_reg(index, val);
    }

    /// Advance guest pc by `instr_len` bytes
    pub fn advance_pc(&mut self, instr_len: usize) {
        self.regs.guest_regs.sepc += instr_len
    }

    /// Gets the vCPU's registers.
    pub fn regs(&mut self) -> &mut VmCpuRegisters {
        &mut self.regs
    }
}

impl<H: AxVMHal> VCpu<H> {
    fn vmexit_handler(&mut self) -> AxResult<AxArchVCpuExitReason> {

        // info!("trap from guest!");
        self.regs.trap_csrs.scause = scause::read().bits();
        // info!("scause:{:x}", scause::read().bits());
        self.regs.trap_csrs.stval = stval::read();
        self.regs.trap_csrs.htval = htval::read();
        self.regs.trap_csrs.htinst = htinst::read();
        let scause = self.regs.trap_csrs.scause;
        // info!("scause:{:x}", scause);

        use scause::{Exception, Interrupt, Trap};
        if scause.get_bit(size_of::<usize>() * 8 - 1) {
            match Trap::Interrupt(Interrupt::from(scause & !(1 << (size_of::<usize>() * 8 - 1)))) {
                Trap::Interrupt(Interrupt::SupervisorTimer) => {
                    irq::handler_irq(irq::TIMER_IRQ_NUM);
                    Ok(AxArchVCpuExitReason::Nothing)
                }
                Trap::Interrupt(Interrupt::SupervisorExternal) => {
                    Ok(AxArchVCpuExitReason::ExternalInterrupt { vector: 0 })
                }
                _ => {
                    panic!(
                        "Unhandled trap: {:?}, sepc: {:#x}, stval: {:#x}",
                        Trap::Interrupt(Interrupt::from(scause)),
                        self.regs.guest_regs.sepc,
                        self.regs.trap_csrs.stval
                    );
                }
            }
        } else {
            match Trap::Exception(Exception::from(scause & !(1 << (size_of::<usize>() * 8 - 1)))) {
                Trap::Exception(Exception::VirtualSupervisorEnvCall) => self.handle_sbi_msg(),
                Trap::Exception(Exception::LoadGuestPageFault)
                | Trap::Exception(Exception::StoreGuestPageFault) => {
                    let fault_addr = self.regs.trap_csrs.htval << 2 | self.regs.trap_csrs.stval & 0x3;
                    Ok(AxArchVCpuExitReason::NestedPageFault { addr: fault_addr })
                }
                _ => {
                    panic!(
                        "Unhandled trap: {:?}, sepc: {:#x}, stval: {:#x}",
                        Trap::Exception(Exception::from(scause)),
                        self.regs.guest_regs.sepc,
                        self.regs.trap_csrs.stval
                    );
                }
            }
        }
    }
}

impl<H: AxVMHal> VCpu<H> {
    fn handle_sbi_msg(&mut self) -> AxResult<AxArchVCpuExitReason> {
        let sbi_msg = SbiMessage::from_regs(self.regs.guest_regs.gprs.a_regs()).ok();
        if let Some(sbi_msg) = sbi_msg {
            match sbi_msg {
                SbiMessage::Base(base) => {
                    self.handle_base_function(base)?;
                }
                SbiMessage::GetChar => {
                    let c = sbi_rt::legacy::console_getchar();
                    self.set_gpr(GprIndex::A0, c);
                }
                SbiMessage::PutChar(c) => {
                    sbi_rt::legacy::console_putchar(c);
                }
                SbiMessage::SetTimer(timer) => {
                    // Clear guest timer interrupt
                    unsafe {
                        hvip::clear_vstip();
                    }

                    register_timer(
                        timer * 100,
                        TimerEventFn::new(|_now| unsafe {
                            hvip::set_vstip();
                        }),
                    );
                }
                SbiMessage::Reset(_) => {
                    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::SystemFailure);
                }
                SbiMessage::RemoteFence(rfnc) => {
                    self.handle_rfnc_function(rfnc)?;
                }
                SbiMessage::PMU(pmu) => {
                    self.handle_pmu_function(pmu)?;
                }
                _ => todo!(),
            }
            self.advance_pc(4);
            Ok(AxArchVCpuExitReason::Nothing)
        } else {
            panic!(
                "Unhandled Trap: {:?}, sepc: {:#x}, stval: {:#x}",
                scause::read().cause(),
                self.regs.guest_regs.sepc,
                self.regs.trap_csrs.stval
            );
        }
    }

    fn handle_base_function(&mut self, base: BaseFunction) -> AxResult<()> {
        match base {
            BaseFunction::GetSepcificationVersion => {
                let version = sbi_rt::get_spec_version();
                self.set_gpr(GprIndex::A1, version.major() << 24 | version.minor());
                debug!(
                    "GetSepcificationVersion: {}",
                    version.major() << 24 | version.minor()
                );
            }
            BaseFunction::GetImplementationID => {
                let id = sbi_rt::get_sbi_impl_id();
                self.set_gpr(GprIndex::A1, id);
            }
            BaseFunction::GetImplementationVersion => {
                let impl_version = sbi_rt::get_sbi_impl_version();
                self.set_gpr(GprIndex::A1, impl_version);
            }
            BaseFunction::ProbeSbiExtension(extension) => {
                let extension = sbi_rt::probe_extension(extension as usize).raw;
                self.set_gpr(GprIndex::A1, extension);
            }
            BaseFunction::GetMachineVendorID => {
                let mvendorid = sbi_rt::get_mvendorid();
                self.set_gpr(GprIndex::A1, mvendorid);
            }
            BaseFunction::GetMachineArchitectureID => {
                let marchid = sbi_rt::get_marchid();
                self.set_gpr(GprIndex::A1, marchid);
            }
            BaseFunction::GetMachineImplementationID => {
                let mimpid = sbi_rt::get_mimpid();
                self.set_gpr(GprIndex::A1, mimpid);
            }
        }
        self.set_gpr(GprIndex::A0, 0);
        Ok(())
    }

    fn handle_rfnc_function(&mut self, rfnc: RemoteFenceFunction) -> AxResult<()> {
        self.set_gpr(GprIndex::A0, 0);
        match rfnc {
            RemoteFenceFunction::FenceI {
                hart_mask,
                hart_mask_base,
            } => {
                let sbi_ret = sbi_rt::remote_fence_i(hart_mask as usize, hart_mask_base as usize);
                self.set_gpr(GprIndex::A0, sbi_ret.error);
                self.set_gpr(GprIndex::A1, sbi_ret.value);
            }
            RemoteFenceFunction::RemoteSFenceVMA {
                hart_mask,
                hart_mask_base,
                start_addr,
                size,
            } => {
                let sbi_ret = sbi_rt::remote_sfence_vma(
                    hart_mask as usize,
                    hart_mask_base as usize,
                    start_addr as usize,
                    size as usize,
                );
                self.set_gpr(GprIndex::A0, sbi_ret.error);
                self.set_gpr(GprIndex::A1, sbi_ret.value);
            }
        }
        Ok(())
    }

    fn handle_pmu_function(&mut self, pmu: PmuFunction) -> AxResult<()> {
        self.set_gpr(GprIndex::A0, 0);
        match pmu {
            PmuFunction::GetNumCounters => self.set_gpr(GprIndex::A1, sbi_rt::pmu_num_counters()),
            PmuFunction::GetCounterInfo(counter_index) => {
                let sbi_ret = pmu_counter_get_info(counter_index as usize);
                self.set_gpr(GprIndex::A0, sbi_ret.error);
                self.set_gpr(GprIndex::A1, sbi_ret.value);
            }
            PmuFunction::StopCounter {
                counter_index,
                counter_mask,
                stop_flags,
            } => {
                let sbi_ret = pmu_counter_stop(
                    counter_index as usize,
                    counter_mask as usize,
                    stop_flags as usize,
                );
                self.set_gpr(GprIndex::A0, sbi_ret.error);
                self.set_gpr(GprIndex::A1, sbi_ret.value);
            }
        }
        Ok(())
    }
}
