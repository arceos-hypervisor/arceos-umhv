use alloc::sync::Arc;

use axvm::{AxVM, VCpu};

use super::hal::AxVMHalImpl;

/// Task extended data for the monolithic kernel.
pub struct TaskExt {
    /// The VM.
    pub vm: Arc<AxVM<AxVMHalImpl>>,
    /// The virtual memory address space.
    pub vcpu: Arc<VCpu<AxVMHalImpl>>,
}

impl TaskExt {
    pub const fn new(vm: Arc<AxVM<AxVMHalImpl>>, vcpu: Arc<VCpu<AxVMHalImpl>>) -> Self {
        Self { vm, vcpu }
    }
}

axtask::def_task_ext!(TaskExt);
