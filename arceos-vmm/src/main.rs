#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

extern crate axstd as std;

mod hal;
mod task;
mod vmm;
use arceos::modules::axhal;
use std::os::arceos;
#[unsafe(no_mangle)]
fn main() {
    info!("Starting virtualization...");

    debug!("Hardware support: {:?}", axvm::has_hardware_support());
    timer_init();
    hal::enable_virtualization();

    info!("Hardware virtualization enabled");
    vmm::init();

    vmm::start();

    info!("VMM shutdown");
}

fn timer_init() {
    axhal::irq::set_enable(26, true);
    axhal::arch::enable_irqs();
    axdevice::timer::init();
    axhal::irq::register_handler(26, || {
        axdevice::timer::scheduler_next_event();
        axdevice::timer::check_events();
    });

    debug!("VMM init done");
}
