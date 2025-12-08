#![no_std]
#![no_main]

use panic_halt as _;

use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};
use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    loop {
        rprintln!("Hello world!");
    }
}
