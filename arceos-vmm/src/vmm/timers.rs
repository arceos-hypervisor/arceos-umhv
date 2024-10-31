extern crate alloc;

use std::os::arceos::modules::axhal;
use std::os::arceos::modules::axtask;
use std::os::arceos::modules::axtask::TaskExtRef;


use alloc::boxed::Box;
use axtask::AxTaskRef;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use timer_list::{TimeValue, TimerEvent, TimerList};


const TICKS_PER_SEC: u64 = 100;
const NANOS_PER_SEC: u64 = 1_000_000_000;
const PERIODIC_INTERVAL_NANOS: u64 = NANOS_PER_SEC / TICKS_PER_SEC;

pub struct VmmTimerEvent {
    task: AxTaskRef,
    timer_callback: Box<dyn FnOnce(TimeValue) + Send + 'static>,
}

impl VmmTimerEvent {
    /// Constructs a new [`VmmTimerEvent`] from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(TimeValue) + Send + 'static,
    {
        Self {
            task: axtask::current().as_task_ref().clone(),
            timer_callback: Box::new(f),
        }
    }
}

impl TimerEvent for VmmTimerEvent {
    fn callback(self, now: TimeValue) {
        let vcpu = self.task.task_ext().vcpu.clone();
        let to = vcpu.get_cpu_id();
        if to < 0 || to == axhal::cpu::this_cpu_id() as isize {
            (self.timer_callback)(now)
        } else {
            // TODO:给 to 发送附带参数的 IPI
            // axvcpu::send_ipi(to, vcpu, self.timer_callback, now);
        }
    }
}

#[percpu::def_percpu]
static TIMER_LIST: LazyInit<SpinNoIrq<TimerList<VmmTimerEvent>>> = LazyInit::new();

// deadline: ns
pub fn register_timer(deadline: u64, handler: VmmTimerEvent) {
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    let mut timers = timer_list.lock();
    timers.set(TimeValue::from_nanos(deadline as u64), handler);
}

pub fn cancel_timer<F>(condition: F)
where
    F: Fn(&VmmTimerEvent) -> bool,
{
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    let mut timers = timer_list.lock();
    timers.cancel(condition);
}

// pub fn inject_timer_irq(curr: AxTaskRef, callback: fn(TimeValue)) {
//     let vcpu = curr.task_ext().vcpu.clone();
//     // TODO: cpu id 在 bind / unbind 时会变化
//     let to = vcpu.get_cpu_id();
//     if to < 0 || to == this_cpu_id() {
// 	    callback();
//     } else {
//         // 给 to 发送附带参数的 IPI
//         // TODO: callback 和 irq 变为 IpiParam
// 	    axvcpu::send_ipi(to, vcpu, callback, irq);
//     }
// }

pub fn check_events() {
    loop {
        let now = axhal::time::wall_time();
        let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
        let event = timer_list.lock().expire_one(now);
        if let Some((_deadline, event)) = event {
            // info!("pick one to handler!!!");
            event.callback(now);
        } else {
            break;
        }
    }
}

pub fn scheduler_next_event() {
    // info!("set deadline!!!");
    let now_ns = axhal::time::monotonic_time_nanos();
    let deadline = now_ns + PERIODIC_INTERVAL_NANOS;
    axhal::time::set_oneshot_timer(deadline);
}

pub fn init() {
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    timer_list.init_once(SpinNoIrq::new(TimerList::new()));

    axhal::irq::register_handler(axhal::time::TIMER_IRQ_NUM, || {
        check_events();
        scheduler_next_event();
    });
}
