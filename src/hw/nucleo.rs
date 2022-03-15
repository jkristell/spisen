use spisen::Led;
use stm32f4xx_hal::{
    gpio::{Alternate, Edge, Input, NoPin, PullUp, PushPull, PA0, PB13, PB15},
    pac,
    pac::SPI2,
    prelude::*,
    spi::{Spi, TransferModeNormal},
    timer::{DelayUs, MonoTimerUs},
};

// A0 on the nucleo
pub type DoorPin = PA0<Input<PullUp>>;
pub type HeaterSpi = Spi<
    SPI2,
    (
        PB13<Alternate<PushPull, 5>>,
        NoPin,
        PB15<Alternate<PushPull, 5>>,
    ),
    TransferModeNormal,
>;
pub type UsDelay = DelayUs<pac::TIM5>;
pub type MonotonicTim = pac::TIM2;

pub fn setup_hw(
    mut device: pac::Peripherals,
) -> (DoorPin, HeaterSpi, UsDelay, Led, MonoTimerUs<MonotonicTim>) {
    // Setup the system clock
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

    let mut syscfg = device.SYSCFG.constrain();
    let gpioa = device.GPIOA.split();
    let gpiob = device.GPIOB.split();

    let mut pin = gpioa.pa0.into_pull_up_input();
    pin.enable_interrupt(&mut device.EXTI);
    pin.make_interrupt_source(&mut syscfg);
    pin.trigger_on_edge(&mut device.EXTI, Edge::RisingFalling);

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

    let mono = device.TIM2.monotonic_us(&clocks);

    let delay = device.TIM5.delay_us(&clocks);
    // Setup the debug led
    let led = Led::new(gpioa.pa5);

    (pin, spi, delay, led, mono)
}
