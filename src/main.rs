#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

mod hw;
use hw::*;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [USART1])]
mod app {
    use spisen::{Heater, OvenDoor};
    use stm32f4xx_hal::{
        gpio::ExtiPin,
        timer::{ExtU32, MonoTimerUs},
    };

    use super::{DoorPin, HeaterSpi, MonotonicTim, UsDelay};

    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<MonotonicTim>;

    #[shared]
    struct Resources {
        door: OvenDoor<DoorPin>,
    }

    #[local]
    struct Local {
        heater: Heater<HeaterSpi>,
        delay: UsDelay,
    }

    #[init]
    fn init(ctx: init::Context) -> (Resources, Local, init::Monotonics) {
        // Device specific peripherals
        let device = ctx.device;

        let (pin, spi, delay, mono) = super::setup_hw(device);

        // Setup the Oven door
        let door = OvenDoor::new(pin);

        // Setup the Oven heater (Neopixel array)
        let heater = Heater::new(spi);

        defmt::info!("init done: Door state: {:?}", door.state());

        (
            Resources { door },
            Local { heater, delay },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        // The idle loop
        loop {}
    }

    #[task(
        binds = EXTI0,
        shared = [door]
    )]
    fn on_door_event(ctx: on_door_event::Context) {
        let mut door = ctx.shared.door;

        match check_door::spawn_after(1.secs()) {
            Ok(handle) => Some(handle),
            Err(_err) => {
                defmt::info!("Door check already spawned");
                None
            }
        };

        // Clear the interrupt
        door.lock(|d| d.pin_mut().clear_interrupt_pending_bit());
    }

    #[task(
        capacity = 1,
        shared = [door],
    )]
    fn check_door(mut ctx: check_door::Context) {
        let open = ctx.shared.door.lock(|p| p.is_open());

        defmt::info!("Oven door open: {}", open);
    }

    #[task(
        local = [heater, delay],
    )]
    fn run_oven(ctx: run_oven::Context) {
        let heater = ctx.local.heater;
        let delay = ctx.local.delay;

        heater.rainbow(delay);
    }
}
