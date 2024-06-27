#![no_std]
#![feature(asm_const)]
#![feature(concat_idents)]
#![feature(naked_functions)]
#![feature(const_trait_impl)]

extern crate alloc;
#[macro_use]
extern crate log;

mod hal;
mod mm;

pub mod arch;

use arch::ArchPerCpuState;
use axerrno::{ax_err, AxResult};

pub use arch::AxvmVcpu;
pub use hal::AxvmHal;
pub use mm::{AxNestedPageTable, NestedPageFaultInfo};
pub use mm::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr, HostVirtAddr};

/// Whether the hardware has virtualization support.
pub fn has_hardware_support() -> bool {
    arch::has_hardware_support()
}

/// Host per-CPU states to run the guest. All methods must be called on the corresponding CPU.
pub struct AxvmPerCpu<H: AxvmHal> {
    _cpu_id: usize,
    arch: ArchPerCpuState<H>,
}

impl<H: AxvmHal> AxvmPerCpu<H> {
    /// Create an uninitialized instance.
    pub fn new(cpu_id: usize) -> Self {
        Self {
            _cpu_id: cpu_id,
            arch: ArchPerCpuState::new(),
        }
    }

    /// Whether the current CPU has hardware virtualization enabled.
    pub fn is_enabled(&self) -> bool {
        self.arch.is_enabled()
    }

    /// Enable hardware virtualization on the current CPU.
    pub fn hardware_enable(&mut self) -> AxResult {
        self.arch.hardware_enable()
    }

    /// Disable hardware virtualization on the current CPU.
    pub fn hardware_disable(&mut self) -> AxResult {
        self.arch.hardware_disable()
    }

    /// Create a [`AxvmVcpu`], set the entry point to `entry`, set the nested
    /// page table root to `npt_root`.
    pub fn create_vcpu(
        &self,
        entry: GuestPhysAddr,
        npt_root: HostPhysAddr,
    ) -> AxResult<AxvmVcpu<H>> {
        if !self.is_enabled() {
            ax_err!(BadState, "virtualization is not enabled")
        } else {
            AxvmVcpu::new(&self.arch, entry, npt_root)
        }
    }
}

impl<H: AxvmHal> Drop for AxvmPerCpu<H> {
    fn drop(&mut self) {
        if self.is_enabled() {
            self.hardware_disable().unwrap();
        }
    }
}
