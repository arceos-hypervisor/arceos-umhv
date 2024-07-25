#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(naked_functions)]
#![allow(warnings)]
#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

extern crate alloc;

#[macro_use]
extern crate log;

// mod device_emu;
mod config;
mod gpm;
mod hal;
// mod vmexit; temporarily removed

use alloc::vec::Vec;

use axerrno::{AxError, AxResult};
use axvm::config::{AxVCpuConfig, AxVMConfig, AxVMCrateConfig};
use axvm::{AxVM, AxVMPerCpu, GuestPhysAddr, HostPhysAddr, HostVirtAddr};
use page_table_entry::MappingFlags;

use self::gpm::{setup_gpm, GuestMemoryRegion, GuestPhysMemorySet, GUEST_ENTRY};
use self::hal::AxVMHalImpl;
use alloc::vec;

#[percpu::def_percpu]
pub static mut AXVM_PER_CPU: AxVMPerCpu<AxVMHalImpl> = AxVMPerCpu::new_uninit();

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Starting virtualization...");

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
    let raw_vm_config = core::include_str!("../configs/nimbos.toml");
    let vm_create_config =
        AxVMCrateConfig::from_toml(raw_vm_config).expect("Failed to resolve VM config");

    let gpm = setup_gpm().expect("Failed to set guest physical memory set");
    debug!("{:#x?}", gpm);

    let config = AxVMConfig::from(vm_create_config);

    let vm = AxVM::<AxVMHalImpl>::new(config, 0, gpm.nest_page_table_root())
        .expect("Failed to create VM");
    info!("Boot VM...");

    // Todo: remove this, details can be get from 
    // this [PR](https://github.com/arceos-hypervisor/arceos-umhv/pull/5). 
    vm.boot().unwrap();
    panic!("VM boot failed")
}
