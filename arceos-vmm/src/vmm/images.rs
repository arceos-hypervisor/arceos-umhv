use axerrno::AxResult;
use memory_addr::VirtAddr;

use crate::vmm::VMRef;

fn copy_data(src: *mut u8, dst: *mut u8, size: usize) {
    unsafe {
        // copy data from .tbdata section
        core::ptr::copy_nonoverlapping(src, dst, size);
    }
}

pub fn load_vm_images_buffer(
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

        // 将数据写入当前区域
        unsafe {
            copy_data(
                buffer.offset(buffer_pos as isize),
                (&mut region[0]) as *mut u8,
                bytes_to_write,
            );
        }

        // region.copy_from_slice(&buffer[buffer_pos..buffer_pos + bytes_to_write]);

        // 更新缓冲区的位置
        buffer_pos += bytes_to_write;

        // 如果缓冲区已全部写入，退出循环
        if buffer_pos >= image_size {
            debug!("copy size: {}", bytes_to_write);
            break;
        }
    }

    Ok(())
}

// / Loads the VM image files from the filesystem
// / into the guest VM's memory space based on the VM configuration.
// pub fn load_vm_images(config: AxVMCrateConfig, vm: VMRef) -> AxResult {
//     // Load kernel image.
//     load_vm_image(
//         config.kernel_path,
//         VirtAddr::from(config.kernel_load_addr),
//         vm.clone(),
//     )?;
//     // Load BIOS image if needed.
//     if let Some(bios_path) = config.bios_path {
//         if let Some(bios_load_addr) = config.bios_load_addr {
//             load_vm_image(bios_path, VirtAddr::from(bios_load_addr), vm.clone())?;
//         } else {
//             return ax_err!(NotFound, "BIOS load addr is missed");
//         }
//     };
//     // Load Ramdisk image if needed.
//     if let Some(ramdisk_path) = config.ramdisk_path {
//         if let Some(ramdisk_load_addr) = config.ramdisk_load_addr {
//             load_vm_image(ramdisk_path, VirtAddr::from(ramdisk_load_addr), vm.clone())?;
//         } else {
//             return ax_err!(NotFound, "Ramdisk load addr is missed");
//         }
//     };
//     // Load DTB image if needed.
//     // Todo: generate DTB file for guest VM.
//     if let Some(dtb_path) = config.dtb_path {
//         if let Some(dtb_load_addr) = config.dtb_load_addr {
//             load_vm_image(dtb_path, VirtAddr::from(dtb_load_addr), vm.clone())?;
//         } else {
//             return ax_err!(NotFound, "DTB load addr is missed");
//         }
//     };
//     Ok(())
// }

// fn load_vm_image(image_path: String, image_load_gpa: VirtAddr, vm: VMRef) -> AxResult {
//     use std::io::{BufReader, Read};
//     let (image_file, image_size) = open_image_file(image_path.as_str())?;

//     let image_load_regions = vm.get_image_load_region(image_load_gpa, image_size)?;
//     let mut file = BufReader::new(image_file);

//     for buffer in image_load_regions {
//         file.read_exact(buffer).map_err(|err| {
//             ax_err_type!(

//                 Io,
//                 format!("Failed in reading from file {}, err {:?}", image_path, err)
//             )
//         })?
//     }

//     Ok(())
// }

// fn open_image_file(file_name: &str) -> AxResult<(File, usize)> {
//     let file = File::open(file_name).map_err(|err| {
//         ax_err_type!(
//             NotFound,
//             format!(
//                 "Failed to open {}, err {:?}, please check your disk.img",
//                 file_name, err
//             )
//         )
//     })?;
//     let file_size = file
//         .metadata()
//         .map_err(|err| {
//             ax_err_type!(
//                 Io,
//                 format!(
//                     "Failed to get metadate of file {}, err {:?}",
//                     file_name, err
//                 )
//             )
//         })?
//         .size() as usize;
//     Ok((file, file_size))
// }
