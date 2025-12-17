use crate::board::Board;
use crate::drivers::button::PressKind;
use crate::start_menu_crab::{CRAB_BITMAP, CRAB_H, CRAB_W};
use crate::ui::chessboard;
use crate::ui::text;

const BG: u16 = 0x0000;
const FG: u16 = 0xFFFF;
const TITLE_COLOR: u16 = 0xFFE0;
const HIGHLIGHT: u16 = 0xE540; // 柔和橙

#[derive(Clone, Copy)]
pub enum Mode {
    HumanVsHuman,
    HumanVsComputer,
    ComputerVsHuman,
    ComputerVsComputer,
}

pub fn select_mode(board: &mut Board) -> Mode {
    let mut selected: usize = 0;
    let mut dirty = true;
    loop {
        if dirty {
            render_menu(board, selected);
            dirty = false;
        }
        if let Some(press) = board.buttons.key2_press(&mut board.delay) {
            if matches!(press, PressKind::Short) {
                let next = (selected + 1).min(3);
                if next != selected {
                    selected = next;
                    dirty = true;
                }
            }
        }
        if let Some(press) = board.buttons.key3_press(&mut board.delay) {
            if matches!(press, PressKind::Short) {
                let next = selected.saturating_sub(1);
                if next != selected {
                    selected = next;
                    dirty = true;
                }
            }
        }
        if let Some(press) = board.buttons.key1_press(&mut board.delay) {
            if matches!(press, PressKind::Short) {
                return match selected {
                    0 => Mode::HumanVsHuman,
                    1 => Mode::HumanVsComputer,
                    2 => Mode::ComputerVsHuman,
                    _ => Mode::ComputerVsComputer,
                };
            }
        }
        board.delay.ms(50);
    }
}

fn render_menu(board: &mut Board, selected: usize) {
    board.lcd.clear(BG);
    let left_width = compute_left_pane_width(board);
    let start_x = left_width.saturating_add(10);
    draw_title_and_crab(board, left_width);
    draw_options(board, start_x, selected);
}

fn compute_left_pane_width(board: &Board) -> u16 {
    let total = board.lcd.width;
    let mut left = ((total as u32 * 48) / 100) as u16; // 稍微让出空间给右侧文字
    let min_left = CRAB_W.saturating_add(16);
    if left < min_left {
        left = min_left;
    }
    left
}

fn draw_title_and_crab(board: &mut Board, left_width: u16) {
    let x = 8;
    let y = 6;
    text::draw_text_scaled(&mut board.lcd, "rhess", x, y, TITLE_COLOR, Some(BG), 3);

    let crab_x = (left_width.saturating_sub(CRAB_W)) / 2;
    let crab_y = (chessboard::BOARD_SIZE.saturating_sub(CRAB_H)) / 2;
    board
        .lcd
        .blit_bitmap(crab_x, crab_y, CRAB_W, CRAB_H, &CRAB_BITMAP);
}

fn draw_options(board: &mut Board, start_x: u16, selected: usize) {
    let start_y = 50u16;
    text::draw_text_scaled(
        &mut board.lcd,
        "Mode",
        start_x,
        start_y - 24,
        FG,
        Some(BG),
        2,
    );
    let entries = [
        "Human vs Human",
        "Human vs Computer",
        "Computer vs Human",
        "Computer vs Computer",
    ];
    for (i, label) in entries.iter().enumerate() {
        let y = start_y + i as u16 * 36;
        let arrow = if i == selected { ">" } else { " " };
        text::draw_text_scaled(&mut board.lcd, arrow, start_x, y, HIGHLIGHT, Some(BG), 2);
        text::draw_text_scaled(&mut board.lcd, label, start_x + 12, y, FG, Some(BG), 2);
    }
    text::draw_text_scaled(
        &mut board.lcd,
        "KEY3 Up  KEY2 Down",
        start_x,
        start_y + 160,
        FG,
        Some(BG),
        1,
    );
    text::draw_text_scaled(
        &mut board.lcd,
        "KEY1 Start",
        start_x,
        start_y + 176,
        FG,
        Some(BG),
        1,
    );
}
