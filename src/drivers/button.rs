use crate::hal;
use embedded_hal::digital::InputPin;
use hal::gpio::{
    gpioa::PA0,
    gpioe::{PE2, PE3, PE4},
    Input,
};

/// 板载四个按键（上拉，按下为低电平）。
pub struct Buttons {
    key1: PE2<Input>,
    key2: PE3<Input>,
    key3: PE4<Input>,
    key4: PA0<Input>,
}

impl Buttons {
    pub fn new(key1: PE2<Input>, key2: PE3<Input>, key3: PE4<Input>, key4: PA0<Input>) -> Self {
        Self {
            key1,
            key2,
            key3,
            key4,
        }
    }

    #[inline]
    pub fn key1_pressed(&mut self) -> bool {
        is_low(&mut self.key1)
    }

    #[inline]
    pub fn key2_pressed(&mut self) -> bool {
        is_low(&mut self.key2)
    }

    #[inline]
    pub fn key3_pressed(&mut self) -> bool {
        is_low(&mut self.key3)
    }

    #[inline]
    pub fn key4_pressed(&mut self) -> bool {
        is_low(&mut self.key4)
    }
}

#[inline]
fn is_low<P: InputPin>(pin: &mut P) -> bool {
    pin.is_low().unwrap_or(false)
}
