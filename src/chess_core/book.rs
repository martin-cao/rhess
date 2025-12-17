use super::{GameState, Move, PieceKind};

/// 一条开局线，存放自起始局面的连续走法。
pub struct BookLine {
    pub moves: &'static [Move],
}

/// 通过模拟标准起始局面，匹配当前局面是否命中开局线前缀。
pub fn book_move(state: &GameState) -> Option<Move> {
    for line in BOOK_LINES.iter() {
        if let Some(mv) = match_prefix(state, line.moves) {
            return Some(mv);
        }
    }
    None
}

fn match_prefix(state: &GameState, line: &[Move]) -> Option<Move> {
    let mut sim = GameState::start_position();
    for (idx, mv) in line.iter().enumerate() {
        if sim == *state {
            return line.get(idx).copied();
        }
        if let Some(next) = sim.make_move(*mv) {
            sim = next;
        } else {
            break;
        }
    }
    None
}

const fn q(from: u8, to: u8) -> Move {
    Move {
        from,
        to,
        promotion: None,
        is_en_passant: false,
        is_castling: false,
    }
}

const fn castle(from: u8, to: u8) -> Move {
    Move {
        from,
        to,
        promotion: None,
        is_en_passant: false,
        is_castling: true,
    }
}

// 选取若干常见开局主线与分支，长度控制在 6–8 回合内。
const ITALIAN: &[Move] = &[
    q(12, 28), // 1. e4
    q(52, 36), // ... e5
    q(6, 21),  // 2. Nf3
    q(57, 42), // ... Nc6
    q(5, 26),  // 3. Bc4
    q(61, 34), // ... Bc5
    q(21, 38), // 4. Nf3->g5 (Ng5)
    q(34, 27), // ... Bxf2+ (forcing-ish)
];

const RUY_LOPEZ_MAIN: &[Move] = &[
    q(12, 28),    // 1. e4
    q(52, 36),    // ... e5
    q(6, 21),     // 2. Nf3
    q(57, 42),    // ... Nc6
    q(5, 33),     // 3. Bb5
    q(48, 40),    // ... a6
    q(33, 24),    // 4. Ba4
    q(62, 45),    // ... Nf6
    castle(4, 6), // 5. O-O
    q(61, 52),    // ... Be7
];

const QUEENS_GAMBIT: &[Move] = &[
    q(11, 27), // 1. d4
    q(51, 35), // ... d5
    q(10, 26), // 2. c4
    q(52, 44), // ... e6
    q(1, 18),  // 3. Nc3
    q(62, 45), // ... Nf6
    q(2, 38),  // 4. Bg5
];

const SICILIAN_NAJDORFISH: &[Move] = &[
    q(12, 28), // 1. e4
    q(50, 34), // ... c5
    q(6, 21),  // 2. Nf3
    q(51, 43), // ... d6
    q(11, 27), // 3. d4
    q(34, 27), // ... cxd4
    q(21, 36), // 4. Nxd4
    q(48, 40), // ... a6
    q(36, 53), // 5. Nb3
    q(57, 42), // ... Nc6
];

const CARO_KANN: &[Move] = &[
    q(12, 28), // 1. e4
    q(50, 34), // ... c6
    q(6, 21),  // 2. Nf3
    q(51, 35), // ... d5
    q(28, 35), // 3. exd5
    q(42, 35), // ... cxd5
    q(10, 26), // 4. c4
    q(57, 42), // ... Nc6
];

const BOOK_LINES: &[BookLine] = &[
    BookLine { moves: ITALIAN },
    BookLine {
        moves: RUY_LOPEZ_MAIN,
    },
    BookLine {
        moves: QUEENS_GAMBIT,
    },
    BookLine {
        moves: SICILIAN_NAJDORFISH,
    },
    BookLine { moves: CARO_KANN },
];
