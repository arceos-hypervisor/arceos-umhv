use axtask::{AxTaskRef, TaskExtRef, TaskInner};
use axvm::AxVMRef;

use crate::hal::AxVMHalImpl;
use crate::task::TaskExt;

const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB

pub fn setup_vm_vcpus(vm: AxVMRef<AxVMHalImpl>) {
    for vcpu in vm.vcpu_list() {
        info!("Spawning task for Vcpu[{}]", vcpu.id());

        let mut task = TaskInner::new(
            || {
                let curr = axtask::current();
                let vcpu = unsafe { curr.task_ext().vcpu.clone() };
                let vm = unsafe { curr.task_ext().vm.clone() };

                vcpu.bind().unwrap_or_else(|err| {
                    warn!("VCpu {} failed to bind, {:?}", vcpu.id(), err);
                    axtask::exit(err.code());
                });

                loop {
                    // todo: device access
                    let exit_reason = vcpu.run().unwrap_or_else(|err| {
                        warn!("VCpu {} failed to run, {:?}", vcpu.id(), err);
                        axtask::exit(err.code());
                    });

                    let device_list = vm.get_device_list();
                    device_list.vmexit_handler(vcpu.get_arch_vcpu(), exit_reason);
                }
            },
            format!("Vcpu[{}]", vcpu.id()),
            KERNEL_STACK_SIZE,
        );

        task.init_task_ext(TaskExt::new(vm.clone(), vcpu.clone()));
        axtask::spawn_task(task);
    }
}
