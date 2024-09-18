extern crate alloc;

use alloc::boxed::Box;
use axtask::CurrentTask;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use timer_list::{TimeValue, TimerEvent, TimerList};

pub struct VmmTimerEvent {
    task: CurrentTask,
    timer_callback: Box<dyn FnOnce(TimeValue) + Send + 'static>,
}


impl VmmTimerEvent {
    /// Constructs a new [`VmmTimerEvent`] from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(TimeValue) + Send + 'static,
    {
        Self {
            task: axtask::current(),
            timer_callback: Box::new(f),
        }
    }
}

impl TimerEvent for VmmTimerEvent {
    fn callback(self, now: TimeValue) {
        (self.timer_callback)(now)
    }
}

#[percpu::def_percpu]
static TIMER_LIST: LazyInit<SpinNoIrq<TimerList<VmmTimerEvent>>> = LazyInit::new();

// deadline: ns
pub fn register_timer(deadline: u64, handler: VmmTimerEvent) {
    // info!("2");
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    let mut timers = timer_list.lock();
    timers.set(TimeValue::from_nanos(deadline as u64), handler);
}

pub fn cancel_timer<F>(condition: F)
where
    F: Fn(&VmmTimerEvent) -> bool, {
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    let mut timers = timer_list.lock();
    timers.cancel(condition);
}

pub fn check_events() {
    // info!("1");
    loop {
        let now = axhal::time::wall_time();
        let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
        let mut event = timer_list.lock();
        let event = event.expire_one(now);
        if let Some((_deadline, event)) = event {
            event.callback(now);
        } else {
            break;
        }
    }
}

pub fn scheduler_next_event() {
    // info!("set deadline!!!");
    let now_ns = axhal::time::monotonic_time_nanos();
    // let deadline = now_ns + axhal::time::NANOS_PER_SEC/axconfig::TIMER_FREQUENCY as u64;
    let deadline = now_ns + 1000;
    // info!("now_ns:{},deadline:{}",now_ns,deadline);
    axhal::time::set_oneshot_timer(deadline);
}

pub fn init() {
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    timer_list.init_once(SpinNoIrq::new(TimerList::new()));

    // axhal::irq::register_handler(axhal::time::TIMER_IRQ_NUM, || {
    //     // info!("TIMER_IRQ_NUM handler!!!");
    //     // unsafe {
    //     //     sie::clear_stimer();
    //     // }

    //     check_events();
    //     scheduler_next_event();
    //     // unsafe {
    //     //     sie::set_stimer();
    //     // }
    // });
}
