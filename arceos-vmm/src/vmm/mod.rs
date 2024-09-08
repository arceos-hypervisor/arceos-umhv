mod config;
mod images;
mod vcpus;
mod vm_list;

use crate::hal::AxVMHalImpl;
use axvm::{AxVM, AxVMRef};

pub type VM = AxVM<AxVMHalImpl>;
pub type VMRef = AxVMRef<AxVMHalImpl>;

use core::sync::atomic::{AtomicUsize, Ordering};

static VMM_INITED: AtomicUsize = AtomicUsize::new(0);

pub fn is_init_ok() -> bool {
    VMM_INITED.load(Ordering::Acquire) == 1
}


pub fn init() {
    // Initialize guest VM according to config file.
    config::init_guest_vms();

    // Setup vcpus, spawn axtask for VCpu.
    info!("Setting up vcpus...");
    for vm in vm_list::get_vm_list() {
        vcpus::setup_vm_vcpus(vm);
    }

    VMM_INITED.fetch_add(1, Ordering::Relaxed);
}

pub fn start() {
    info!("VMM starting, booting VMs...");
    for vm in vm_list::get_vm_list() {
        match vm.boot() {
            Ok(_) => {
                vcpus::get_vm_vcpus(vm.id()).notify_all_vcpus();
                info!("VM[{}] boot success", vm.id())
            }
            Err(err) => warn!("VM[{}] boot failed, error {:?}", vm.id(), err),
        }
    }
}
