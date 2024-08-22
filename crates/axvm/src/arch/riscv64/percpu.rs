use crate::percpu::AxVMArchPerCpu;
use axerrno::AxError;
use axerrno::AxResult;

use crate::AxVMHal;
use super::csrs::{RiscvCsrTrait, CSR};
use super::consts::traps;
use super::detect::detect_h_extension;
use super::timers;


pub struct PerCpu<H: AxVMHal> {
    _marker: core::marker::PhantomData<H>,
}

impl<H: AxVMHal> AxVMArchPerCpu for PerCpu<H> {
    fn new(cpu_id: usize) -> AxResult<Self> {
        unsafe {
            setup_csrs();
        }
        
        // #[cfg(feature = "irq")]
        if cpu_id == 0 {
            info!("register_handler");
            //TODO: dont use axhal
            axhal::irq::register_handler(axhal::time::TIMER_IRQ_NUM, || {
                // info!("TIMER_IRQ_NUM handler!!!");
                CSR.sie.read_and_clear_bits(traps::interrupt::SUPERVISOR_TIMER);
                timers::check_events();
                timers::scheduler_next_event();
                CSR.sie.read_and_set_bits(traps::interrupt::SUPERVISOR_TIMER);
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
    CSR.hedeleg.write_value(
        traps::exception::INST_ADDR_MISALIGN
            | traps::exception::BREAKPOINT
            | traps::exception::ENV_CALL_FROM_U_OR_VU
            | traps::exception::INST_PAGE_FAULT
            | traps::exception::LOAD_PAGE_FAULT
            | traps::exception::STORE_PAGE_FAULT
            | traps::exception::ILLEGAL_INST,
    );

    // Delegate all interupts.
    CSR.hideleg.write_value(
        traps::interrupt::VIRTUAL_SUPERVISOR_TIMER
            | traps::interrupt::VIRTUAL_SUPERVISOR_EXTERNAL
            | traps::interrupt::VIRTUAL_SUPERVISOR_SOFT,
    );

    // Clear all interrupts.
    CSR.hvip.read_and_clear_bits(
        traps::interrupt::VIRTUAL_SUPERVISOR_TIMER
            | traps::interrupt::VIRTUAL_SUPERVISOR_EXTERNAL
            | traps::interrupt::VIRTUAL_SUPERVISOR_SOFT,
    );

    // clear all interrupts.
    CSR.hcounteren.write_value(0xffff_ffff);

    // enable interrupt
    CSR.sie.write_value(
        traps::interrupt::SUPERVISOR_EXTERNAL
            | traps::interrupt::SUPERVISOR_SOFT
            | traps::interrupt::SUPERVISOR_TIMER,
    );
    debug!("sie: {:#x}", CSR.sie.get_value());
}