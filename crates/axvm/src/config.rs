use alloc::string::String;
use alloc::vec::Vec;

use axerrno::AxResult;

// use axvcpu::AxArchVCpu;

// use crate::arch::AxArchVCpuImpl;
use crate::{GuestPhysAddr, HostPhysAddr};

/// A part of `AxVCpuConfig`, which represents an architecture-dependent `VCpu`.
///
/// The concrete type of configuration is defined in `AxArchVCpuImpl`.
// #[derive(Clone, Copy, Debug, Default)]
// pub struct AxArchVCpuConfig<H: AxVMHal> {
//     pub create_config: <AxArchVCpuImpl<H> as AxArchVCpu>::CreateConfig,
//     pub setup_config: <AxArchVCpuImpl<H> as AxArchVCpu>::SetupConfig,
// }

/// A part of `AxVMConfig`, which represents a `VCpu`.
#[derive(Clone, Copy, Debug, Default)]
pub struct AxVCpuConfig {
    // pub arch_config: AxArchVCpuConfig,
    pub bsp_entry: usize,
    pub ap_entry: usize,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum VMType {
    VMTHostVM = 0,
    #[default]
    VMTRTOS = 1,
    VMTLinux = 2,
}

impl From<usize> for VMType {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::VMTHostVM,
            1 => Self::VMTRTOS,
            2 => Self::VMTLinux,
            _ => {
                warn!("Unknown VmType value: {}, default to VMTRTOS", value);
                Self::default()
            }
        }
    }
}

#[derive(Debug, Default)]
struct VMImageConfig {
    kernel_load_gpa: GuestPhysAddr,

    bios_load_gpa: Option<GuestPhysAddr>,
    dtb_load_gpa: Option<GuestPhysAddr>,
    ramdisk_load_gpa: Option<GuestPhysAddr>,

    kernel_load_hpa: HostPhysAddr,

    bios_load_hpa: Option<HostPhysAddr>,
    dtb_load_hpa: Option<HostPhysAddr>,
    ramdisk_load_hpa: Option<HostPhysAddr>,
}

/// A part of `AxVMCrateConfig`, which represents a `VM`.
#[derive(Debug, Default)]
pub struct AxVMConfig {
    id: usize,
    name: String,
    vm_type: VMType,

    cpu_mask: usize,
    cpu_config: AxVCpuConfig,

    image_config: VMImageConfig,

    memory_regions: Vec<VmMemConfig>,
    // To be added: device configuration
}

impl From<AxVMCrateConfig> for AxVMConfig {
    fn from(cfg: AxVMCrateConfig) -> Self {
        Self {
            id: cfg.id,
            name: cfg.name,
            vm_type: VMType::from(cfg.vm_type),
            cpu_mask: cfg.cpu_set,
            cpu_config: AxVCpuConfig {
                bsp_entry: cfg.entry_point,
                ap_entry: cfg.entry_point,
            },
            image_config: VMImageConfig {
                kernel_load_gpa: cfg.kernel_load_addr,
                bios_load_gpa: cfg.bios_load_addr,
                dtb_load_gpa: cfg.dtb_load_addr,
                ramdisk_load_gpa: cfg.ramdisk_load_addr,
                kernel_load_hpa: HostPhysAddr::from(0xdead_beef),
                bios_load_hpa: None,
                dtb_load_hpa: None,
                ramdisk_load_hpa: None,
            },
            memory_regions: cfg.memory_regions,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct VmMemConfig {
    gpa: usize,
    size: usize,
    flags: usize,
}

/// The configuration of axvm crate. It's not used yet, may be used in the future.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AxVMCrateConfig {
    // Basic Information
    id: usize,
    name: String,
    vm_type: usize,

    // Resources.
    cpu_set: usize,

    entry_point: usize,
    kernel_path: String,
    kernel_load_addr: usize,
    bios_path: Option<String>,
    bios_load_addr: Option<usize>,
    dtb_path: Option<String>,
    dtb_load_addr: Option<usize>,
    ramdisk_path: Option<String>,
    ramdisk_load_addr: Option<usize>,
    disk_path: Option<String>,

    /// Memory Information
    memory_regions: Vec<VmMemConfig>,
    // Todo:
    // Device Information
}

impl AxVMCrateConfig {
    pub fn from_toml(raw_cfg_str: &str) -> AxResult<Self> {
        let config = toml::from_str(raw_cfg_str).map_err(|err| {
            axerrno::ax_err_type!(
                InvalidInput,
                alloc::format!("toml deserialize get err {err:?}")
            )
        })?;
        Ok(config)
    }
}
