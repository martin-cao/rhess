#![no_std]
#![no_main]

use panic_halt as _;

use stm32f4xx_hal as hal;

mod board;
mod drivers;
mod chess_core;

use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut board = board::Board::new();
    board.leds.all_off();
    rprintln!("board init ok");
    board.lcd.clear(0x0000); // 初始清屏为黑

    loop {
        if board.buttons.key1_pressed() {
            board.leds.all_off();
            board.leds.led1.set_low();
            board.lcd.clear(0xF800); // 红
            rprintln!("KEY1 -> LED1 red screen");
        } else if board.buttons.key2_pressed() {
            board.leds.all_off();
            board.leds.led2.set_low();
            board.lcd.clear(0x07E0); // 绿
            rprintln!("KEY2 -> LED2 green screen");
        } else if board.buttons.key3_pressed() {
            board.leds.all_off();
            board.leds.led3.set_low();
            board.lcd.clear(0x001F); // 蓝
            rprintln!("KEY3 -> LED3 blue screen");
        } else if board.buttons.key4_pressed() {
            board.leds.all_off();
            board.leds.led4.set_low();
            board.lcd.clear(0xFFFF); // 白
            rprintln!("KEY4 -> LED4 blue screen");
        }

        // 简单轮询延时，减少抖动与输出刷屏。
        board.delay.ms(20);
    }
}
