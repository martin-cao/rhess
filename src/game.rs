use crate::board::Board;
use crate::chess_core::ai::{AiConfig, choose_best_move};
use crate::chess_core::{Color, GameState, Move, PieceKind};
use crate::interaction::{Action, PromotionChoice, poll_action, poll_promotion_choice};
use crate::ui::{chessboard, pieces, text};
use rtt_target::rprintln;

const SELECTED_PIECE_COLOR: u16 = 0xF800; // 红色
const UI_BG: u16 = 0x0000; // 右侧背景
const UI_FG: u16 = 0xFFFF; // 文本颜色
const UI_ALERT: u16 = 0xF800; // 亮红色提示
const RIGHT_X: u16 = chessboard::BOARD_SIZE;
const RIGHT_MARGIN: u16 = 4;
const AI_COLOR: Color = Color::Black;
const PLAYER_COLOR: Color = match AI_COLOR {
    Color::White => Color::Black,
    Color::Black => Color::White,
};

pub struct Game {
    state: GameState,
    cursor: (u8, u8),     // (file, rank_from_bottom)
    selected: Option<u8>, // 0..63
    promotion: Option<PromotionPrompt>,
}

#[derive(Clone, Copy)]
struct PromotionPrompt {
    from: u8,
    to: u8,
    color: Color,
    moves: [Option<Move>; 4], // 按顺序对应 车/马/象/后
}

impl Game {
    pub fn run(board: &mut Board) -> ! {
        let mut game = Game {
            state: GameState::start_position(),
            cursor: (0, 0),
            selected: None,
            promotion: None,
        };
        game.render(board);

        loop {
            game.step(board);
            board.delay.ms(20);
        }
    }

    fn step(&mut self, board: &mut Board) {
        if self.handle_promotion(board) {
            return;
        }
        if self.state.side_to_move == AI_COLOR {
            self.run_ai(board);
            return;
        }

        if let Some(action) = poll_action(board) {
            match action {
                Action::MoveLeft => self.cursor.0 = self.cursor.0.saturating_sub(1),
                Action::MoveRight => self.cursor.0 = (self.cursor.0 + 1).min(7),
                Action::MoveUp => self.cursor.1 = (self.cursor.1 + 1).min(7),
                Action::MoveDown => self.cursor.1 = self.cursor.1.saturating_sub(1),
                Action::ToggleSelect => self.toggle_select(),
                Action::SubmitMove => self.try_submit_move(board),
            }
            self.render(board);
        }
    }

    fn toggle_select(&mut self) {
        let idx = Self::index(self.cursor.0, self.cursor.1);
        if self.selected == Some(idx) {
            self.selected = None;
            return;
        }
        if self.state.board[idx as usize].is_some() {
            self.selected = Some(idx);
        }
    }

    fn try_submit_move(&mut self, board: &mut Board) {
        let Some(src) = self.selected else {
            return;
        };
        let dst = Self::index(self.cursor.0, self.cursor.1);
        if src == dst {
            return;
        }
        let move_set = self.find_moves(src, dst);
        if move_set.is_none() {
            rprintln!("非法走子: {} -> {}", src, dst);
            return;
        }
        let (normal, promo_moves) = move_set.unwrap();

        // 若存在升变选项且当前为玩家回合，进入升变选择
        if promo_moves.iter().any(|m| m.is_some()) && self.state.side_to_move != AI_COLOR {
            self.promotion = Some(PromotionPrompt {
                from: src,
                to: dst,
                color: self.state.side_to_move,
                moves: promo_moves,
            });
            self.selected = None;
            self.render(board);
            return;
        }

        if let Some(mv) = normal.or_else(|| promo_moves.iter().flatten().next().copied()) {
            if let Some(next) = self.state.make_move(mv) {
                self.state = next;
                self.selected = None;
                self.render(board); // 先显示玩家落子
                // 白方落子后交给 AI（黑方）
                if self.state.side_to_move == AI_COLOR {
                    self.run_ai(board);
                }
            }
        }
    }

    fn render(&self, board: &mut Board) {
        for rank in 0..8 {
            for file in 0..8 {
                self.render_square(board, file, rank);
            }
        }
        self.render_side_info(board);
    }

