#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

use defmt_rtt as _;
use panic_probe as _;

pub use stm32f4xx_hal as hal;
pub use hal::pac;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

pub mod door;
pub use door::{OvenDoor};

mod heater;
pub use heater::Heater;
