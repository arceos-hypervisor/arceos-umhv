use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use spin::Mutex;

use axtask::{AxTaskRef, TaskExtRef, TaskInner, WaitQueue};
use axvm::AxVMRef;

use crate::hal::AxVMHalImpl;
use crate::task::TaskExt;

const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB

// A global list of VMs, protected by a mutex for thread-safe access.
static VM_VCPU_TASK_WAIT_QUEUE: Mutex<BTreeMap<usize, VMVcpus>> = Mutex::new(BTreeMap::new());

pub struct VMVcpus {
    _vm_id: usize,
    wait_queue: WaitQueue,
    vcpu_task_list: Vec<AxTaskRef>,
}

impl VMVcpus {
    fn new(vm: AxVMRef<AxVMHalImpl>) -> Self {
        Self {
            _vm_id: vm.id(),
            wait_queue: WaitQueue::new(),
            vcpu_task_list: Vec::with_capacity(vm.vcpu_num()),
        }
    }

    fn add_vcpu_task(&mut self, vcpu_task: AxTaskRef) {
        self.vcpu_task_list.push(vcpu_task)
    }
}

fn wait_for_boot<F>(vm_id: usize, condition: F)
where
    F: Fn() -> bool,
{
    VM_VCPU_TASK_WAIT_QUEUE
        .lock()
        .get(&vm_id)
        .unwrap()
        .wait_queue
        .wait_until(condition);
}

pub fn setup_vm_vcpus(vm: AxVMRef<AxVMHalImpl>) {
    let vm_id = vm.id();

    VM_VCPU_TASK_WAIT_QUEUE
        .lock()
        .insert(vm_id, VMVcpus::new(vm.clone()));

    for vcpu in vm.vcpu_list() {
        info!("Spawning task for Vcpu[{}]", vcpu.id());
        let mut task = TaskInner::new(
            || {
                let curr = axtask::current();

                let vm = curr.task_ext().vm.clone();
                let vcpu = curr.task_ext().vcpu.clone();
                let vm_id = vm.id();

                info!("VM[{}] Vcpu[{}] waiting for running", vm.id(), vcpu.id());
                wait_for_boot(vm_id, || vm.running());

                info!("VM[{}] Vcpu[{}] running...", vm.id(), vcpu.id());

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
                    let _ = device_list.vmexit_handler(vcpu.get_arch_vcpu(), exit_reason);
                }
            },
            format!("VCpu[{}]", vcpu.id()),
            KERNEL_STACK_SIZE,
        );

        task.init_task_ext(TaskExt::new(vm.clone(), vcpu.clone()));
        let task_ref = axtask::spawn_task(task);

        VM_VCPU_TASK_WAIT_QUEUE
            .lock()
            .get_mut(&vm_id)
            .unwrap()
            .add_vcpu_task(task_ref);
    }
}
