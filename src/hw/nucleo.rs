use stm32f4xx_hal::{
    gpio::{Alternate, Edge, Input, NoPin, PullUp, PushPull, PA0, PA5, PA7},
    pac,
    pac::SPI1,
    prelude::*,
    spi::{Spi, TransferModeNormal},
    timer::{DelayUs, MonoTimerUs},
};

// A0 on the nucleo
pub type DoorPin = PA0<Input<PullUp>>;

//
pub type HeaterSpi = Spi<
    SPI1,
    (
        PA5<Alternate<PushPull, 5>>,
        NoPin,
        PA7<Alternate<PushPull, 5>>,
    ),
    TransferModeNormal,
>;
pub type UsDelay = DelayUs<pac::TIM5>;
pub type MonotonicTim = pac::TIM2;

pub fn setup_hw(
    mut device: pac::Peripherals,
) -> (DoorPin, HeaterSpi, UsDelay, MonoTimerUs<MonotonicTim>) {
    // Setup the system clock
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

    let mut syscfg = device.SYSCFG.constrain();
    let gpioa = device.GPIOA.split();

    let mut pin = gpioa.pa0.into_pull_up_input();
    pin.enable_interrupt(&mut device.EXTI);
    pin.make_interrupt_source(&mut syscfg);
    pin.trigger_on_edge(&mut device.EXTI, Edge::RisingFalling);

    // Configure pins for SPI
    let sclk = gpioa.pa5.into_alternate();
    let miso = NoPin; // miso not needed
    let mosi = gpioa.pa7.into_alternate().internal_pull_up(true);

    // SPI1 with 3Mhz
    let spi = Spi::new(
        device.SPI1,
        (sclk, miso, mosi),
        ws2812_spi::MODE,
        3_000_000.Hz(),
        &clocks,
    );

    let mono = device.TIM2.monotonic_us(&clocks);

    let delay = device.TIM5.delay_us(&clocks);

    (pin, spi, delay, mono)
}
