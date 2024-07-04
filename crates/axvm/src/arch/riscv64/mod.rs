mod detect;
mod csrs;
mod vcpu;
mod vmexit;
mod regs;
mod sbi;
mod devices;

pub(crate) use self::detect::detect_h_extension as has_hardware_support;
pub use self::PerCpu as ArchPerCpuState;
pub use self::vcpu::VCpu as AxvmVcpu;
pub use vmexit::VmExitInfo;
use axerrno::AxResult;

use crate::{
    AxvmHal,
};

#[repr(C)]
pub struct PerCpu<H: AxvmHal> {
    marker: core::marker::PhantomData<H>,
}

impl<H: AxvmHal> PerCpu<H> {
    pub const fn new() -> Self {
        Self {
            marker: core::marker::PhantomData,
        }
    }

    pub fn is_enabled(&self) -> bool {
        has_hardware_support()
    }

    pub fn hardware_enable(&mut self) -> AxResult {
        // TODO:
        info!("[AxVM] successed to turn on VMX.");
        Ok(())
    }

    pub fn hardware_disable(&mut self) -> AxResult {
        // TODO:
        info!("[AxVM] successed to turn off VMX.");
        Ok(())
    }
}