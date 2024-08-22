use axvm::AxVMHal;
// Todo: should we know about HostPhysAddr and HostVirtAddr here???
use memory_addr::{PhysAddr, VirtAddr};

/// Implementation for `AxVMHal` trait.
pub struct AxVMHalImpl;

impl AxVMHal for AxVMHalImpl {
    type PagingHandler = axhal::paging::PagingHandlerImpl;

    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        axhal::mem::virt_to_phys(vaddr)
    }

    fn current_time_nanos() -> u64 {
        axhal::time::monotonic_time_nanos()
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        use page_table_multiarch::PagingHandler;

        /// Implementation for `PhysFrameIf` trait provided by [x86_vcpu](https://github.com/arceos-hypervisor/x86_vcpu) crate.
        struct PhysFrameIfImpl;

        #[crate_interface::impl_interface]
        impl axvm::PhysFrameIf for PhysFrameIfImpl {
            fn alloc_frame() -> Option<PhysAddr> {
                <AxVMHalImpl as AxVMHal>::PagingHandler::alloc_frame()
            }

            fn dealloc_frame(paddr: PhysAddr) {
                <AxVMHalImpl as AxVMHal>::PagingHandler::dealloc_frame(paddr)
            }

            #[inline]
            fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
                <AxVMHalImpl as AxVMHal>::PagingHandler::phys_to_virt(paddr)
            }
        }
    }
}
