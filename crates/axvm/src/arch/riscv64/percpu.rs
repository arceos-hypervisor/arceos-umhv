use crate::percpu::AxVMArchPerCpu;
use axerrno::AxError;
use axerrno::AxResult;

use super::consts::traps;
use super::detect::detect_h_extension;
use super::timers;
use super::irq;
use crate::AxVMHal;
use riscv::register::{hedeleg, hideleg, hvip, sie, stvec};

extern "C" {
    fn trap_base();
}

pub struct PerCpu<H: AxVMHal> {
    _marker: core::marker::PhantomData<H>,
}

impl<H: AxVMHal> AxVMArchPerCpu for PerCpu<H> {
    fn new(cpu_id: usize) -> AxResult<Self> {
        unsafe {
            setup_csrs();
        }

        if cpu_id == 0 {
            info!("[TRANCE]register_handler");
            irq::register_handler(irq::TIMER_IRQ_NUM, || {
                // info!("TIMER_IRQ_NUM handler!!!");
                unsafe {
                    sie::clear_stimer();
                }

                timers::check_events();
                timers::scheduler_next_event();
                unsafe {
                    sie::set_stimer();
                }
            });
        }

        timers::init();

        Ok(Self {
            _marker: core::marker::PhantomData,
        })
    }

    fn is_enabled(&self) -> bool {
        unimplemented!()
    }

    fn hardware_enable(&mut self) -> AxResult<()> {
        if detect_h_extension() {
            Ok(())
        } else {
            Err(AxError::Unsupported)
        }
    }

    fn hardware_disable(&mut self) -> AxResult<()> {
        unimplemented!()
    }
}

/// Initialize (H)S-level CSRs to a reasonable state.
unsafe fn setup_csrs() {
    // Delegate some synchronous exceptions.
    hedeleg::Hedeleg::from_bits(
        traps::exception::INST_ADDR_MISALIGN
            | traps::exception::BREAKPOINT
            | traps::exception::ENV_CALL_FROM_U_OR_VU
            | traps::exception::INST_PAGE_FAULT
            | traps::exception::LOAD_PAGE_FAULT
            | traps::exception::STORE_PAGE_FAULT
            | traps::exception::ILLEGAL_INST,
    )
    .write();

    // Delegate all interupts.
    hideleg::Hideleg::from_bits(
        traps::interrupt::VIRTUAL_SUPERVISOR_TIMER
            | traps::interrupt::VIRTUAL_SUPERVISOR_EXTERNAL
            | traps::interrupt::VIRTUAL_SUPERVISOR_SOFT,
    )
    .write();

    // Clear all interrupts.
    hvip::clear_vssip();
    hvip::clear_vstip();
    hvip::clear_vseip();

    // clear all interrupts.
    // the csr num of hcounteren is 0x606, the riscv repo is error!!!
    // hcounteren::Hcounteren::from_bits(0xffff_ffff).write();
    core::arch::asm!("csrw {csr}, {rs}", csr = const 0x606, rs = in(reg) -1);

    // enable interrupt
    sie::set_sext();
    sie::set_ssoft();
    sie::set_stimer();

    stvec::write(trap_base as usize, stvec::TrapMode::Direct);
}
