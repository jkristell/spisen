use embedded_hal::{spi::FullDuplex};
use smart_leds::{
    gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB8,
};
use ws2812_spi::Ws2812;

/// Heater element
pub struct Heater<Spi> {
    ws: Ws2812<Spi>,
    run: bool,
    step: usize,
}

impl<Spi> Heater<Spi>
where
    Spi: FullDuplex<u8>,
    <Spi as FullDuplex<u8>>::Error: core::fmt::Debug,
{
    pub fn new(spi: Spi) -> Self {
        Heater {
            ws: Ws2812::new(spi),
            run: false,
            step: 0,
        }
    }

    pub fn enable(&mut self, enable: bool) {
        self.run = enable;
    }

    pub fn rainbow(&mut self) -> bool {
        const LED_NUM: usize = 8;
        let mut data = [RGB8::default(); LED_NUM];

        if !self.run {

            self.ws.write(gamma(data.iter().cloned())).unwrap();

            return true;
        }

        let j = self.step;

        self.step += 1;

        if self.step == 1024 {
            self.step = 0;
            self.run = false;
        }


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
        self.ws.write(gamma(data.iter().cloned())).unwrap();

        false
    }

}
