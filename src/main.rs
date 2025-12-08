#![no_std]
#![no_main]

use panic_halt as _;

use stm32f4xx_hal as hal;

mod board;
mod drivers;

use cortex_m_rt::entry;
use core::fmt::Write;

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut board = board::Board::new();
    let _ = writeln!(&mut board.serial, "dino board init");
    board.leds.all_off();
    rprintln!("board init ok");
    board.lcd.clear(0x0000); // 初始清屏为黑

    loop {
        let _ = board.leds.led1.toggle();
        board.delay.ms(500);

        if board.buttons.key1_pressed() {
            rprintln!("KEY1 pressed");
        }
        if board.buttons.key4_pressed() {
            rprintln!("KEY4 pressed");
        }
    }
}
