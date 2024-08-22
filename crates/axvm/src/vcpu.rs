//! Architecture dependent vcpu implementations.

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        pub use x86_vcpu::VmxArchVCpu as AxArchVCpuImpl;
        pub use x86_vcpu::VmxArchPerCpuState as AxVMArchPerCpuImpl;
        pub use x86_vcpu::has_hardware_support;

        pub use x86_vcpu::PhysFrameIf;
    } else if #[cfg(target_arch = "riscv64")] {
        mod riscv64;
        pub use self::riscv64::*;
    } else if #[cfg(target_arch = "aarch64")] {
        pub use arm_vcpu::Aarch64VCpu as AxArchVCpuImpl;
        pub use arm_vcpu::Aarch64PerCpu as AxVMArchPerCpuImpl;
        pub use arm_vcpu::has_hardware_support;
    }
}
