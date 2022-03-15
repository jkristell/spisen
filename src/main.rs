#![no_main]
#![no_std]

use defmt_rtt as _;
use hal::gpio::{Input, PullUp, PA0};
use panic_probe as _;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::{
    gpio::{Alternate, NoPin, PushPull, PB13, PB15},
    pac::SPI2,
    spi::{Spi, TransferModeNormal},
    timer::DelayUs,
    pac,
};

// A0 on the nucleo
type DoorPin = PA0<Input<PullUp>>;
type HeaterSpi = Spi<
    SPI2,
    (
        PB13<Alternate<PushPull, 5>>,
        NoPin,
        PB15<Alternate<PushPull, 5>>,
    ),
    TransferModeNormal,
>;
type UsDelay = DelayUs<pac::TIM5>;


#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [USART1])]
mod app {

    use spisen::{Heater, Led, OvenDoor};
    use stm32f4xx_hal::{
        gpio::{Edge, NoPin},
        pac,
        prelude::*,
        spi::Spi,
        timer::{MonoTimerUs},
    };

    use super::{HeaterSpi, UsDelay, DoorPin};

    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM2>;

    #[shared]
    struct Resources {
        door: OvenDoor<DoorPin>,
    }

    #[local]
    struct Local {
        led: Led,
        heater: Heater<HeaterSpi>,
        delay: UsDelay,
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
        let gpiob = device.GPIOB.split();

        let mono = device.TIM2.monotonic_us(&clocks);

        let mut pin = gpioa.pa0.into_pull_up_input();
        pin.enable_interrupt(&mut device.EXTI);
        pin.make_interrupt_source(&mut syscfg);
        pin.trigger_on_edge(&mut device.EXTI, Edge::RisingFalling);

        // Setup the Oven door
        let door = OvenDoor::new(pin);

        // Setup the Oven heater (Neopixel array)

        // Configure pins for SPI
        let mosi = gpiob.pb15.into_alternate().internal_pull_up(true);
        let sck = gpiob.pb13.into_alternate();

        let miso1 = NoPin; // miso not needed

        // SPI1 with 3Mhz
        let spi = Spi::new(
            device.SPI2,
            (sck, miso1, mosi),
            ws2812_spi::MODE,
            3_000_000.Hz(),
            &clocks,
        );

        let heater = Heater::new(spi);

        let delay = device.TIM5.delay_us(&clocks);

        // Setup the debug led
        let led = Led::new(gpioa.pa5);

        defmt::info!("init done: is_open: {}", door.is_open());

        (
            Resources { door },
            Local { led, heater, delay },
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
        local = [led]
    )]
    fn check_door(mut ctx: check_door::Context) {
        let led = ctx.local.led;
        let open = ctx.shared.door.lock(|p| p.is_open());

        defmt::info!("Oven door open: {}", open);

        led.set(open)
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
