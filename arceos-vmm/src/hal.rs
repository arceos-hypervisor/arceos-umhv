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

/// This design might seem strange,
/// but the underlying reason is that the vCPU implementations for ARM and RISC-V architectures
/// **DO NOT** require dependency on OS-related resource management interfaces.
///
/// However, the vCPU implementation for the x86_64 architecture relies on OS-provided physical memory management interfaces to allocate memory for VMX-related control regions.
/// To avoid unnecessary Rust generic type applications, we decided to introduce `crate_interface` in the [`x86_vcpu`](https://github.com/arceos-hypervisor/x86_vcpu) crate
/// and use it to call OS-related resource allocation interfaces to implement `PhysFrameIf`.
#[cfg(target_arch = "x86_64")]
mod frame_x86 {
    use memory_addr::{PhysAddr, VirtAddr};
    use page_table_multiarch::PagingHandler;

    use axvm::AxVMHal;

    use crate::hal::AxVMHalImpl;

    /// Implementation for `PhysFrameIf` trait provided by [x86_vcpu](https://github.com/arceos-hypervisor/x86_vcpu) crate.
    struct PhysFrameIfImpl;

    #[crate_interface::impl_interface]
    impl x86_vcpu::PhysFrameIf for PhysFrameIfImpl {
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


#[cfg(target_arch = "aarch64")]
mod gic_trait {
    use vgic::GicTrait;
    use axhal::GICD;
    /// Implementation for `PhysFrameIf` trait provided by [x86_vcpu](https://github.com/arceos-hypervisor/x86_vcpu) crate.
    struct GicIfImpl;

    #[crate_interface::impl_interface]
    impl GicTrait for GicIfImpl{
        fn set_enable(vector: usize, enable: bool) {
            GICD.lock().set_enable(vector, enable);
        }
        fn get_enable(vector: usize) -> bool {
            GICD.lock().get_enable(vector)
        }
    
        fn get_typer() -> u32 {
            GICD.lock().get_typer()
        }
        fn get_iidr() -> u32 {
            GICD.lock().get_iidr()
        }
    
        fn set_state(int_id: usize, state: usize, current_cpu_id: usize) {
            GICD.lock().set_state(int_id, state, current_cpu_id);
        }
        fn get_state(int_id: usize) -> usize {
            GICD.lock().get_state(int_id)
        }
    
        fn set_icfgr(int_id: usize, cfg: u8) {
            GICD.lock().set_icfgr(int_id, cfg);
        }
    
        fn get_target_cpu(int_id: usize) -> usize {
            GICD.lock().get_target_cpu(int_id)
        }
        fn set_target_cpu(int_id: usize, target: u8) {
            GICD.lock().set_target_cpu(int_id, target);
        }
    
        fn get_priority(int_id: usize) -> usize {
            GICD.lock().get_priority(int_id)
        }
        fn set_priority(int_id: usize, priority: u8) {
            GICD.lock().set_priority(int_id, priority);
        }
    }
}
