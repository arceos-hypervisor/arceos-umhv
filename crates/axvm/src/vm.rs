use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use axerrno::{ax_err, ax_err_type, AxResult};

use crate::arch::AxArchDeviceList;
use crate::arch::AxArchVCpuImpl;
use crate::config::AxVMConfig;
use crate::{has_hardware_support, AxVMHal, HostPhysAddr};
use axvcpu::AxArchVCpu;
use axvcpu::AxVCpu;
use core::cell::UnsafeCell;

#[allow(type_alias_bounds)] // we know the bound is not enforced here, we keep it for clarity
type VCpu<H: AxVMHal> = AxVCpu<AxArchVCpuImpl<H>>;

struct AxVMInnerConst<H: AxVMHal> {
    id: usize,
    ept_root: HostPhysAddr,
    vcpu_list: Box<[VCpu<H>]>,
    // to be added: device_list: ...
    device_list: UnsafeCell<AxArchDeviceList<H>>,
}

struct AxVMInnerMut<H: AxVMHal> {
    // memory: ...
    _marker: core::marker::PhantomData<H>,
}

/// A Virtual Machine.
pub struct AxVM<H: AxVMHal> {
    inner_const: AxVMInnerConst<H>,
    inner_mut: AxVMInnerMut<H>,
}

impl<H: AxVMHal> AxVM<H> {
    // TODO: move guest memory mapping to AxVMConfig, and store GuestPhysMemorySet in AxVM
    pub fn new(config: AxVMConfig, id: usize, ept_root: HostPhysAddr) -> AxResult<Arc<Self>> {
        let result = Arc::new({
            let mut vcpu_list = Vec::with_capacity(config.cpu_num());
            for id in 0..config.cpu_num() {
                vcpu_list.push(VCpu::new(
                    id,
                    0,
                    0,
                    <AxArchVCpuImpl<H> as AxArchVCpu>::CreateConfig::default(),
                )?);
            }

            Self {
                inner_const: AxVMInnerConst {
                    id,
                    ept_root,
                    vcpu_list: vcpu_list.into_boxed_slice(),
                    device_list: UnsafeCell::new(AxArchDeviceList::<H>::new()),
                },
                inner_mut: AxVMInnerMut {
                    _marker: core::marker::PhantomData,
                },
            }
        });

        info!("VM created: id={}", result.id());
        for vcpu in result.vcpu_list() {
            let entry = if vcpu.id() == 0 {
                config.bsp_entry()
            } else {
                config.ap_entry()
            };
            vcpu.setup(
                entry,
                result.ept_root(),
                <AxArchVCpuImpl<H> as AxArchVCpu>::SetupConfig::default(),
            )?;
        }
        info!("VM setup: id={}", result.id());

        Ok(result)
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.inner_const.id
    }

    #[inline]
    pub fn vcpu(&self, vcpu_id: usize) -> Option<&VCpu<H>> {
        self.vcpu_list().get(vcpu_id)
    }

    #[inline]
    pub fn vcpu_list(&self) -> &[VCpu<H>] {
        &self.inner_const.vcpu_list
    }

    pub fn ept_root(&self) -> HostPhysAddr {
        self.inner_const.ept_root
    }

    pub fn boot(&self) -> AxResult {
        if !has_hardware_support() {
            ax_err!(Unsupported, "Hardware does not support virtualization")
        } else {
            self.run_vcpu(0)
        }
    }

    pub fn get_device_list(&self) -> &mut AxArchDeviceList<H> {
        unsafe { &mut *self.inner_const.device_list.get() }
    }

    pub fn run_vcpu(&self, vcpu_id: usize) -> AxResult {
        let vcpu = self
            .vcpu(vcpu_id)
            .ok_or_else(|| ax_err_type!(InvalidInput, "Invalid vcpu_id"))?;
        vcpu.bind()?;
        loop {
            // todo: device access
            let exit_reason = vcpu.run()?;
            let device_list = self.get_device_list();
            device_list.vmexit_handler(vcpu.get_arch_vcpu(), exit_reason);
        }
        vcpu.unbind()
    }
}
