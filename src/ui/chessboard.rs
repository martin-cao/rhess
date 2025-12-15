use crate::drivers::lcd::Lcd;

// 棋盘与方格尺寸（屏幕左侧 272x272 区域，8x8 棋盘）
pub const BOARD_SIZE: u16 = 272;
pub const SQUARE_SIZE: u16 = BOARD_SIZE / 8;

// 16-bit RGB565 颜色
const LIGHT_SQUARE: u16 = 0xC618; // 浅灰
const DARK_SQUARE: u16 = 0x8410; // 深灰
pub const HIGHLIGHT_COLOR: u16 = 0xFFE0; // 亮黄
pub const PROMOTION_COLOR: u16 = 0x07E0; // 绿色用于升变提示

pub fn draw_board(lcd: &mut Lcd) {
    for rank in 0..8 {
        for file in 0..8 {
            draw_square(lcd, file, rank);
        }
    }
}

pub fn draw_square(lcd: &mut Lcd, file: u8, rank_from_bottom: u8) {
    draw_square_with_color(
        lcd,
        file,
        rank_from_bottom,
        square_color(file, rank_from_bottom),
    );
}

pub fn draw_square_with_color(lcd: &mut Lcd, file: u8, rank_from_bottom: u8, color: u16) {
    if file >= 8 || rank_from_bottom >= 8 {
        return;
    }
    let x = file as u16 * SQUARE_SIZE;
    // rank_from_bottom=0 在屏幕底部
    let y = (7 - rank_from_bottom as u16) * SQUARE_SIZE;
    lcd.fill_rect(x, y, SQUARE_SIZE, SQUARE_SIZE, color);
}

pub fn square_color(file: u8, rank_from_bottom: u8) -> u16 {
    if (file + rank_from_bottom) % 2 == 0 {
        LIGHT_SQUARE
    } else {
        DARK_SQUARE
    }
}
