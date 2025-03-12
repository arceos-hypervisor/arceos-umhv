use core::cell::{OnceCell, Ref, RefCell};

use alloc::boxed::Box;
use axaddrspace::{device::{self, AccessWidth}, GuestPhysAddr, GuestPhysAddrRange};
use axdevice_base::{BaseDeviceOps, DeviceRWContext, InterruptInjector};
use cpumask::CpuMask;

pub struct MockTimer {
    injector: RefCell<Option<Box<InterruptInjector>>>,
}

impl BaseDeviceOps<GuestPhysAddrRange> for MockTimer {
    fn emu_type(&self) -> axdevice_base::EmuDeviceType {
        axdevice_base::EmuDeviceType::EmuDeviceTConsole // just a placeholder
    }

    fn address_range(&self) -> GuestPhysAddrRange {
        // a placeholder
        GuestPhysAddrRange::from_start_size(0x1234_0000.into(), 0x1000)
    }

    fn handle_read(
        &self,
        addr: <GuestPhysAddrRange as device::DeviceAddrRange>::Addr,
        width: AccessWidth,
        context: DeviceRWContext,
    ) -> axerrno::AxResult<usize> {
        todo!()
    }

    fn handle_write(
        &self,
        addr: <GuestPhysAddrRange as device::DeviceAddrRange>::Addr,
        width: AccessWidth,
        val: usize,
        context: DeviceRWContext,
    ) -> axerrno::AxResult {
        todo!()
    }

    fn set_interrupt_injector(&self, injector: Box<InterruptInjector>) {
        self.injector.borrow_mut().replace(injector);
    }
}

impl MockTimer {
    pub fn new() -> Self {
        Self {
            injector: RefCell::new(None),
        }
    }

    pub fn tick(&self) {
        // Warning! Potential deadlock here
        if let Some(injector) = self.injector.borrow_mut().as_mut() {
            injector(CpuMask::one_shot(0), 0x77);
        }
    }
}

unsafe impl Send for MockTimer {}
unsafe impl Sync for MockTimer {}
