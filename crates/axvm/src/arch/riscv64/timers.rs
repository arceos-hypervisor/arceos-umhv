extern crate alloc;

use alloc::boxed::Box;
use axhal::time::wall_time;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use timer_list::{TimeValue, TimerEvent, TimerList};

// TODO:complete TimerEventFn: including guest owmer, ... 
pub struct TimerEventFn(Box<dyn FnOnce(TimeValue) + Send + 'static>);

impl TimerEventFn {
    /// Constructs a new [`TimerEventFn`] from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(TimeValue) + Send + 'static,
    {
        Self(Box::new(f))
    }
}

impl TimerEvent for TimerEventFn {
    fn callback(self, now: TimeValue) {
        (self.0)(now)
    }
}

#[percpu::def_percpu]
static TIMER_LIST: LazyInit<SpinNoIrq<TimerList<TimerEventFn>>> = LazyInit::new();

pub fn register_timer(deadline: usize, handler: TimerEventFn) {
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw()};
    let mut timers = timer_list.lock();
    timers.set(TimeValue::from_nanos(deadline as u64), handler);
}

// pub fn unregister_timer() {
//     let mut timers = TIMER_LIST.lock();
//     timers.cancel(|t| Arc::ptr_eq(&t.0, task));
// }

pub fn check_events() {
    loop {
        let now = wall_time();
        let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw()};
        let event = timer_list.lock().expire_one(now);
        if let Some((_deadline, event)) = event {
            // info!("pick one to handler!!!");
            event.callback(now);
        } else {
            break;
        }
    }
}

const PERIODIC_INTERVAL_NANOS: u64 =
    axhal::time::NANOS_PER_SEC / axconfig::TICKS_PER_SEC as u64;

pub fn scheduler_next_event() {
    // info!("set deadline!!!");
    let now_ns = axhal::time::monotonic_time_nanos();
    let deadline = now_ns + PERIODIC_INTERVAL_NANOS;
    axhal::time::set_oneshot_timer(deadline);
}

pub fn init() {
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw()};
    timer_list.init_once(SpinNoIrq::new(TimerList::new()));
}
