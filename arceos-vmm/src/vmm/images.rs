use alloc::string::String;

use std::fs::File;

use axerrno::{ax_err, ax_err_type, AxResult};
use memory_addr::VirtAddr;

use axvm::config::AxVMCrateConfig;

use crate::vmm::VMRef;

// Linkage symbols from the guest.S.
extern "C" {
    #[linkage = "extern_weak"]
    static guestdtb_start: Option<unsafe extern "C" fn()>;
    // fn guestdtb_end();
    #[linkage = "extern_weak"]
    static guestkernel_start: Option<unsafe extern "C" fn()>;
    // fn guestkernel_end();
}

/// Loads the VM image files.
pub fn load_vm_images(config: AxVMCrateConfig, vm: VMRef) -> AxResult {
    match config.image_location.as_deref() {
        Some("memory") => load_vm_images_memory(config, vm),
        _ => load_vm_images_filesystem(config, vm),
    }
}

/// Loads the VM image files from the filesystem
/// into the guest VM's memory space based on the VM configuration.
fn load_vm_images_filesystem(config: AxVMCrateConfig, vm: VMRef) -> AxResult {
    info!("Loading VM images from filesystem");
    // Load kernel image.
    load_vm_image(
        config.kernel_path,
        VirtAddr::from(config.kernel_load_addr),
        vm.clone(),
    )?;
    // Load BIOS image if needed.
    if let Some(bios_path) = config.bios_path {
        if let Some(bios_load_addr) = config.bios_load_addr {
            load_vm_image(bios_path, VirtAddr::from(bios_load_addr), vm.clone())?;
        } else {
            return ax_err!(NotFound, "BIOS load addr is missed");
        }
    };
    // Load Ramdisk image if needed.
    if let Some(ramdisk_path) = config.ramdisk_path {
        if let Some(ramdisk_load_addr) = config.ramdisk_load_addr {
            load_vm_image(ramdisk_path, VirtAddr::from(ramdisk_load_addr), vm.clone())?;
        } else {
            return ax_err!(NotFound, "Ramdisk load addr is missed");
        }
    };
    // Load DTB image if needed.
    // Todo: generate DTB file for guest VM.
    if let Some(dtb_path) = config.dtb_path {
        if let Some(dtb_load_addr) = config.dtb_load_addr {
            load_vm_image(dtb_path, VirtAddr::from(dtb_load_addr), vm.clone())?;
        } else {
            return ax_err!(NotFound, "DTB load addr is missed");
        }
    };
    Ok(())
}

fn load_vm_image(image_path: String, image_load_gpa: VirtAddr, vm: VMRef) -> AxResult {
    use std::io::{BufReader, Read};
    let (image_file, image_size) = open_image_file(image_path.as_str())?;

    let image_load_regions = vm.get_image_load_region(image_load_gpa, image_size)?;
    let mut file = BufReader::new(image_file);

    for buffer in image_load_regions {
        file.read_exact(buffer).map_err(|err| {
            ax_err_type!(
                Io,
                format!("Failed in reading from file {}, err {:?}", image_path, err)
            )
        })?
    }

    Ok(())
}

fn open_image_file(file_name: &str) -> AxResult<(File, usize)> {
    let file = File::open(file_name).map_err(|err| {
        ax_err_type!(
            NotFound,
            format!(
                "Failed to open {}, err {:?}, please check your disk.img",
                file_name, err
            )
        )
    })?;
    let file_size = file
        .metadata()
        .map_err(|err| {
            ax_err_type!(
                Io,
                format!(
                    "Failed to get metadate of file {}, err {:?}",
                    file_name, err
                )
            )
        })?
        .size() as usize;
    Ok((file, file_size))
}

/// Load VM images from memory (guest.S)
/// into the guest VM's memory space based on the VM configuration.
fn load_vm_images_memory(config: AxVMCrateConfig, vm: VMRef) -> AxResult {
    info!("Loading VM images from memory");

    if let Some(value) = unsafe { guestdtb_start } {
        let dtb_start_addr = value as usize;

        // Load DTB image
        if let Some(_dtb_path) = config.dtb_path {
            load_vm_image_memory(
                dtb_start_addr as *mut u8,
                config.dtb_load_addr.unwrap(),
                config.dtb_size.unwrap(),
                vm.clone(),
            )
            .expect("Failed to load DTB images");
        }
    }
    if let Some(value) = unsafe { guestkernel_start } {
        let kernel_start_addr = value as usize;
        // Load kernel image.
        load_vm_image_memory(
            kernel_start_addr as *mut u8,
            config.kernel_load_addr,
            config.kernel_size.unwrap(),
            vm.clone(),
        )
        .expect("Failed to load VM images");
    } else {
        panic!("VM images is missed, add `VM_CONFIGS=configs/aarch64-linux.toml` in make command.");
    }

    Ok(())
}

fn load_vm_image_memory(
    buffer: *mut u8,
    load_addr: usize,
    image_size: usize,
    vm: VMRef,
) -> AxResult {
    // let image_size = buffer.len();
    let mut buffer_pos = 0;
    let image_load_gpa = VirtAddr::from(load_addr);

    let image_load_regions = vm.get_image_load_region(image_load_gpa, image_size)?;

    for region in image_load_regions {
        let region_len = region.len();
        let bytes_to_write = region_len.min(image_size - buffer_pos);

        // copy data from .tbdata section
        unsafe {
            core::ptr::copy_nonoverlapping(
                buffer.offset(buffer_pos as isize),
                (&mut region[0]) as *mut u8,
                bytes_to_write,
            );
        }

        // Update the position of the buffer.
        buffer_pos += bytes_to_write;

        // If the buffer is fully written, exit the loop.
        if buffer_pos >= image_size {
            debug!("copy size: {}", bytes_to_write);
            break;
        }
    }

    Ok(())
}
