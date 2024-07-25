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
mod gpm;
mod hal;
mod task;
// mod vmexit; temporarily removed

use alloc::sync::Arc;
use alloc::vec::Vec;

use axerrno::{AxError, AxResult};
use axvm::config::{AxArchVCpuConfig, AxVCpuConfig, AxVMConfig};
use axvm::{AxVM, AxVMPerCpu, GuestPhysAddr, HostPhysAddr, HostVirtAddr};
use page_table_entry::MappingFlags;

use self::gpm::{setup_gpm, GuestMemoryRegion, GuestPhysMemorySet, GUEST_ENTRY};
pub use self::hal::AxVMHalImpl;

#[percpu::def_percpu]
pub static mut AXVM_PER_CPU: AxVMPerCpu<AxVMHalImpl> = AxVMPerCpu::new_uninit();

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Starting virtualization...");
    info!("Hardware support: {:?}", axvm::has_hardware_support());

    let percpu = unsafe { AXVM_PER_CPU.current_ref_mut_raw() };
    percpu.init(0).expect("Failed to initialize percpu state");
    percpu
        .hardware_enable()
        .expect("Failed to enable virtualization");

    let gpm = setup_gpm().expect("Failed to set guest physical memory set");
    debug!("{:#x?}", gpm);

    let config = AxVMConfig {
        cpu_count: 1,
        cpu_config: AxVCpuConfig {
            arch_config: AxArchVCpuConfig {
                setup_config: (),
                create_config: (),
            },
            ap_entry: GUEST_ENTRY,
            bsp_entry: GUEST_ENTRY,
        },
        // gpm: gpm.nest_page_table_root(),
        // gpm : 0.into(),
    };

    let vm: std::sync::Arc<AxVM<AxVMHalImpl>> =
        AxVM::<AxVMHalImpl>::new(config, 0, gpm.nest_page_table_root())
            .expect("Failed to create VM");

    use self::task::TaskExt;
    use axtask::{AxTaskRef, TaskExtRef, TaskInner};

    const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB

    for vcpu_for_task in vm.vcpu_list() {
        let mut task = TaskInner::new(
            || {
                let curr = axtask::current();
                let vcpu = unsafe { curr.task_ext().vcpu.clone() };
                let vm = unsafe { curr.task_ext().vm.clone() };

                vcpu.bind().unwrap_or_else(|err| {
                    warn!("VCpu {} failed to bind, {:?}", vcpu.id(), err);
                    axtask::exit(err.code());
                });

                loop {
                    // todo: device access
                    let exit_reason = vcpu.run().unwrap_or_else(|err| {
                        warn!("VCpu {} failed to run, {:?}", vcpu.id(), err);
                        axtask::exit(err.code());
                    });

                    let device_list = vm.get_device_list();
                    device_list.vmexit_handler(vcpu.get_arch_vcpu(), exit_reason);
                }
            },
            format!("Vcpu[{}]", vcpu_for_task.id()),
            KERNEL_STACK_SIZE,
        );

        task.init_task_ext(TaskExt::new(vm.clone(), vcpu_for_task.clone()));
        axtask::spawn_task(task);
    }

    info!("Boot VM...");
    vm.boot().unwrap();
    panic!("VM boot failed")
}
