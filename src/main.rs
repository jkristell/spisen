#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use stm32f4xx_hal as hal;

use hal::{
        gpio::{Input, Pin, PullUp}, 
    };


// A0 on the nucleo
type DoorPin = Pin<Input<PullUp>, 'A', 0>;


pub struct OvenDoor {
    /// Default closed    
    pin: DoorPin,
}

impl OvenDoor {

    pub fn new(pin: DoorPin) -> Self {
        OvenDoor {
            pin
        }
    }

    pub fn is_open(&self) -> bool {
        self.pin.is_low()
    }
}


#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [USART1])]
mod app {

    use super::OvenDoor;

    use stm32f4xx_hal::{
            gpio::{Edge}, 
            timer::MonoTimerUs,
            pac,
            prelude::*
        };
    use spisen::Led;

    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM2>;

    #[shared]
    struct Resources {
        door: OvenDoor,
    }

    #[local]
    struct Local {
        led: Led,
    }

    #[init]
    fn init(ctx: init::Context) -> (Resources, Local, init::Monotonics) {
        // Device specific peripherals
        let mut device = ctx.device;

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

        let mut syscfg = device.SYSCFG.constrain();

        let gpioa = device.GPIOA.split();


        let mut pin = gpioa.pa0.into_pull_up_input();
        pin.enable_interrupt(&mut device.EXTI);
        pin.make_interrupt_source(&mut syscfg);
        pin.trigger_on_edge(&mut device.EXTI, Edge::RisingFalling);

        let door = OvenDoor::new(pin);

        let mono = device.TIM2.monotonic_us(&clocks);

        // Setup the led
        let led = Led::new(gpioa.pa5);

        defmt::info!("init done: is_open: {}", door.is_open());

        (Resources { door }, Local { led }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        // The idle loop
        loop {}
    }

    #[task(
        binds = EXTI0, 
        shared = [door],
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
        door.lock(|b| b.pin.clear_interrupt_pending_bit());
    }

    #[task(
        capacity = 1,
        shared = [door],
        local = [led]
    )]
    fn check_door(mut ctx: check_door::Context) {
        let led = ctx.local.led;
        let open = ctx.shared.door.lock(|p| p.is_open());

        defmt::info!("Oven door open: {}", open);

        led.set(open)
    }
}
