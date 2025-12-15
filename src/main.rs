#![no_std]
#![no_main]

use panic_halt as _;

use stm32f4xx_hal as hal;

// 常用屏幕颜色（RGB565）
const RED: u16 = 0xF800;
const GREEN: u16 = 0x07E0;
const BLUE: u16 = 0x001F;
const WHITE: u16 = 0xFFFF;
const ORANGE: u16 = 0xFD20;
const YELLOW: u16 = 0xFFE0;
const CYAN: u16 = 0x07FF;
const MAGENTA: u16 = 0xF81F;

mod board;
mod chess_core;
mod drivers;
mod ui;

use cortex_m_rt::entry;
use drivers::button::PressKind;
use ui::{chessboard, pieces};

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut board = board::Board::new();
    board.leds.all_off();
    rprintln!("board init ok");
    board.lcd.clear(0x0000); // 初始清屏为黑
    render_board(&mut board);

    loop {
        handle_buttons(&mut board);

        // 简单轮询延时，减少抖动与输出刷屏。
        board.delay.ms(20);
    }
}

fn render_board(board: &mut board::Board) {
    chessboard::draw_board(&mut board.lcd);
    render_start_position(board);
}

fn render_start_position(board: &mut board::Board) {
    let state = chess_core::GameState::start_position();
    for (idx, piece) in state.board.iter().enumerate() {
        if let Some(piece) = piece {
            let file = (idx % 8) as u8;
            let rank_from_bottom = (idx / 8) as u8; // 0 表示白方底线
            pieces::draw_piece_on_square(
                &mut board.lcd,
                piece.kind,
                piece.color,
                file,
                rank_from_bottom,
            );
        }
    }
}

fn handle_buttons(board: &mut board::Board) {
    if let Some(event) = poll_button(board) {
        match event {
            ButtonEvent::Key1(press) => {
                board.leds.all_off();
                board.leds.led1.set_low();
                match press {
                    PressKind::Short => {
                        board.lcd.clear(RED);
                        log_press("KEY1", press, "LED1 red screen");
                        render_board(board);
                    }
                    PressKind::Long => {
                        board.lcd.clear(ORANGE);
                        log_press("KEY1", press, "LED1 orange screen (long)");
                        render_board(board);
                    }
                }
            }
            ButtonEvent::Key2(press) => {
                board.leds.all_off();
                board.leds.led2.set_low();
                match press {
                    PressKind::Short => {
                        board.lcd.clear(GREEN);
                        log_press("KEY2", press, "LED2 green screen");
                        render_board(board);
                    }
                    PressKind::Long => {
                        board.lcd.clear(YELLOW);
                        log_press("KEY2", press, "LED2 yellow screen (long)");
                        render_board(board);
                    }
                }
            }
            ButtonEvent::Key3(press) => {
                board.leds.all_off();
                board.leds.led3.set_low();
                match press {
                    PressKind::Short => {
                        board.lcd.clear(BLUE);
                        log_press("KEY3", press, "LED3 blue screen");
                        render_board(board);
                    }
                    PressKind::Long => {
                        board.lcd.clear(CYAN);
                        log_press("KEY3", press, "LED3 cyan screen (long)");
                        render_board(board);
                    }
                }
            }
            ButtonEvent::Key4(press) => {
                board.leds.all_off();
                board.leds.led4.set_low();
                match press {
                    PressKind::Short => {
                        board.lcd.clear(WHITE); // 白
                        log_press("KEY4", press, "LED4 blue screen");
                        render_board(board);
                    }
                    PressKind::Long => {
                        board.lcd.clear(MAGENTA);
                        log_press("KEY4", press, "LED4 magenta screen (long)");
                        render_board(board);
                    }
                }
            }
        }
    }
}

fn poll_button(board: &mut board::Board) -> Option<ButtonEvent> {
    if let Some(press) = board.buttons.key1_press(&mut board.delay) {
        return Some(ButtonEvent::Key1(press));
    }
    if let Some(press) = board.buttons.key2_press(&mut board.delay) {
        return Some(ButtonEvent::Key2(press));
    }
    if let Some(press) = board.buttons.key3_press(&mut board.delay) {
        return Some(ButtonEvent::Key3(press));
    }
    if let Some(press) = board.buttons.key4_press(&mut board.delay) {
        return Some(ButtonEvent::Key4(press));
    }
    None
}

enum ButtonEvent {
    Key1(PressKind),
    Key2(PressKind),
    Key3(PressKind),
    Key4(PressKind),
}

fn log_press(key: &str, press: PressKind, action: &str) {
    match press {
        PressKind::Short => rprintln!("{} short -> {}", key, action),
        PressKind::Long => rprintln!("{} long  -> {}", key, action),
    }
}
