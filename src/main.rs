#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

mod hw;
use hw::*;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [USART1])]
mod app {
    use spisen::{door, Heater, OvenDoor};
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
        heater: Heater<HeaterSpi>,
    }

    #[local]
    struct Local {
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

        run_oven::spawn().unwrap();

        (
            Resources { door, heater },
            Local { delay },
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

        let _ = check_door::spawn_after(100.millis());

        // Clear the interrupt
        door.lock(|d| d.pin_mut().clear_interrupt_pending_bit());
    }

    #[task(
        shared = [door, heater],
    )]
    fn check_door(mut ctx: check_door::Context) {
        let state = ctx.shared.door.lock(|p| p.state());
        let mut heater = ctx.shared.heater;

        let _ = match state {
            door::State::Open => {
                heater.lock(|h| h.enable(false));
                //run_oven::spawn()
            },
            door::State::Closed => {
                heater.lock(|h| h.enable(true));

                //run_oven::spawn()
            },
        };

        defmt::info!("Door state: {:?}", state);
    }

    #[task(
        local = [delay],
        shared = [heater]
    )]
    fn run_oven(ctx: run_oven::Context) {
        let mut heater = ctx.shared.heater;

        heater.lock(|h| h.rainbow());

        run_oven::spawn_after(10.millis());
    }
}
