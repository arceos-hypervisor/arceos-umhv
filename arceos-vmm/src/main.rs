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
    axdevice::timer::init();

    debug!("VMM init done");
}
