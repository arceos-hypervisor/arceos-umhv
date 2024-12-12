extern crate alloc;

use std::os::arceos::modules::axhal;

use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use timer_list::{TimeValue, TimerEvent, TimerList};

use axdevice::AxVmTimer;

const TICKS_PER_SEC: u64 = 100;
const NANOS_PER_SEC: u64 = 1_000_000_000;
const PERIODIC_INTERVAL_NANOS: u64 = NANOS_PER_SEC / TICKS_PER_SEC;

#[percpu::def_percpu]
static TIMER_LIST: LazyInit<SpinNoIrq<AxVmTimer>> = LazyInit::new();

// Register a new timer with specified deadline (in nanoseconds) and handler
pub fn register_timer<F>(deadline: u64, handler: F) -> usize
where
    F: FnOnce(TimeValue) + Send + 'static,
{
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    let mut timers = timer_list.lock();
    timers.register_timer(deadline, handler)
}

// Check and process any pending timer events
pub fn check_events() {
    loop {
        let now = axhal::time::wall_time();
        let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
        if !timer_list.lock().check_event(now) {
            break;
        }
    }
}

// Schedule the next timer event based on the periodic interval
pub fn scheduler_next_event() {
    let now_ns = axhal::time::monotonic_time_nanos();
    let deadline = now_ns + PERIODIC_INTERVAL_NANOS;
    // info!("PHY deadline {} !!!", deadline);
    axhal::time::set_oneshot_timer(deadline);
}

// Initialize the hypervisor timer system
pub fn init() {
    info!("Initing HV Timer...");

    use arceos::api::config;
    use arceos::api::task::{ax_set_current_affinity, AxCpuMask};
    use std::os::arceos;

    use std::thread;

    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering;

    static CORES: AtomicUsize = AtomicUsize::new(0);

    for cpu_id in 0..config::SMP {
        info!("spawning CPU{} init task ...", cpu_id);
        thread::spawn(move || {
            // Initialize cpu affinity here.
            assert!(
                ax_set_current_affinity(AxCpuMask::one_shot(cpu_id)).is_ok(),
                "Initialize CPU affinity failed!"
            );

            info!("Init HV timer in CPU{}", cpu_id);

            let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
            timer_list.init_once(SpinNoIrq::new(AxVmTimer::new()));

            let _ = CORES.fetch_add(1, Ordering::Release);

            thread::yield_now();
        });
    }

    thread::yield_now();

    // Wait for all cores
    while CORES.load(Ordering::Acquire) != config::SMP {
        core::hint::spin_loop();
    }
}
