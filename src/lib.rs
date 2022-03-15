#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

pub use hal::pac;
pub use stm32f4xx_hal as hal;

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

mod led;
pub use led::Led;

pub mod door;
pub use door::{DoorState, OvenDoor};

mod heater;
pub use heater::Heater;
