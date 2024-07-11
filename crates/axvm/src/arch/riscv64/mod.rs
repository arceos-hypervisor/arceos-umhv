mod detect;
pub mod csrs;
mod vcpu;
mod regs;
pub mod sbi;
mod devices;
mod vmexit;
mod vm_pages;
mod device_list;

pub(crate) use self::detect::detect_h_extension as has_hardware_support;
pub use self::PerCpu as AxArchPerCpuState;
pub use self::vcpu::VCpu as AxArchVCpu;
pub use self::vcpu::VCpuConfig as AxArchVCpuConfig;
pub use self::device_list::DeviceList as AxArchDeviceList;
use self::csrs::{CSR, traps, RiscvCsrTrait};
use axerrno::AxResult;
use axerrno::AxError;

use crate::{
    AxVMHal,
};

pub struct PerCpu<H: AxVMHal> {
    _marker: core::marker::PhantomData<H>,
}

impl<H: AxVMHal> PerCpu<H> {
    pub fn new(_cpu_id: usize) -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }

    pub fn is_enabled(&self) -> bool {
        unimplemented!()
    }

    pub fn hardware_enable(&mut self) -> AxResult<()> {
        if has_hardware_support() {
            unsafe {
                setup_csrs();
            }
            Ok(())
        } else {
            Err(AxError::Unsupported)
        }
    }

    pub fn hardware_disable(&mut self) -> AxResult<()> {
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