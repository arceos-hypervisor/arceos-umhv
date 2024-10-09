use axvm::config::{AxVMConfig, AxVMCrateConfig};

use crate::vmm::{images::load_vm_images_buffer, vm_list::push_vm, VM};

// const NIMBOS_KERNEL_SIZE: usize = 634880; // 620k
// const NIMBOS_MEM_SIZE: usize = 0x40_0000; // 4M
// const LINUX_KERNEL_SIZE: usize = 1000000;
// const LINUX_KERNEL_SIZE: usize = 552960;
// const LINUX_DTB_SIZE: usize = 161580;
// #[link_section = ".guestdata.dtb"]
// static NIMBOS_DTB: [u8; NIMBOS_DTB_SIZE] = *include_bytes!("../../guest/nimbos/nimbos-aarch64_rk3588.dtb");
// #[link_section = ".guestdata.kernel"]
// static LINUX_KERNEL: [u8; LINUX_KERNEL_SIZE] = *include_bytes!("../../../../nimbos.bin");
// static LINUX_KERNEL: [u8; LINUX_KERNEL_SIZE] = *include_bytes!("../../../../nimbos-aarch64-v3.bin");
// #[link_section = ".guestdata.dtb"]
// static LINUX_DTB: [u8; LINUX_DTB_SIZE] = *include_bytes!("../../../../rk3588.dtb");
// #[link_section = ".guestdata.mem"]
// static NIMBOS_MEM: [u8; NIMBOS_MEM_SIZE] = [0; NIMBOS_MEM_SIZE];

core::arch::global_asm!(include_str!("../../guest.S"));

extern "C" {
    fn guestdtb_start();
    fn guestdtb_end();
    fn guestkernel_start();
    fn guestkernel_end();
}

mod config {
    use alloc::vec::Vec;

    /// Default static VM configs. Used when no VM config is provided.
    pub fn default_static_vm_configs() -> Vec<&'static str> {
        vec![
            // #[cfg(target_arch = "x86_64")]
            // core::include_str!("../../configs/nimbos-x86_64.toml"),
            #[cfg(target_arch = "x86_64")]
            core::include_str!("../../configs/arceos-x86_64.toml"),
            #[cfg(target_arch = "aarch64")]
            core::include_str!("../../configs/linux-aarch64.toml"),
            // #[cfg(target_arch = "aarch64")]
            // core::include_str!("../../configs/arceos-aarch64.toml"),
            // #[cfg(target_arch = "riscv64")]
            // core::include_str!("../../configs/nimbos-riscv64.toml"),
            #[cfg(target_arch = "riscv64")]
            core::include_str!("../../configs/arceos-riscv64.toml"),
        ]
    }

    include!(concat!(env!("OUT_DIR"), "/vm_configs.rs"));
}

pub fn init_guest_vms() {
    let gvm_raw_configs = config::static_vm_configs();

    for raw_cfg_str in gvm_raw_configs {
        let vm_create_config =
            AxVMCrateConfig::from_toml(raw_cfg_str).expect("Failed to resolve VM config");
        let vm_config = AxVMConfig::from(vm_create_config.clone());

        // Create VM.
        debug!("init guest vm");
        let vm = VM::new(vm_config).expect("Failed to create VM");
        push_vm(vm.clone());

        // Load corresponding images for VM.
        info!("VM[{}] created success, loading images...", vm.id());

        let dtb_start_addr = guestdtb_start as usize;
        let kernel_start_addr = guestkernel_start as usize;
        // unsafe {
        //     copy_data(dtb_start_addr as *mut u8, vm1_dtb as *mut u8, 0x20_0000);
        //     copy_data(kernel_start_addr as *mut u8, vm1_kernel_entry as *mut u8, 0x320_0000);
        // }

        let _ = load_vm_images_buffer(
            kernel_start_addr as *mut u8,
            vm_create_config.kernel_load_addr,
            0x400_0000,
            vm.clone(),
        )
        .expect("Failed to load VM images");

        let _ = load_vm_images_buffer(
            dtb_start_addr as *mut u8,
            vm_create_config.dtb_load_addr.unwrap(),
            0x8_0000,
            vm.clone(),
        )
        .expect("Failed to load DTB images");
    }
}
