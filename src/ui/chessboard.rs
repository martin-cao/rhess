use crate::drivers::lcd::Lcd;

// 棋盘与方格尺寸（屏幕左侧 272x272 区域，8x8 棋盘）
pub const BOARD_SIZE: u16 = 272;
pub const SQUARE_SIZE: u16 = BOARD_SIZE / 8;

// 16-bit RGB565 颜色
const LIGHT_SQUARE: u16 = 0xC618; // 浅灰
const DARK_SQUARE: u16 = 0x8410; // 深灰

pub fn draw_board(lcd: &mut Lcd) {
    for rank in 0..8 {
        for file in 0..8 {
            let file = file as u16;
            let rank = rank as u16;
            let is_light = (rank + file) % 2 == 0;
            let color = if is_light { LIGHT_SQUARE } else { DARK_SQUARE };
            let x = file * SQUARE_SIZE;
            let y = rank * SQUARE_SIZE;
            lcd.fill_rect(x, y, SQUARE_SIZE, SQUARE_SIZE, color);
        }
    }
}
