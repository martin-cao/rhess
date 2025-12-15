use crate::chess_core::{Color, PieceKind};
use crate::drivers::lcd::Lcd;
use crate::ui::chessboard::SQUARE_SIZE;

// 统一的棋子位图尺寸（16x20 单色，居中绘制）
const SPRITE_W: usize = 16;
const SPRITE_H: usize = 20;

// RGB565: 白棋纯白，黑棋纯黑
const WHITE_FG: u16 = 0xFFFF;
const BLACK_FG: u16 = 0x0000;

struct Sprite {
    rows: &'static [u16; SPRITE_H],
}

// 位图：顶部到尾部逐行，高位在左。
const PAWN: Sprite = Sprite {
    rows: &[
        0x07C0, 0x0FE0, 0x0FE0, 0x0FE0, 0x07C0, 0x07C0, 0x0FE0, 0x1FF0, 0x3FF8, 0x7FFC, 0x7FFC,
        0x7FFC, 0x3FF8, 0x1FF0, 0x0FE0, 0x1FF0, 0x3FF8, 0x3FF8, 0x7FFC, 0xFFFF,
    ],
};

const ROOK: Sprite = Sprite {
    rows: &[
        0xF0F0, 0xF0F0, 0xFFFF, 0x7FFE, 0x3FFC, 0x3FFC, 0x3FFC, 0x3FFC, 0x3FFC, 0x3FFC, 0x3FFC,
        0x3FFC, 0x3FFC, 0x3FFC, 0x3FFC, 0x3FFC, 0x7FFE, 0x7FFE, 0xFFFF, 0xFFFF,
    ],
};

const BISHOP: Sprite = Sprite {
    rows: &[
        0x07C0, 0x0FE0, 0x1FF0, 0x3FF8, 0x7FFC, 0x7EFC, 0x7C7C, 0x3CF8, 0x1FF0, 0x0FE0, 0x1FF0,
        0x3FF8, 0x7FFC, 0x7FFC, 0x7FFC, 0x3FF8, 0x1FF0, 0x0FE0, 0x0FE0, 0x1FF0,
    ],
};

const KNIGHT: Sprite = Sprite {
    rows: &[
        0x07F0, 0x0FF8, 0x1FFC, 0x3FFC, 0x7FF8, 0xFFE0, 0xFFC0, 0xFF00, 0xFE00, 0xFC00, 0xFC00,
        0xFE00, 0xFF00, 0xFF80, 0x7FC0, 0x3FF0, 0x1FF8, 0x0FFC, 0x07FE, 0x03FF,
    ],
};

const QUEEN: Sprite = Sprite {
    rows: &[
        0x8001, 0x4002, 0x2004, 0x0FF8, 0x1FFC, 0x3FFE, 0x3FFE, 0x3FFE, 0x1FFC, 0x0FF8, 0x0FF8,
        0x0FF8, 0x1FFC, 0x3FFE, 0x3FFE, 0x3FFE, 0x1FFC, 0x1FFC, 0x3FFE, 0xFFFF,
    ],
};

const KING: Sprite = Sprite {
    rows: &[
        0x0180, 0x03C0, 0x03C0, 0xFFFF, 0x03C0, 0x03C0, 0x07E0, 0x0FF0, 0x1FF8, 0x3FFC, 0x7FFE,
        0x7FFE, 0x7FFE, 0x7FFE, 0x7FFE, 0x3FFC, 0x3FFC, 0x3FFC, 0x7FFE, 0xFFFF,
    ],
};

pub fn draw_piece_on_square(
    lcd: &mut Lcd,
    kind: PieceKind,
    color: Color,
    file: u8,
    rank_from_bottom: u8,
) {
    if file >= 8 || rank_from_bottom >= 8 {
        return;
    }
    let sprite = match kind {
        PieceKind::Pawn => &PAWN,
        PieceKind::Rook => &ROOK,
        PieceKind::Knight => &KNIGHT,
        PieceKind::Bishop => &BISHOP,
        PieceKind::Queen => &QUEEN,
        PieceKind::King => &KING,
    };
    let fg = match color {
        Color::White => WHITE_FG,
        Color::Black => BLACK_FG,
    };

    let base_x = file as u16 * SQUARE_SIZE;
    // rank_from_bottom=0 表示底部（白方后排），因此需要从屏幕顶部反转
    let base_y = (7 - rank_from_bottom as u16) * SQUARE_SIZE;

    let offset_x = base_x + (SQUARE_SIZE - SPRITE_W as u16) / 2;
    let offset_y = base_y + (SQUARE_SIZE - SPRITE_H as u16) / 2;

    draw_sprite(lcd, sprite, fg, offset_x, offset_y);
}

fn draw_sprite(lcd: &mut Lcd, sprite: &Sprite, fg: u16, start_x: u16, start_y: u16) {
    for (row_idx, bits) in sprite.rows.iter().enumerate() {
        let y = start_y + row_idx as u16;
        if y >= lcd.height {
            break;
        }
        for bit in 0..SPRITE_W {
            let mask = 1 << (SPRITE_W - 1 - bit);
            if bits & mask != 0 {
                let x = start_x + bit as u16;
                if x >= lcd.width {
                    break;
                }
                lcd.draw_pixel(x, y, fg);
            }
        }
    }
}
