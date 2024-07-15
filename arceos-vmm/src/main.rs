#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(naked_functions)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

extern crate alloc;

#[macro_use]
extern crate log;

// mod device_emu;
// mod gconfig;
mod gpm;
mod hal;
// mod vmexit;

use axerrno::{AxError, AxResult};
use axhal::mem::virt_to_phys;
use axvm::arch::AxArchVCpuConfig;
use axvm::config::{AxVCpuConfig, AxVMConfig};
use axvm::{AxVM, AxVMPerCpu, GuestPhysAddr, HostPhysAddr, HostVirtAddr};
use page_table_entry::MappingFlags;

// use self::gconfig::*;
use self::gpm::{setup_gpm, GuestMemoryRegion, GuestPhysMemorySet, GUEST_ENTRY};
use self::hal::AxvmHalImpl;

#[percpu::def_percpu]
pub static mut AXVM_PER_CPU: AxVMPerCpu<AxvmHalImpl> = AxVMPerCpu::new_uninit();

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let percpu = unsafe { AXVM_PER_CPU.current_ref_mut_raw() };
    percpu.init(0).expect("Failed to initialize percpu state");
    percpu.hardware_enable();

    let gpm = setup_gpm().expect("Failed to set guest physical memory set");
    debug!("{:#x?}", gpm);

    let config = AxVMConfig {
        cpu_count: 1,
        cpu_config: AxVCpuConfig {
            arch_config: AxArchVCpuConfig {},
            ap_entry: GUEST_ENTRY,
            bsp_entry: GUEST_ENTRY,
        },
        gpm: gpm.nest_page_table_root(),
        // gpm : 0.into(),
    };

    let vm = AxVM::<AxvmHalImpl>::new(config, 0).expect("Failed to create VM");
    vm.boot().unwrap()
}
