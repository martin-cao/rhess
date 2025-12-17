#![no_std]
#![no_main]

use panic_halt as _;

use stm32f4xx_hal as hal;

mod board;
mod chess_core;
mod drivers;
mod game;
mod interaction;
mod start_menu;
mod start_menu_crab;
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
    let mode = start_menu::select_mode(&mut board);
    let (ai_sides, human_focus) = match mode {
        start_menu::Mode::HumanVsHuman => ([false, false], Some(chess_core::Color::White)),
        start_menu::Mode::HumanVsComputer => ([false, true], Some(chess_core::Color::White)),
        start_menu::Mode::ComputerVsHuman => ([true, false], Some(chess_core::Color::Black)),
        start_menu::Mode::ComputerVsComputer => ([true, true], None),
    };
    game::Game::run(&mut board, ai_sides, human_focus);
}