    fn render_square(&self, board: &mut Board, file: u8, rank: u8) {
        let idx = Self::index(file, rank);
        let is_promo_target = self.promotion.map_or(false, |p| p.to == idx);
        let is_promo_from = self.promotion.map_or(false, |p| p.from == idx);
        let is_cursor = self.cursor == (file, rank);
        let square_color = if is_promo_target {
            chessboard::PROMOTION_COLOR
        } else if is_cursor {
            chessboard::HIGHLIGHT_COLOR
        } else {
            chessboard::square_color(file, rank)
        };
        chessboard::draw_square_with_color(&mut board.lcd, file, rank, square_color);

        if is_promo_from {
            // 避免在原位重复显示
            return;
        }

        if let Some(prompt) = self.promotion {
            if prompt.to == idx {
                let piece_color = prompt.color;
                pieces::draw_piece_on_square_custom(
                    &mut board.lcd,
                    PieceKind::Pawn,
                    piece_color,
                    file,
                    rank,
                    None,
                );
                return;
            }
        }

        if let Some(piece) = self.state.board[idx as usize] {
            let override_color = if self.selected == Some(idx) {
                Some(SELECTED_PIECE_COLOR)
            } else {
                None
            };
            pieces::draw_piece_on_square_custom(
                &mut board.lcd,
                piece.kind,
                piece.color,
                file,
                rank,
                override_color,
            );
        }
    }

    fn render_side_info(&self, board: &mut Board) {
        let start_x = RIGHT_X + RIGHT_MARGIN;
        let width = board.lcd.width.saturating_sub(start_x);
        // 右侧信息区域
        board
            .lcd
            .fill_rect(start_x, 0, width, board.lcd.height, UI_BG);

        let side = match self.state.side_to_move {
            Color::White => "White",
            Color::Black => "Black",
        };
        let text_x = start_x + 2;
        let text_y = 6;
        text::draw_text_scaled(
            &mut board.lcd,
            "Side:",
            text_x,
            text_y,
            UI_FG,
            Some(UI_BG),
            2,
        );
        text::draw_text_scaled(
            &mut board.lcd,
            side,
            text_x + 64,
            text_y,
            UI_FG,
            Some(UI_BG),
            2,
        );

        let diff = self.material_diff(PLAYER_COLOR);
        let mut buf = [0u8; 12];
        let diff_str = i32_to_str(diff, &mut buf);

        text::draw_text_scaled(
            &mut board.lcd,
            "Mat:",
            text_x,
            text_y + 20,
            UI_FG,
            Some(UI_BG),
            2,
        );
        text::draw_text_scaled(
            &mut board.lcd,
            diff_str,
            text_x + 64,
            text_y + 20,
            UI_FG,
            Some(UI_BG),
            2,
        );

        if self.is_player_checkmated() {
            text::draw_text_scaled(
                &mut board.lcd,
                "Being checkmated",
                text_x,
                text_y + 70,
                UI_ALERT,
                Some(UI_BG),
                2,
            );
        }

        if let Some(prompt) = self.promotion {
            self.render_promotion_menu(board, start_x, prompt);
        }
    }

    fn render_promotion_menu(&self, board: &mut Board, start_x: u16, prompt: PromotionPrompt) {
        let x = start_x + 2;
        let mut y = 80;
        text::draw_text_scaled(
            &mut board.lcd,
            "Promote (KEY1-4)",
            x,
            y,
            UI_FG,
            Some(UI_BG),
            2,
        );
        y += 24;
        let entries = [
            ("1", "Rook", PieceKind::Rook),
            ("2", "Knight", PieceKind::Knight),
            ("3", "Bishop", PieceKind::Bishop),
            ("4", "Queen", PieceKind::Queen),
        ];
        for (_idx, (num, label, kind)) in entries.iter().copied().enumerate() {
            text::draw_text_scaled(&mut board.lcd, num, x, y, UI_FG, Some(UI_BG), 2);
            text::draw_text_scaled(&mut board.lcd, label, x + 20, y, UI_FG, Some(UI_BG), 2);
            // Place icon slightly above text baseline for better alignment.
            pieces::draw_piece_icon(&mut board.lcd, kind, prompt.color, x + 90, y - 2, None);
            y += 28;
        }
    }

