use std::os::arceos;

use page_table_multiarch::PagingHandler;

use arceos::modules::axhal;
use axaddrspace::{HostPhysAddr, HostVirtAddr};
use axvcpu::AxVCpuHal;
use axvm::{AxVMHal, AxVMPerCpu};

/// Implementation for `AxVMHal` trait.
pub struct AxVMHalImpl;

impl AxVMHal for AxVMHalImpl {
    type PagingHandler = axhal::paging::PagingHandlerImpl;

    fn virt_to_phys(vaddr: HostVirtAddr) -> HostPhysAddr {
        axhal::mem::virt_to_phys(vaddr)
    }

    fn current_time_nanos() -> u64 {
        axhal::time::monotonic_time_nanos()
    }
}

pub struct AxVCpuHalImpl;

impl AxVCpuHal for AxVCpuHalImpl {
    fn alloc_frame() -> Option<HostPhysAddr> {
        <AxVMHalImpl as AxVMHal>::PagingHandler::alloc_frame()
    }

    fn dealloc_frame(paddr: HostPhysAddr) {
        <AxVMHalImpl as AxVMHal>::PagingHandler::dealloc_frame(paddr)
    }

    #[inline]
    fn phys_to_virt(paddr: HostPhysAddr) -> HostVirtAddr {
        <AxVMHalImpl as AxVMHal>::PagingHandler::phys_to_virt(paddr)
    }

    fn virt_to_phys(vaddr: axaddrspace::HostVirtAddr) -> axaddrspace::HostPhysAddr {
        std::os::arceos::modules::axhal::mem::virt_to_phys(vaddr)
    }

    #[cfg(target_arch = "aarch64")]
    fn irq_fecth() -> usize {
        axhal::irq::fetch_irq()
    }

    fn irq_hanlder() {
        todo!()
    }
}

#[percpu::def_percpu]
static mut AXVM_PER_CPU: AxVMPerCpu<AxVCpuHalImpl> = AxVMPerCpu::<AxVCpuHalImpl>::new_uninit();

/// Init hardware virtualization support in each core.
pub(crate) fn enable_virtualization() {
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering;

    use std::thread;

    use arceos::api::config;
    use arceos::api::task::{ax_set_current_affinity, AxCpuMask};
    use arceos::modules::axhal::cpu::this_cpu_id;

    static CORES: AtomicUsize = AtomicUsize::new(0);

    for cpu_id in 0..config::SMP {
        thread::spawn(move || {
            // Initialize cpu affinity here.
            assert!(
                ax_set_current_affinity(AxCpuMask::one_shot(cpu_id)).is_ok(),
                "Initialize CPU affinity failed!"
            );

            let percpu = unsafe { AXVM_PER_CPU.current_ref_mut_raw() };
            percpu
                .init(this_cpu_id())
                .expect("Failed to initialize percpu state");
            percpu
                .hardware_enable()
                .expect("Failed to enable virtualization");

            info!("Hardware virtualization support enabled on core {}", cpu_id);

            let _ = CORES.fetch_add(1, Ordering::Release);

            thread::yield_now();
        });
    }

    thread::yield_now();

    // Wait for all cores to enable virtualization.
    while CORES.load(Ordering::Acquire) != config::SMP {
        core::hint::spin_loop();
    }
}

/// Since we re-introduce [`axvcpu::AxVCpuHal`], why do we still need [`arm_vcpu::HalIf`]?
///
/// I've tried, there is a `irq_hanlder` in [`arm_vcpu::HalIf`], but it can not be used for vcpu to call
/// the irq handler provided by host OS (like `axhal::irq::handler_irq(irq_num)`).
///
/// The key reason is that a generic type should be applied to a specific struct,
/// while we need to call this `irq_handler` through symbol link in [`arm_vcpu`] crate's exception handling routine.
#[cfg(target_arch = "aarch64")]
mod hal_arm {
    use std::os::arceos::modules::axhal;

    /// Implementation for `HalIf` trait provided by [aarch64_vcpu](https://github.com/arceos-hypervisor/aarch64_vcpu) crate.
    struct HalIfImpl;

    #[crate_interface::impl_interface]
    impl arm_vcpu::HalIf for HalIfImpl {
        fn irq_hanlder() {
            let irq_num = axhal::irq::fetch_irq();
            warn!("IRQ handler {irq_num}");
            axhal::irq::handler_irq(irq_num);
        }
    }
}
