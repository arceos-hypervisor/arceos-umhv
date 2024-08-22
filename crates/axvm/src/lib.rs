#![no_std]
#![feature(asm_const)]
#![feature(concat_idents)]
#![feature(naked_functions)]
#![feature(const_trait_impl)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]

//! This crate provides a minimal VM monitor (VMM) for running guest VMs.
//!
//! This crate contains:
//! - [`AxVM`]: The main structure representing a VM.
//!   - It currently handles only virtual CPUs
//!   - Memory mapping and device emulation are not implemented yet
//! - Implementations for [`axvcpu::AxArchVCpu`]: The architecture-dependent part of a virtual CPU.

extern crate alloc;
#[macro_use]
extern crate log;

mod hal;
mod vcpu;
mod vm;

// pub mod arch;
pub mod config;

pub use hal::AxVMHal;
pub use vm::AxVCpuRef;
pub use vm::AxVM;
pub use vm::AxVMRef;

pub type AxVMPerCpu = axvcpu::AxPerCpu<vcpu::AxVMArchPerCpuImpl>;

pub use vcpu::PhysFrameIf;

/// Whether the hardware has virtualization support.
pub fn has_hardware_support() -> bool {
    vcpu::has_hardware_support()
}
