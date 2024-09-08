use axvm::config::{AxVMConfig, AxVMCrateConfig};

use crate::vmm::{images::load_vm_images, vm_list::push_vm, VM};

pub fn init_guest_vms() {
    // Config file for guest VM should be read into memory in a more flexible way.
    // FIXME: remove this hardcode.
    let gvm_raw_configs = vec![
        core::include_str!("../../configs/starry-x86_64.toml"),
        core::include_str!("../../configs/arceos-x86_64-sleep.toml"),
    ];

    for raw_cfg_str in gvm_raw_configs {
        let vm_create_config =
            AxVMCrateConfig::from_toml(raw_cfg_str).expect("Failed to resolve VM config");
        let vm_config = AxVMConfig::from(vm_create_config.clone());

        // Create VM.
        let vm = VM::new(vm_config).expect("Failed to create VM");
        push_vm(vm.clone());

        // Load corresponding images for VM.
        info!("VM[{}] created success, loading images...", vm.id());
        load_vm_images(vm_create_config, vm.clone()).expect("Failed to load VM images");
    }
}
