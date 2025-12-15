use crate::drivers::lcd::Lcd;
use crate::ui::font::{FONT_HEIGHT, FONT_SPACING, FONT_WIDTH, glyph};

/// 绘制单个 ASCII 字符；若提供 bg，将覆盖背景。
pub fn draw_char(lcd: &mut Lcd, ch: char, x: u16, y: u16, color: u16, bg: Option<u16>) {
    let Some(g) = glyph(ch) else {
        return;
    };

    for (col_idx, col_bits) in g.iter().enumerate() {
        for row in 0..FONT_HEIGHT {
            let mask = 1 << row;
            let px = x + col_idx as u16;
            let py = y + row as u16;
            if col_bits & mask != 0 {
                lcd.draw_pixel(px, py, color);
            } else if let Some(bg) = bg {
                lcd.draw_pixel(px, py, bg);
            }
        }
    }

    // 间隔列（作为背景填充）
    if let Some(bg) = bg {
        let px = x + FONT_WIDTH as u16;
        for row in 0..FONT_HEIGHT {
            lcd.draw_pixel(px, y + row as u16, bg);
        }
    }
}

/// 绘制字符串，支持换行 `\n`。
pub fn draw_text(lcd: &mut Lcd, text: &str, mut x: u16, mut y: u16, color: u16, bg: Option<u16>) {
    let step_x = FONT_WIDTH as u16 + FONT_SPACING as u16;
    for ch in text.chars() {
        if ch == '\n' {
            x = 0;
            y = y.saturating_add(FONT_HEIGHT as u16 + 1);
            continue;
        }
        draw_char(lcd, ch, x, y, color, bg);
        x = x.saturating_add(step_x);
    }
}

/// 绘制单个字符（整型缩放，scale>=1），scale=2 可得到约 10x14 粗体效果。
pub fn draw_char_scaled(
    lcd: &mut Lcd,
    ch: char,
    x: u16,
    y: u16,
    color: u16,
    bg: Option<u16>,
    scale: u8,
) {
    let Some(g) = glyph(ch) else {
        return;
    };
    let s = scale.max(1) as u16;
    for (col_idx, col_bits) in g.iter().enumerate() {
        for row in 0..FONT_HEIGHT {
            let mask = 1 << row;
            let base_x = x + col_idx as u16 * s;
            let base_y = y + row as u16 * s;
            let draw_fg = col_bits & mask != 0;
            for dx in 0..s {
                for dy in 0..s {
                    let px = base_x + dx;
                    let py = base_y + dy;
                    if draw_fg {
                        lcd.draw_pixel(px, py, color);
                    } else if let Some(bg) = bg {
                        lcd.draw_pixel(px, py, bg);
                    }
                }
            }
        }
    }
    // 间隔列
    if let Some(bg) = bg {
        let base_x = x + (FONT_WIDTH as u16) * s;
        for dx in 0..(FONT_SPACING as u16 * s) {
            for dy in 0..(FONT_HEIGHT as u16 * s) {
                lcd.draw_pixel(base_x + dx, y + dy, bg);
            }
        }
    }
}

/// 绘制字符串（可缩放），scale=2 推荐用于右侧 UI 提高清晰度。
pub fn draw_text_scaled(
    lcd: &mut Lcd,
    text: &str,
    mut x: u16,
    mut y: u16,
    color: u16,
    bg: Option<u16>,
    scale: u8,
) {
    let s = scale.max(1) as u16;
    let step_x = (FONT_WIDTH as u16 * s) + (FONT_SPACING as u16 * s);
    let step_y = FONT_HEIGHT as u16 * s + s; // 行距近似 1 像素*s
    for ch in text.chars() {
        if ch == '\n' {
            x = 0;
            y = y.saturating_add(step_y);
            continue;
        }
        draw_char_scaled(lcd, ch, x, y, color, bg, scale);
        x = x.saturating_add(step_x);
    }
}
