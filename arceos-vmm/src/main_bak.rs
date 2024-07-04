#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(naked_functions)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

extern crate alloc;

#[macro_use]
extern crate log;

mod device_emu;
mod gconfig;
mod gpm;
mod hal;
mod vmexit;

use axerrno::{AxError, AxResult};
use axhal::mem::virt_to_phys;
use axvm::{AxvmPerCpu, GuestPhysAddr, HostPhysAddr, HostVirtAddr};
use page_table_entry::MappingFlags;

use self::gconfig::*;
use self::gpm::{GuestMemoryRegion, GuestPhysMemorySet};
use self::hal::AxvmHalImpl;



#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Starting virtualization...");
    info!("Hardware support: {:?}", axvm::has_hardware_support());

    let mut percpu = AxvmPerCpu::<AxvmHalImpl>::new(0);
    percpu
        .hardware_enable()
        .expect("Failed to enable virtualization");

    let gpm = setup_gpm().expect("Failed to set guest physical memory set");
    debug!("{:#x?}", gpm);
    let mut vcpu = percpu
        .create_vcpu(GUEST_ENTRY, gpm.nest_page_table_root())
        .expect("Failed to create vcpu");

    debug!("{:#x?}", vcpu);

    println!("Running guest...");

    vcpu.run();
}
