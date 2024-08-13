#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;
extern crate axstd as std;

mod hal;
mod task;
mod vmm;

use axvm::config::{AxVMConfig, AxVMCrateConfig};
use axvm::{AxVM, AxVMPerCpu};

use crate::hal::AxVMHalImpl;

#[percpu::def_percpu]
pub static mut AXVM_PER_CPU: AxVMPerCpu<AxVMHalImpl> = AxVMPerCpu::new_uninit();

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    info!("Starting virtualization...");

    // TODO: remove this to somewhere else like `percpu.hardware_enable()`.
    info!("Hardware support: {:?}", axvm::has_hardware_support());

    // Init hardware virtualization support in each core.
    // Note: This is awkward because we need to find a proper place to call this on each core.
    let percpu = unsafe { AXVM_PER_CPU.current_ref_mut_raw() };
    percpu.init(0).expect("Failed to initialize percpu state");
    percpu
        .hardware_enable()
        .expect("Failed to enable virtualization");

    // Config file for guest VM should be read into memory in a more flexible way.
    // FIXME: remove this hardcode.
    #[cfg(target_arch = "x86_64")]
    let raw_vm_config = core::include_str!("../configs/nimbos-x86.toml");
    #[cfg(target_arch = "aarch64")]
    let raw_vm_config = core::include_str!("../configs/nimbos-aarch64.toml");
    #[cfg(target_arch = "riscv64")]
    let raw_vm_config = core::include_str!("../configs/nimbos-riscv64.toml");

    let vm_create_config =
        AxVMCrateConfig::from_toml(raw_vm_config).expect("Failed to resolve VM config");

    let config = AxVMConfig::from(vm_create_config.clone());

    // Create VM.
    let vm = AxVM::<AxVMHalImpl>::new(config).expect("Failed to create VM");
    vmm::push_vm(vm.clone());

    // Load corresponding images for VM.
    info!("VM[{}] created success, loading images...", vm.id());
    vmm::load_vm_images(vm_create_config, vm.clone()).expect("Failed to load VM images");

    // Setup vcpus, spawn axtask for VCpu.
    info!("VM[{}] images load success, setting up vcpus...", vm.id());
    vmm::setup_vm_vcpus(vm.clone());

    info!("Boot VM[{}]...", vm.id());
    axtask::WaitQueue::new().wait();

    unreachable!("VM boot failed")
}
