use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use axerrno::{ax_err, AxResult};

use crate::arch::AxArchDeviceList;
use crate::config::AxVMConfig;
use crate::AxVCpu;
use crate::{has_hardware_support, AxVMHal, HostPhysAddr};
use core::cell::UnsafeCell;
use core::fmt;

struct AxVMInnerConst<H: AxVMHal> {
    id: usize,
    vcpu_list: Box<[AxVCpu<H>]>,
    // to be added: device_list: ...
    device_list: UnsafeCell<AxArchDeviceList<H>>,
    gpm: HostPhysAddr,
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

impl<H: AxVMHal> fmt::Debug for AxVM<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxVM")
            .field("id", &self.inner_const.id)
            .field("gpm", &self.inner_const.gpm)
            .finish()
    }
}

impl<H: AxVMHal> AxVM<H> {
    pub fn new(config: AxVMConfig, id: usize) -> AxResult<Arc<Self>> {
        let result = Arc::new_cyclic(|weak_self| {
            let mut vcpu_list = Vec::with_capacity(config.cpu_count);
            for vcpu_id in 0..config.cpu_count {
                vcpu_list.push(
                    AxVCpu::new(config.cpu_config, vcpu_id, weak_self.clone(), 0, 0).unwrap(),
                );
            }

            Self {
                inner_const: AxVMInnerConst {
                    id,
                    vcpu_list: vcpu_list.into_boxed_slice(),
                    device_list: UnsafeCell::new(AxArchDeviceList::<H>::new()),
                    gpm: config.gpm,
                },
                inner_mut: AxVMInnerMut {
                    _marker: core::marker::PhantomData,
                },
            }
        });

        result.init_vcpu()?;

        Ok(result)
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.inner_const.id
    }

    #[inline]
    pub fn vcpu(&self, vcpu_id: usize) -> Option<&AxVCpu<H>> {
        self.vcpu_list().get(vcpu_id)
    }

    #[inline]
    pub fn vcpu_list(&self) -> &[AxVCpu<H>] {
        &self.inner_const.vcpu_list
    }

    #[inline]
    fn init_vcpu(&self) -> AxResult {
        for vcpu in self.vcpu_list() {
            vcpu.init()?;
        }

        Ok(())
    }

    pub fn ept_root(&self) -> HostPhysAddr {
        self.inner_const.gpm
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
        loop {
            // todo: device access
            let exit_reason = self.vcpu(vcpu_id).unwrap().run()?;
            let vcpu = self.vcpu(vcpu_id).unwrap();
            let device_list = self.get_device_list();
            device_list.vmexit_handler(vcpu.get_arch_vcpu(), exit_reason);
        }
    }
}
