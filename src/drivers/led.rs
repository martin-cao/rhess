use crate::hal;
use hal::gpio::{gpiob::PB0, gpiob::PB1, gpioc::PC0, gpiof::PF10, Output, PushPull};

/// 板载四个 LED（低电平点亮）。
pub struct Leds {
    pub led1: PC0<Output<PushPull>>,
    pub led2: PF10<Output<PushPull>>,
    pub led3: PB0<Output<PushPull>>,
    pub led4: PB1<Output<PushPull>>,
}

impl Leds {
    pub fn new(
        led1: PC0<Output<PushPull>>,
        led2: PF10<Output<PushPull>>,
        led3: PB0<Output<PushPull>>,
        led4: PB1<Output<PushPull>>,
    ) -> Self {
        Self {
            led1,
            led2,
            led3,
            led4,
        }
    }

    pub fn all_off(&mut self) {
        self.led1.set_high();
        self.led2.set_high();
        self.led3.set_high();
        self.led4.set_high();
    }

    pub fn all_toggle(&mut self) {
        self.led1.toggle();
        self.led2.toggle();
        self.led3.toggle();
        self.led4.toggle();
    }
}
