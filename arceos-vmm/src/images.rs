use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use axerrno::{AxError, AxResult};
use memory_addr::VirtAddr;

use axvm::config::AxVMCrateConfig;
use axvm::AxVM;

use crate::hal::AxVMHalImpl;

pub fn load_vm_images(config: AxVMCrateConfig, vm: Arc<AxVM<AxVMHalImpl>>) -> AxResult {
    let (kernel_path, bios_path, dtb_path, ramdisk_path) = config.get_images_path();
    let (kernel_load_hva, bios_load_hva, dtb_load_hva, ramdisk_load_hva) =
        vm.get_images_load_addrs()?;

    // Load kernel image.
    load_image_from_file_system_vectored(kernel_path.as_str(), kernel_load_hva)?;

    // Load BIOS image if needed.
    if let Some(bios_path) = bios_path {
        load_image_from_file_system(
            bios_path.as_str(),
            bios_load_hva.expect("Failed to get BIOS load address"),
        )?;
    };
    // Load DTB image if needed.
    if let Some(dtb_path) = dtb_path {
        load_image_from_file_system(
            dtb_path.as_str(),
            dtb_load_hva.expect("Failed to get DTB load address"),
        )?;
    };
    // Load Ramdisk image if needed.
    if let Some(ramdisk_path) = ramdisk_path {
        load_image_from_file_system(
            ramdisk_path.as_str(),
            ramdisk_load_hva.expect("Failed to get ramdisk load address"),
        )?;
    };

    Ok(())
}

fn load_image_from_file_system(file_name: &str, addr: VirtAddr) -> AxResult {
    use std::io::{BufReader, Read};
    let file = std::fs::File::open(file_name).map_err(|err| {
        warn!(
            "Failed to open {}, err {:?}, please check your disk.img",
            file_name, err
        );
        AxError::NotFound
    })?;
    let file_size = file
        .metadata()
        .map_err(|err| {
            warn!(
                "Failed to get metadate of file {}, err {:?}",
                file_name, err
            );
            AxError::Io
        })?
        .size() as usize;
    debug!(
        "Loading {} to {:?}, size {} Bytes",
        file_name, addr, file_size
    );
    let buffer = unsafe { core::slice::from_raw_parts_mut(addr.as_mut_ptr(), file_size) };
    let mut file = BufReader::new(file);
    file.read_exact(buffer).map_err(|err| {
        warn!("Failed to read from file {}, err {:?}", file_name, err);
        AxError::Io
    })?;
    Ok(())
}

fn load_image_from_file_system_vectored(
    file_name: &str,
    buffers: Vec<&'static mut [u8]>,
) -> AxResult {
    use std::io::{BufReader, Read};
    let file = std::fs::File::open(file_name).map_err(|err| {
        warn!(
            "Failed to open {}, err {:?}, please check your disk.img",
            file_name, err
        );
        AxError::NotFound
    })?;
    let file_size = file
        .metadata()
        .map_err(|err| {
            warn!(
                "Failed to get metadate of file {}, err {:?}",
                file_name, err
            );
            AxError::Io
        })?
        .size() as usize;
    debug!("Loading {} vectored, size {} Bytes", file_name, file_size);

    let mut file = BufReader::new(file);

    for buffer in buffers {
        match file.read_exact(buffer).map_err(|err| {
            warn!("Failed to read from file {}, err {:?}", file_name, err);
            AxError::Io
        }) {
            Ok(_) => {}
            Err(_) => {
                break;
            }
        }
    }

    Ok(())
}