    fn handle_promotion(&mut self, board: &mut Board) -> bool {
        let Some(prompt) = self.promotion else {
            return false;
        };

        // 确保高亮/菜单可见
        self.render(board);

        if let Some(choice) = poll_promotion_choice(board) {
            let idx = match choice {
                PromotionChoice::Rook => 0,
                PromotionChoice::Knight => 1,
                PromotionChoice::Bishop => 2,
                PromotionChoice::Queen => 3,
            };
            if let Some(mv) = prompt.moves.get(idx).and_then(|m| *m) {
                if let Some(next) = self.state.make_move(mv) {
                    self.state = next;
                }
            }
            self.promotion = None;
            self.selected = None;
            self.render(board);
        }
        true
    }

    fn find_moves(&self, src: u8, dst: u8) -> Option<(Option<Move>, [Option<Move>; 4])> {
        let mut normal = None;
        let mut promos: [Option<Move>; 4] = [None, None, None, None];
        let mut found = false;
        let moves = self.state.generate_legal_moves();
        for mv in moves.iter().copied() {
            if mv.from == src && mv.to == dst {
                found = true;
                if let Some(kind) = mv.promotion {
                    let slot = match kind {
                        PieceKind::Rook => Some(0),
                        PieceKind::Knight => Some(1),
                        PieceKind::Bishop => Some(2),
                        PieceKind::Queen => Some(3),
                        PieceKind::King | PieceKind::Pawn => None,
                    };
                    if let Some(i) = slot {
                        promos[i] = Some(mv);
                    }
                } else {
                    normal = Some(mv);
                }
            }
        }
        if found { Some((normal, promos)) } else { None }
    }

    fn index(file: u8, rank_from_bottom: u8) -> u8 {
        rank_from_bottom * 8 + file
    }

    fn run_ai(&mut self, board: &mut Board) {
        if self.state.side_to_move != AI_COLOR {
            return;
        }
        let cfg = AiConfig {
            max_depth: 2,
            node_limit: Some(8_000),
        };
        if let Some(mv) = choose_best_move(&self.state, AI_COLOR, cfg) {
            if let Some(next) = self.state.make_move(mv) {
                self.state = next;
            }
        }
        self.render(board);
    }

    fn material_scores(&self) -> (u32, u32) {
        let mut white = 0u32;
        let mut black = 0u32;
        for piece in self.state.board.iter().flatten() {
            let v = material_value(piece.kind);
            match piece.color {
                Color::White => white += v,
                Color::Black => black += v,
            }
        }
        (white, black)
    }

    fn material_diff(&self, player: Color) -> i32 {
        let (white, black) = self.material_scores();
        match player {
            Color::White => white as i32 - black as i32,
            Color::Black => black as i32 - white as i32,
        }
    }

    fn is_player_checkmated(&self) -> bool {
        if self.state.side_to_move != PLAYER_COLOR {
            return false;
        }
        let moves = self.state.generate_legal_moves();
        moves.len == 0 && self.state.is_in_check(PLAYER_COLOR)
    }
}

fn material_value(kind: PieceKind) -> u32 {
    match kind {
        PieceKind::Pawn => 1,
        PieceKind::Knight => 2,
        PieceKind::Bishop => 3,
        PieceKind::Rook => 5,
        PieceKind::Queen => 9,
        PieceKind::King => 0,
    }
}

fn u32_to_str<'a>(mut value: u32, buf: &'a mut [u8; 10]) -> &'a str {
    let mut i = buf.len();
    if value == 0 {
        buf[i - 1] = b'0';
        return core::str::from_utf8(&buf[i - 1..i]).unwrap();
    }
    while value > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    core::str::from_utf8(&buf[i..]).unwrap()
}

fn i32_to_str<'a>(value: i32, buf: &'a mut [u8; 12]) -> &'a str {
    let mut i = buf.len();
    let mut val = if value < 0 {
        (value as i64).abs() as u32
    } else {
        value as u32
    };

    if val == 0 {
        buf[i - 1] = b'0';
        return core::str::from_utf8(&buf[i - 1..i]).unwrap();
    }

    while val > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (val % 10) as u8;
        val /= 10;
    }

    if value < 0 && i > 0 {
        i -= 1;
        buf[i] = b'-';
    }

    core::str::from_utf8(&buf[i..]).unwrap()
}
