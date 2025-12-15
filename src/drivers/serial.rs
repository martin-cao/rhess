use crate::hal;
use core::fmt;

use hal::pac;
use hal::prelude::*;
use hal::serial::{CommonPins, Rx, Serial, Tx, config::Config};
use hal::time::Bps;
use nb::block;

pub struct SerialPort {
    pub tx: Tx<pac::USART1>,
    pub rx: Rx<pac::USART1>,
}

impl SerialPort {
    pub fn new(
        usart1: pac::USART1,
        tx_pin: impl Into<<pac::USART1 as CommonPins>::Tx<hal::gpio::PushPull>>,
        rx_pin: impl Into<<pac::USART1 as CommonPins>::Rx<hal::gpio::PushPull>>,
        rcc: &mut hal::rcc::Rcc,
        baud: Bps,
    ) -> Self {
        let config = Config::default().baudrate(baud);
        let serial = Serial::new(usart1, (tx_pin, rx_pin), config, rcc).unwrap();
        let (tx, rx) = serial.split();
        Self { tx, rx }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            let _ = block!(self.tx.write(*b));
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}
