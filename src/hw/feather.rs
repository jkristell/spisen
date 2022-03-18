use feather_f405::{
    pac,
    hal::{
        prelude::*,
    }
};
use feather_f405::pins::{A0, all_pins, MO, SCK};
use stm32f4xx_hal::gpio::{Alternate, Edge, Input, NoPin, PullUp, PushPull};
use stm32f4xx_hal::spi::{Spi, TransferModeNormal};
use stm32f4xx_hal::timer::MonoTimerUs;



pub type DoorPin = A0<Input<PullUp>>;

//
pub type HeaterSpi = Spi<
    pac::SPI2,
    (
        SCK<Alternate<PushPull, 5>>,
        NoPin,
        MO<Alternate<PushPull, 5>>,
    ),
    TransferModeNormal,
>;
pub type MonotonicTim = pac::TIM2;

pub fn setup_hw(
    mut device: pac::Peripherals,
) -> (DoorPin, HeaterSpi, MonoTimerUs<MonotonicTim>) {
    // Setup the system clock
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

    let mut syscfg = device.SYSCFG.constrain();

    let pins = all_pins(device.GPIOA, device.GPIOB, device.GPIOC, device.GPIOD);

    let mut pin = pins.a0.into_pull_up_input();
    pin.enable_interrupt(&mut device.EXTI);
    pin.make_interrupt_source(&mut syscfg);
    pin.trigger_on_edge(&mut device.EXTI, Edge::RisingFalling);

    // Configure pins for SPI
    let sclk = pins.sck.into_alternate();
    let miso = NoPin; // miso not needed
    let mosi = pins.mo.into_alternate().internal_pull_up(true);

    // SPI1 with 3Mhz
    let spi = Spi::new(
        device.SPI2,
        (sclk, miso, mosi),
        ws2812_spi::MODE,
        3_000_000.Hz(),
        &clocks,
    );

    let mono = device.TIM2.monotonic_us(&clocks);


    (pin, spi, mono)
}
