use std::os::arceos::modules::axhal;
use crate::vmm::timers::VmmTimerEvent;
use std::os::arceos::api::config;
use spin::Mutex;
use alloc::collections::LinkedList;
use timer_list::TimerEvent;

pub enum IpiMessage {
    Timer(VmmTimerEvent),
}

const PER_CPU_IPI_MSG_QUEUE: Mutex<IpiMsgQueue> = Mutex::new(IpiMsgQueue::new());
pub static CORE_IPI_LIST: [Mutex<IpiMsgQueue>; config::SMP] = [PER_CPU_IPI_MSG_QUEUE; config::SMP];

pub struct IpiMsgQueue {
    msg_queue: LinkedList<IpiMessage>,
}

impl IpiMsgQueue {
    const fn new() -> Self {
        Self {
            msg_queue: LinkedList::new(),
        }
    }
    pub fn push(&mut self, ipi_msg: IpiMessage) {
        self.msg_queue.push_back(ipi_msg);
    }

    pub fn pop(&mut self) -> Option<IpiMessage> {
        self.msg_queue.pop_front()
    }
}

pub fn ipi_send_msg_by_core_id(target_cpu_id: usize, msg: IpiMessage) {
    let current_cpu = axhal::cpu::this_cpu_id();
    
    if target_cpu_id == current_cpu {
        warn!(
            "CPU{} try send ipi to self, something is wrong",
            current_cpu
        );
        return;
    }
    info!(
        "CPU {} send ipi to CPU{} (Linux processor ID {})",
        current_cpu, target_cpu_id, target_cpu_id
    );
    CORE_IPI_LIST[target_cpu_id].lock().push(msg);
    // Send ipi to target core through local APIC.
    axvm::send_ipi(target_cpu_id);
}

pub fn ipi_handle() {
    let current_cpu = axhal::cpu::this_cpu_id();
    let mut ipi_queue = CORE_IPI_LIST[current_cpu].lock();
    while let Some(msg) = ipi_queue.pop() {
        match msg {
            IpiMessage::Timer(vmm_timer_event) => {
                let now = axhal::time::wall_time();
                vmm_timer_event.callback(now);
            }
            _ => {
                warn!("unknown ipi message");
            }
        }
    }
}