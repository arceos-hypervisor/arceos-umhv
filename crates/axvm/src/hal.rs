use axaddrspace::{HostPhysAddr, HostVirtAddr};

/// The interfaces which the underlying software (kernel or hypervisor) must implement.
pub trait AxVMHal: Sized {
    type PagingHandler: page_table_multiarch::PagingHandler;

    // /// Allocates a 4K-sized contiguous physical page, returns its physical address.
    // fn alloc_page() -> Option<HostPhysAddr>;
    // /// Deallocates the given physical page.
    // fn dealloc_page(paddr: HostPhysAddr);
    // /// Converts a physical address to a virtual address which can access.
    // fn phys_to_virt(paddr: HostPhysAddr) -> HostVirtAddr;
    /// Converts a virtual address to the corresponding physical address.
    fn virt_to_phys(vaddr: HostVirtAddr) -> HostPhysAddr;
    /// VM-Exit handler.
    // fn vmexit_handler(vcpu: &mut crate::AxVMVcpu<Self>);
    /// Current time in nanoseconds.
    fn current_time_nanos() -> u64;
}
