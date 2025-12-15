use crate::drivers::delay::Delay;
use crate::hal;
use embedded_hal::digital::InputPin;
use hal::gpio::{
    Input,
    gpioa::PA0,
    gpioe::{PE2, PE3, PE4},
};

/// 长按识别阈值（毫秒）。
pub const LONG_PRESS_MS: u32 = 500;

// 轻量防抖与轮询步进，保持阻塞时间可控。
const DEBOUNCE_MS: u32 = 20;
const POLL_INTERVAL_MS: u32 = 10;

/// 板载四个按键（上拉，按下为低电平）。
pub struct Buttons {
    key1: PE2<Input>,
    key2: PE3<Input>,
    key3: PE4<Input>,
    key4: PA0<Input>,
    key1_long_latched: bool,
    key2_long_latched: bool,
    key3_long_latched: bool,
    key4_long_latched: bool,
}

impl Buttons {
    pub fn new(key1: PE2<Input>, key2: PE3<Input>, key3: PE4<Input>, key4: PA0<Input>) -> Self {
        Self {
            key1,
            key2,
            key3,
            key4,
            key1_long_latched: false,
            key2_long_latched: false,
            key3_long_latched: false,
            key4_long_latched: false,
        }
    }

    #[inline]
    pub fn key1_press(&mut self, delay: &mut Delay) -> Option<PressKind> {
        detect_press(&mut self.key1, &mut self.key1_long_latched, delay)
    }

    #[inline]
    pub fn key2_press(&mut self, delay: &mut Delay) -> Option<PressKind> {
        detect_press(&mut self.key2, &mut self.key2_long_latched, delay)
    }

    #[inline]
    pub fn key3_press(&mut self, delay: &mut Delay) -> Option<PressKind> {
        detect_press(&mut self.key3, &mut self.key3_long_latched, delay)
    }

    #[inline]
    pub fn key4_press(&mut self, delay: &mut Delay) -> Option<PressKind> {
        detect_press(&mut self.key4, &mut self.key4_long_latched, delay)
    }

    #[inline]
    pub fn key1_pressed(&mut self, delay: &mut Delay) -> bool {
        matches!(self.key1_press(delay), Some(PressKind::Short))
    }

    #[inline]
    pub fn key2_pressed(&mut self, delay: &mut Delay) -> bool {
        matches!(self.key2_press(delay), Some(PressKind::Short))
    }

    #[inline]
    pub fn key3_pressed(&mut self, delay: &mut Delay) -> bool {
        matches!(self.key3_press(delay), Some(PressKind::Short))
    }

    #[inline]
    pub fn key4_pressed(&mut self, delay: &mut Delay) -> bool {
        matches!(self.key4_press(delay), Some(PressKind::Short))
    }

    #[inline]
    pub fn key1_long_pressed(&mut self, delay: &mut Delay) -> bool {
        matches!(self.key1_press(delay), Some(PressKind::Long))
    }

    #[inline]
    pub fn key2_long_pressed(&mut self, delay: &mut Delay) -> bool {
        matches!(self.key2_press(delay), Some(PressKind::Long))
    }

    #[inline]
    pub fn key3_long_pressed(&mut self, delay: &mut Delay) -> bool {
        matches!(self.key3_press(delay), Some(PressKind::Long))
    }

    #[inline]
    pub fn key4_long_pressed(&mut self, delay: &mut Delay) -> bool {
        matches!(self.key4_press(delay), Some(PressKind::Long))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PressKind {
    Short,
    Long,
}

fn detect_press<P: InputPin>(
    pin: &mut P,
    long_latched: &mut bool,
    delay: &mut Delay,
) -> Option<PressKind> {
    // 若已报告长按且仍保持按下，不重复触发，等待松手后复位。
    if *long_latched && is_low(pin) {
        return None;
    }

    if !is_low(pin) {
        *long_latched = false;
        return None;
    }

    // 简单防抖。
    delay.ms(DEBOUNCE_MS);
    if !is_low(pin) {
        return None;
    }

    let mut held_ms = 0;
    while is_low(pin) {
        if held_ms >= LONG_PRESS_MS {
            *long_latched = true;
            return Some(PressKind::Long);
        }

        delay.ms(POLL_INTERVAL_MS);
        held_ms += POLL_INTERVAL_MS;
    }

    Some(PressKind::Short)
}

#[inline]
fn is_low<P: InputPin>(pin: &mut P) -> bool {
    pin.is_low().unwrap_or(false)
}
