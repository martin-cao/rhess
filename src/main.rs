#![no_std]
#![no_main]

use panic_halt as _;

use stm32f4xx_hal as hal;

mod board;
mod chess_core;
mod drivers;
mod game;
mod interaction;
mod ui;

use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut board = board::Board::new();
    board.leds.all_off();
    rprintln!("board init ok");
    board.lcd.clear(0x0000); // 初始清屏为黑
    game::Game::run(&mut board);
}
