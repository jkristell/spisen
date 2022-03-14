use embedded_hal::{blocking::delay::DelayMs, spi::FullDuplex};
use smart_leds::{
    gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB8,
};
use ws2812_spi::Ws2812;

/// Heater element
pub struct Heater<Spi> {
    ws: Ws2812<Spi>,
}

impl<Spi> Heater<Spi>
where
    Spi: FullDuplex<u8>,
    <Spi as FullDuplex<u8>>::Error: core::fmt::Debug,
{
    pub fn new(spi: Spi) -> Self {
        Heater {
            ws: Ws2812::new(spi),
        }
    }

    pub fn rainbow(&mut self, delay: &mut impl DelayMs<u8>) {
        const LED_NUM: usize = 8;
        let mut data = [RGB8::default(); LED_NUM];

        loop {
            for j in 0..256 {
                for i in 0..LED_NUM {
                    // rainbow cycle using HSV, where hue goes through all colors in circle
                    // value sets the brightness
                    let hsv = Hsv {
                        hue: ((i * 3 + j) % 256) as u8,
                        sat: 255,
                        val: 100,
                    };

                    data[i] = hsv2rgb(hsv);
                }
                // before writing, apply gamma correction for nicer rainbow
                self.ws.write(gamma(data.iter().cloned())).unwrap();
                delay.delay_ms(10u8);
            }
        }
    }
}
