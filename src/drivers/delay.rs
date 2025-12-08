use crate::hal;
use hal::pac;
use hal::prelude::*;
use hal::rcc::Clocks;
use hal::timer::{SysDelay, Timer};

/// SysTick 基础延时，毫秒/微秒。
pub struct Delay {
    inner: SysDelay,
}

impl Delay {
    pub fn new(syst: pac::SYST, clocks: &Clocks) -> Self {
        Self {
            inner: Timer::syst(syst, clocks).delay(),
        }
    }

    #[inline]
    pub fn ms(&mut self, ms: u32) {
        self.inner.delay(ms.millis());
    }

    #[inline]
    pub fn us(&mut self, us: u32) {
        self.inner.delay(us.micros());
    }
}
