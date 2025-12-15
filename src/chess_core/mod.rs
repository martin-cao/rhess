// Core chess engine targeted at bare-metal: no heap, fixed-size arrays only.
// Coordinates use 0..63 (a1 = 0, h8 = 63) with rank = idx / 8, file = idx % 8.

// Re-export the actual `core` crate so macros (e.g., RTT) using `core::...` keep working.
pub use ::core::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    // Returns opponent color.
    fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    // +1 for White (up the board in array terms), -1 for Black.
    fn pawn_direction(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    // Starting rank for pawns of this color.
    fn home_rank(self) -> u8 {
        match self {
            Color::White => 1,
            Color::Black => 6,
        }
    }

    // Promotion rank for pawns of this color.
    fn promotion_rank(self) -> u8 {
        match self {
            Color::White => 7,
            Color::Black => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promotion: Option<PieceKind>,
    pub is_en_passant: bool,
    pub is_castling: bool,
}

impl Move {
    // Helper for quiet/non-special moves.
    pub const fn quiet(from: u8, to: u8) -> Move {
        Move {
            from,
            to,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastlingRights {
    bits: u8,
}

impl CastlingRights {
    const WHITE_KING: u8 = 0b0001;
    const WHITE_QUEEN: u8 = 0b0010;
    const BLACK_KING: u8 = 0b0100;
    const BLACK_QUEEN: u8 = 0b1000;

    pub const fn new() -> CastlingRights {
        CastlingRights { bits: 0 }
    }

    pub const fn full() -> CastlingRights {
        CastlingRights {
            bits: Self::WHITE_KING | Self::WHITE_QUEEN | Self::BLACK_KING | Self::BLACK_QUEEN,
        }
    }

    fn remove_white(&mut self) {
        self.bits &= !(Self::WHITE_KING | Self::WHITE_QUEEN);
    }

    fn remove_black(&mut self) {
        self.bits &= !(Self::BLACK_KING | Self::BLACK_QUEEN);
    }

    fn remove_white_king_side(&mut self) {
        self.bits &= !Self::WHITE_KING;
    }

    fn remove_white_queen_side(&mut self) {
        self.bits &= !Self::WHITE_QUEEN;
    }

    fn remove_black_king_side(&mut self) {
        self.bits &= !Self::BLACK_KING;
    }

    fn remove_black_queen_side(&mut self) {
        self.bits &= !Self::BLACK_QUEEN;
    }

    fn can_castle(&self, color: Color, king_side: bool) -> bool {
        match (color, king_side) {
            (Color::White, true) => self.bits & Self::WHITE_KING != 0,
            (Color::White, false) => self.bits & Self::WHITE_QUEEN != 0,
            (Color::Black, true) => self.bits & Self::BLACK_KING != 0,
            (Color::Black, false) => self.bits & Self::BLACK_QUEEN != 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u16,
    pub fullmove_number: u16,
}

impl GameState {
    // Standard initial position.
    pub const fn start_position() -> GameState {
        use Color::*;
        use PieceKind::*;
        let mut board = [None; 64];
        // White pieces
        board[0] = Some(Piece {
            color: White,
            kind: Rook,
        });
        board[1] = Some(Piece {
            color: White,
            kind: Knight,
        });
        board[2] = Some(Piece {
            color: White,
            kind: Bishop,
        });
        board[3] = Some(Piece {
            color: White,
            kind: Queen,
        });
        board[4] = Some(Piece {
            color: White,
            kind: King,
        });
        board[5] = Some(Piece {
            color: White,
            kind: Bishop,
        });
        board[6] = Some(Piece {
            color: White,
            kind: Knight,
        });
        board[7] = Some(Piece {
            color: White,
            kind: Rook,
        });
        let mut i = 8;
        while i < 16 {
            board[i] = Some(Piece {
                color: White,
                kind: Pawn,
            });
            i += 1;
        }
        // Black pieces
        let mut j = 48;
        while j < 56 {
            board[j] = Some(Piece {
                color: Black,
                kind: Pawn,
            });
            j += 1;
        }
        board[56] = Some(Piece {
            color: Black,
            kind: Rook,
        });
        board[57] = Some(Piece {
            color: Black,
            kind: Knight,
        });
        board[58] = Some(Piece {
            color: Black,
            kind: Bishop,
        });
        board[59] = Some(Piece {
            color: Black,
            kind: Queen,
        });
        board[60] = Some(Piece {
            color: Black,
            kind: King,
        });
        board[61] = Some(Piece {
            color: Black,
            kind: Bishop,
        });
        board[62] = Some(Piece {
            color: Black,
            kind: Knight,
        });
        board[63] = Some(Piece {
            color: Black,
            kind: Rook,
        });

        GameState {
            board,
            side_to_move: White,
            castling: CastlingRights::full(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    // Generate fully legal moves by filtering pseudo-legal moves that leave king in check.
    pub fn generate_legal_moves(&self) -> MoveList {
        let mut list = MoveList::new();
        self.generate_pseudo_legal_moves(&mut list);
        list.retain(|mv| {
            let mut cloned = *self;
            cloned.apply_move_unchecked(*mv);
            !cloned.is_in_check(self.side_to_move)
        });
        list
    }

    // Lightweight legality test against generated list.
    pub fn is_move_legal(&self, mv: Move) -> bool {
        self.generate_legal_moves().iter().any(|m| *m == mv)
    }

    // Play a move if legal and return new state.
    pub fn make_move(&self, mv: Move) -> Option<GameState> {
        if !self.is_move_legal(mv) {
            return None;
        }
        let mut next = *self;
        next.apply_move_unchecked(mv);
        Some(next)
    }

    // Pseudo-legal generator (no self-check filtering).
    fn generate_pseudo_legal_moves(&self, list: &mut MoveList) {
        for idx in 0..64 {
            if let Some(piece) = self.board[idx] {
                if piece.color != self.side_to_move {
                    continue;
                }
                match piece.kind {
                    PieceKind::Pawn => self.gen_pawn_moves(idx as u8, piece.color, list),
                    PieceKind::Knight => self.gen_knight_moves(idx as u8, piece.color, list),
                    PieceKind::Bishop => {
                        self.gen_slider_moves(idx as u8, piece.color, list, &[9, 7, -9, -7])
                    }
                    PieceKind::Rook => {
                        self.gen_slider_moves(idx as u8, piece.color, list, &[8, -8, 1, -1])
                    }
                    PieceKind::Queen => self.gen_slider_moves(
                        idx as u8,
                        piece.color,
                        list,
                        &[8, -8, 1, -1, 9, 7, -9, -7],
                    ),
                    PieceKind::King => self.gen_king_moves(idx as u8, piece.color, list),
                }
            }
        }
    }

    fn gen_pawn_moves(&self, sq: u8, color: Color, list: &mut MoveList) {
        let dir = color.pawn_direction();
        let rank = rank_of(sq);
        let forward = (sq as i8 + 8 * dir) as i16;
        if forward >= 0 && forward < 64 {
            let fwd_sq = forward as u8;
            if self.board[fwd_sq as usize].is_none() {
                self.push_pawn_move(sq, fwd_sq, color, list);
                if rank == color.home_rank() {
                    let double_forward = (sq as i8 + 16 * dir) as i16;
                    if double_forward >= 0 && double_forward < 64 {
                        let df = double_forward as u8;
                        if self.board[df as usize].is_none() {
                            list.push(Move {
                                from: sq,
                                to: df,
                                promotion: None,
                                is_en_passant: false,
                                is_castling: false,
                            });
                        }
                    }
                }
            }
        }

        let capture_offsets = [7 * dir, 9 * dir];
        for off in capture_offsets.iter() {
            let target = sq as i16 + *off as i16;
            if target < 0 || target >= 64 {
                continue;
            }
            let to = target as u8;
            if file_distance(sq, to) != 1 {
                continue;
            }
            if let Some(piece) = self.board[to as usize] {
                if piece.color != color {
                    self.push_pawn_move(sq, to, color, list);
                }
            } else if let Some(ep) = self.en_passant {
                if ep == to {
                    list.push(Move {
                        from: sq,
                        to,
                        promotion: None,
                        is_en_passant: true,
                        is_castling: false,
                    });
                }
            }
        }
    }

    fn push_pawn_move(&self, from: u8, to: u8, color: Color, list: &mut MoveList) {
        if rank_of(to) == color.promotion_rank() {
            let promos = [
                PieceKind::Queen,
                PieceKind::Rook,
                PieceKind::Bishop,
                PieceKind::Knight,
            ];
            for kind in promos.iter() {
                list.push(Move {
                    from,
                    to,
                    promotion: Some(*kind),
                    is_en_passant: false,
                    is_castling: false,
                });
            }
        } else {
            list.push(Move {
                from,
                to,
                promotion: None,
                is_en_passant: false,
                is_castling: false,
            });
        }
    }

    fn gen_knight_moves(&self, sq: u8, color: Color, list: &mut MoveList) {
        const OFFSETS: [i8; 8] = [17, 15, 10, 6, -17, -15, -10, -6];
        for off in OFFSETS.iter() {
            let target = sq as i16 + *off as i16;
            if target < 0 || target >= 64 {
                continue;
            }
            let to = target as u8;
            if knight_move_wraps(sq, to) {
                continue;
            }
            if self.board[to as usize].map_or(true, |p| p.color != color) {
                list.push(Move::quiet(sq, to));
            }
        }
    }

    fn gen_slider_moves(&self, sq: u8, color: Color, list: &mut MoveList, dirs: &[i8]) {
        for dir in dirs.iter() {
            let mut cur = sq as i16;
            loop {
                cur += *dir as i16;
                if cur < 0 || cur >= 64 {
                    break;
                }
                let to = cur as u8;
                if wraps(sq, to, *dir) {
                    break;
                }
                match self.board[to as usize] {
                    None => list.push(Move::quiet(sq, to)),
                    Some(p) if p.color != color => {
                        list.push(Move::quiet(sq, to));
                        break;
                    }
                    _ => break,
                }
            }
        }
    }

    fn gen_king_moves(&self, sq: u8, color: Color, list: &mut MoveList) {
        let offsets = [1, -1, 8, -8, 9, 7, -7, -9];
        for off in offsets.iter() {
            let target = sq as i16 + *off as i16;
            if target < 0 || target >= 64 {
                continue;
            }
            let to = target as u8;
            if king_wraps(sq, to) {
                continue;
            }
            if self.board[to as usize].map_or(true, |p| p.color != color) {
                list.push(Move::quiet(sq, to));
            }
        }
        self.gen_castling(sq, color, list);
    }

    fn gen_castling(&self, sq: u8, color: Color, list: &mut MoveList) {
        if self.is_in_check(color) {
            return;
        }
        match color {
            Color::White => {
                if self.castling.can_castle(Color::White, true)
                    && self.board[5].is_none()
                    && self.board[6].is_none()
                    && !self.is_square_attacked(5, Color::Black)
                    && !self.is_square_attacked(6, Color::Black)
                {
                    list.push(Move {
                        from: sq,
                        to: 6,
                        promotion: None,
                        is_en_passant: false,
                        is_castling: true,
                    });
                }
                if self.castling.can_castle(Color::White, false)
                    && self.board[1].is_none()
                    && self.board[2].is_none()
                    && self.board[3].is_none()
                    && !self.is_square_attacked(2, Color::Black)
                    && !self.is_square_attacked(3, Color::Black)
                {
                    list.push(Move {
                        from: sq,
                        to: 2,
                        promotion: None,
                        is_en_passant: false,
                        is_castling: true,
                    });
                }
            }
            Color::Black => {
                if self.castling.can_castle(Color::Black, true)
                    && self.board[61].is_none()
                    && self.board[62].is_none()
                    && !self.is_square_attacked(61, Color::White)
                    && !self.is_square_attacked(62, Color::White)
                {
                    list.push(Move {
                        from: sq,
                        to: 62,
                        promotion: None,
                        is_en_passant: false,
                        is_castling: true,
                    });
                }
                if self.castling.can_castle(Color::Black, false)
                    && self.board[57].is_none()
                    && self.board[58].is_none()
                    && self.board[59].is_none()
                    && !self.is_square_attacked(58, Color::White)
                    && !self.is_square_attacked(59, Color::White)
                {
                    list.push(Move {
                        from: sq,
                        to: 58,
                        promotion: None,
                        is_en_passant: false,
                        is_castling: true,
                    });
                }
            }
        }
    }

    fn apply_move_unchecked(&mut self, mv: Move) {
        let moving_piece = self.board[mv.from as usize].unwrap();
        // Reset en-passant; may be set again for double pawn pushes.
        self.en_passant = None;
        self.halfmove_clock += 1;

        // Handle captures and special pawn captures.
        if mv.is_en_passant {
            let dir = if moving_piece.color == Color::White {
                -8
            } else {
                8
            };
            let captured_sq = (mv.to as i16 + dir) as u8;
            self.board[captured_sq as usize] = None;
        } else if self.board[mv.to as usize].is_some() {
            self.halfmove_clock = 0;
        }

        // Move piece.
        self.board[mv.to as usize] = self.board[mv.from as usize];
        self.board[mv.from as usize] = None;

        // Promotion.
        if let Some(promote) = mv.promotion {
            self.board[mv.to as usize] = Some(Piece {
                color: moving_piece.color,
                kind: promote,
            });
            self.halfmove_clock = 0;
        }

        // Castling rook move.
        if mv.is_castling {
            match (moving_piece.color, mv.to) {
                (Color::White, 6) => {
                    self.board[5] = self.board[7];
                    self.board[7] = None;
                }
                (Color::White, 2) => {
                    self.board[3] = self.board[0];
                    self.board[0] = None;
                }
                (Color::Black, 62) => {
                    self.board[61] = self.board[63];
                    self.board[63] = None;
                }
                (Color::Black, 58) => {
                    self.board[59] = self.board[56];
                    self.board[56] = None;
                }
                _ => {}
            }
        }

        // Double pawn push -> set en-passant target.
        if moving_piece.kind == PieceKind::Pawn {
            self.halfmove_clock = 0;
            let diff = mv.to as i16 - mv.from as i16;
            if diff == 16 || diff == -16 {
                let ep_sq = (mv.from as i16 + diff / 2) as u8;
                self.en_passant = Some(ep_sq);
            }
        }

        // Update castling rights based on moved or captured pieces.
        match moving_piece.kind {
            PieceKind::King => match moving_piece.color {
                Color::White => self.castling.remove_white(),
                Color::Black => self.castling.remove_black(),
            },
            PieceKind::Rook => match mv.from {
                0 => self.castling.remove_white_queen_side(),
                7 => self.castling.remove_white_king_side(),
                56 => self.castling.remove_black_queen_side(),
                63 => self.castling.remove_black_king_side(),
                _ => {}
            },
            _ => {}
        }

        // Capturing rooks also disables opponent castling on that side.
        match mv.to {
            0 => self.castling.remove_white_queen_side(),
            7 => self.castling.remove_white_king_side(),
            56 => self.castling.remove_black_queen_side(),
            63 => self.castling.remove_black_king_side(),
            _ => {}
        }

        if self.side_to_move == Color::Black {
            self.fullmove_number += 1;
        }
        self.side_to_move = self.side_to_move.opposite();
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let king_sq = self.board.iter().position(
            |p| matches!(p, Some(Piece { color: c, kind: PieceKind::King }) if *c == color),
        );
        match king_sq {
            Some(sq) => self.is_square_attacked(sq as u8, color.opposite()),
            None => false,
        }
    }

    // Attack detection for a given square and attacker color.
    fn is_square_attacked(&self, sq: u8, by: Color) -> bool {
        // Pawn attacks
        let dir = by.pawn_direction();
        for off in [7 * dir, 9 * dir].iter() {
            let target = sq as i16 + *off as i16;
            if target >= 0 && target < 64 {
                let from = target as u8;
                if file_distance(sq, from) == 1 {
                    if let Some(piece) = self.board[from as usize] {
                        if piece.color == by && piece.kind == PieceKind::Pawn {
                            return true;
                        }
                    }
                }
            }
        }
        // Knights
        const KNIGHT_OFFSETS: [i8; 8] = [17, 15, 10, 6, -17, -15, -10, -6];
        for off in KNIGHT_OFFSETS.iter() {
            let target = sq as i16 + *off as i16;
            if target >= 0 && target < 64 {
                let from = target as u8;
                if knight_move_wraps(from, sq) {
                    continue;
                }
                if let Some(piece) = self.board[from as usize] {
                    if piece.color == by && piece.kind == PieceKind::Knight {
                        return true;
                    }
                }
            }
        }
        // Sliding pieces
        let directions = [
            (8, PieceKind::Rook),
            (-8, PieceKind::Rook),
            (1, PieceKind::Rook),
            (-1, PieceKind::Rook),
            (9, PieceKind::Bishop),
            (7, PieceKind::Bishop),
            (-7, PieceKind::Bishop),
            (-9, PieceKind::Bishop),
        ];
        for (dir, base) in directions.iter() {
            let mut cur = sq as i16;
            loop {
                cur += *dir as i16;
                if cur < 0 || cur >= 64 {
                    break;
                }
                let from = cur as u8;
                if wraps(sq, from, *dir) {
                    break;
                }
                if let Some(piece) = self.board[from as usize] {
                    if piece.color != by {
                        break;
                    }
                    match (base, piece.kind) {
                        (PieceKind::Rook, PieceKind::Rook | PieceKind::Queen) => return true,
                        (PieceKind::Bishop, PieceKind::Bishop | PieceKind::Queen) => return true,
                        _ => {}
                    }
                    break;
                }
            }
        }
        // King
        for off in [1, -1, 8, -8, 9, 7, -7, -9].iter() {
            let target = sq as i16 + *off as i16;
            if target >= 0 && target < 64 {
                let from = target as u8;
                if king_wraps(from, sq) {
                    continue;
                }
                if let Some(piece) = self.board[from as usize] {
                    if piece.color == by && piece.kind == PieceKind::King {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MoveList {
    pub moves: [Move; MoveList::MAX_MOVES],
    pub len: usize,
}

impl MoveList {
    // Theoretical upper bound of chess branching factor used to bound the array.
    pub const MAX_MOVES: usize = 218; // Upper bound for chess branching factor.

    pub const fn new() -> MoveList {
        MoveList {
            moves: [Move {
                from: 0,
                to: 0,
                promotion: None,
                is_en_passant: false,
                is_castling: false,
            }; MoveList::MAX_MOVES],
            len: 0,
        }
    }

    fn push(&mut self, mv: Move) {
        if self.len < Self::MAX_MOVES {
            self.moves[self.len] = mv;
            self.len += 1;
        }
    }

    fn retain<F: FnMut(&Move) -> bool>(&mut self, mut f: F) {
        let mut write = 0;
        for i in 0..self.len {
            let mv = self.moves[i];
            if f(&mv) {
                self.moves[write] = mv;
                write += 1;
            }
        }
        self.len = write;
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Move> {
        self.moves[..self.len].iter()
    }
}

pub struct Engine {
    state: GameState,
}

impl Engine {
    // Construct engine with standard initial position.
    pub const fn new_startpos() -> Engine {
        Engine {
            state: GameState::start_position(),
        }
    }

    // Read-only access to current state.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    // List all legal moves from current state.
    pub fn legal_moves(&self) -> MoveList {
        self.state.generate_legal_moves()
    }

    // Play a move if legal; otherwise return MoveError.
    pub fn play_move(&mut self, mv: Move) -> Result<(), MoveError> {
        match self.state.make_move(mv) {
            Some(next) => {
                self.state = next;
                Ok(())
            }
            None => Err(MoveError::Illegal),
        }
    }
}

#[derive(Debug)]
pub enum MoveError {
    Illegal,
}

fn rank_of(sq: u8) -> u8 {
    sq / 8
}

fn file_of(sq: u8) -> u8 {
    sq % 8
}

fn file_distance(a: u8, b: u8) -> u8 {
    let fa = file_of(a);
    let fb = file_of(b);
    if fa > fb { fa - fb } else { fb - fa }
}

fn wraps(from: u8, to: u8, dir: i8) -> bool {
    let f_from = file_of(from);
    let f_to = file_of(to);
    match dir {
        1 | -1 | 9 | -7 => f_to <= f_from.wrapping_sub(1),
        -9 | 7 => f_to >= f_from.wrapping_add(1),
        _ => false,
    }
}

fn knight_move_wraps(from: u8, to: u8) -> bool {
    let df = file_distance(from, to);
    df == 0 || df > 2
}

fn king_wraps(from: u8, to: u8) -> bool {
    file_distance(from, to) > 1
}

#[cfg(feature = "std")]
impl core::fmt::Display for GameState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let idx = rank * 8 + file;
                let symbol = match self.board[idx as usize] {
                    None => '.',
                    Some(Piece { color, kind }) => match (color, kind) {
                        (Color::White, PieceKind::Pawn) => 'P',
                        (Color::White, PieceKind::Knight) => 'N',
                        (Color::White, PieceKind::Bishop) => 'B',
                        (Color::White, PieceKind::Rook) => 'R',
                        (Color::White, PieceKind::Queen) => 'Q',
                        (Color::White, PieceKind::King) => 'K',
                        (Color::Black, PieceKind::Pawn) => 'p',
                        (Color::Black, PieceKind::Knight) => 'n',
                        (Color::Black, PieceKind::Bishop) => 'b',
                        (Color::Black, PieceKind::Rook) => 'r',
                        (Color::Black, PieceKind::Queen) => 'q',
                        (Color::Black, PieceKind::King) => 'k',
                    },
                };
                write!(f, "{} ", symbol)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Side: {:?}", self.side_to_move)
    }
}

pub mod ai;
