use super::{Color, GameState, Move, MoveList, PieceKind, book};

// Mate score large enough to dominate any material/eval.
const MATE_SCORE: i32 = 30_000;

// Tiny transposition table: 2^10 = 1024 entries (~24 KB).
const TT_BITS: usize = 10;
const TT_SIZE: usize = 1 << TT_BITS;
const TT_MASK: usize = TT_SIZE - 1;

#[derive(Clone, Copy)]
pub struct AiConfig {
    /// Maximum search depth for iterative deepening (plies).
    pub max_depth: u8,
    /// Optional safety cap on explored nodes.
    pub node_limit: Option<u32>,
}

impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            max_depth: 6,
            node_limit: Some(20_000),
        }
    }
}

pub fn choose_best_move<F: FnMut()>(
    state: &GameState,
    ai_color: Color,
    cfg: AiConfig,
    mut tick: F,
) -> Option<Move> {
    if state.side_to_move != ai_color {
        return None;
    }

    // 开局表优先，匹配不到再进入搜索。
    if let Some(book_mv) = book::book_move(state) {
        return Some(book_mv);
    }
    let depth_limit = cfg.max_depth.clamp(1, 8);
    let mut moves = state.generate_legal_moves();
    if moves.len == 0 {
        return None;
    }

    let mut ctx = SearchCtx::new(cfg.node_limit);
    let mut best = None;
    let mut best_score = i32::MIN + 1;

    for depth in 1..=depth_limit {
        tick();
        let hash = zobrist(state);
        let tt_hint = ctx.tt_probe(hash).and_then(|e| e.best_move);

        sort_moves(state, &mut moves, tt_hint, true);
        let mut local_best = best;
        let mut local_best_score = i32::MIN + 1;

        for mv in moves.iter() {
            if let Some(next) = state.make_move(*mv) {
                tick();
                ctx.bump();
                let score = alphabeta(
                    &next,
                    ai_color,
                    depth.saturating_sub(1),
                    i32::MIN + 1,
                    i32::MAX - 1,
                    &mut ctx,
                );
                if score > local_best_score {
                    local_best_score = score;
                    local_best = Some(*mv);
                }
            }
            if ctx.hit_limit() {
                break;
            }
        }

        if local_best.is_some() {
            best = local_best;
            best_score = local_best_score;
        }

        if ctx.hit_limit() {
            break;
        }
    }

    let _ = best_score; // placeholder to avoid warnings when logging is off.
    best
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone, Copy)]
struct TtEntry {
    key: u64,
    depth: u8,
    value: i32,
    flag: Bound,
    best_move: Option<Move>,
}

impl TtEntry {
    const EMPTY: TtEntry = TtEntry {
        key: 0,
        depth: 0,
        value: 0,
        flag: Bound::Exact,
        best_move: None,
    };
}

struct SearchCtx {
    nodes: u32,
    node_limit: Option<u32>,
    tt: [TtEntry; TT_SIZE],
}

impl SearchCtx {
    fn new(node_limit: Option<u32>) -> Self {
        SearchCtx {
            nodes: 0,
            node_limit,
            tt: [TtEntry::EMPTY; TT_SIZE],
        }
    }

    fn bump(&mut self) {
        self.nodes = self.nodes.saturating_add(1);
    }

    fn hit_limit(&self) -> bool {
        match self.node_limit {
            Some(limit) => self.nodes >= limit,
            None => false,
        }
    }

    fn tt_probe(&self, key: u64) -> Option<TtEntry> {
        let idx = (key as usize) & TT_MASK;
        let entry = self.tt[idx];
        if entry.key == key { Some(entry) } else { None }
    }

    fn tt_store(&mut self, key: u64, depth: u8, value: i32, flag: Bound, best_move: Option<Move>) {
        let idx = (key as usize) & TT_MASK;
        let entry = &mut self.tt[idx];
        if entry.key != key || depth >= entry.depth {
            *entry = TtEntry {
                key,
                depth,
                value,
                flag,
                best_move,
            };
        }
    }
}

fn alphabeta(
    state: &GameState,
    ai_color: Color,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    ctx: &mut SearchCtx,
) -> i32 {
    let orig_alpha = alpha;
    let orig_beta = beta;
    ctx.bump();
    if ctx.hit_limit() {
        return evaluate(state, ai_color);
    }

    let hash = zobrist(state);

    if let Some(entry) = ctx.tt_probe(hash) {
        if entry.depth >= depth {
            match entry.flag {
                Bound::Exact => return entry.value,
                Bound::Lower if entry.value > alpha => alpha = entry.value,
                Bound::Upper if entry.value < beta => beta = entry.value,
                _ => {}
            }
            if alpha >= beta {
                return entry.value;
            }
        }
    }

    if depth == 0 {
        return quiesce(state, ai_color, alpha, beta, ctx);
    }

    let mut moves = state.generate_legal_moves();
    if moves.len == 0 {
        return terminal_score(state, ai_color);
    }

    let tt_hint = ctx.tt_probe(hash).and_then(|e| e.best_move);
    sort_moves(state, &mut moves, tt_hint, state.side_to_move == ai_color);

    let maximizing = state.side_to_move == ai_color;
    let mut best = if maximizing {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };
    let mut best_move = None;

    for mv in moves.iter() {
        if let Some(next) = state.make_move(*mv) {
            let score = alphabeta(&next, ai_color, depth - 1, alpha, beta, ctx);
            if maximizing {
                if score > best {
                    best = score;
                    best_move = Some(*mv);
                }
                if best > alpha {
                    alpha = best;
                }
            } else if score < best {
                best = score;
                best_move = Some(*mv);
                if best < beta {
                    beta = best;
                }
            }
        }
        if beta <= alpha || ctx.hit_limit() {
            break;
        }
    }

    let flag = if best <= orig_alpha {
        Bound::Upper
    } else if best >= orig_beta {
        Bound::Lower
    } else {
        Bound::Exact
    };
    ctx.tt_store(hash, depth, best, flag, best_move);
    best
}

fn quiesce(
    state: &GameState,
    ai_color: Color,
    mut alpha: i32,
    beta: i32,
    ctx: &mut SearchCtx,
) -> i32 {
    let stand_pat = evaluate(state, ai_color);
    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let mut moves = state.generate_legal_moves();
    if moves.len == 0 {
        return terminal_score(state, ai_color);
    }

    sort_moves(state, &mut moves, None, true);
    let maximizing = state.side_to_move == ai_color;
    if maximizing {
        let mut best = alpha;
        for mv in moves.iter() {
            if !is_capture(state, *mv) && mv.promotion.is_none() {
                continue;
            }
            if let Some(next) = state.make_move(*mv) {
                ctx.bump();
                let score = quiesce(&next, ai_color, best, beta, ctx);
                if score > best {
                    best = score;
                }
                if best >= beta || ctx.hit_limit() {
                    break;
                }
            }
        }
        best
    } else {
        let mut best = beta;
        for mv in moves.iter() {
            if !is_capture(state, *mv) && mv.promotion.is_none() {
                continue;
            }
            if let Some(next) = state.make_move(*mv) {
                ctx.bump();
                let score = quiesce(&next, ai_color, alpha, best, ctx);
                if score < best {
                    best = score;
                }
                if best <= alpha || ctx.hit_limit() {
                    break;
                }
            }
        }
        best
    }
}

fn terminal_score(state: &GameState, ai_color: Color) -> i32 {
    if state.is_in_check(state.side_to_move) {
        if state.side_to_move == ai_color {
            -MATE_SCORE
        } else {
            MATE_SCORE
        }
    } else {
        0
    }
}

fn evaluate(state: &GameState, ai_color: Color) -> i32 {
    // Material + PST + small check bonus/penalty.
    let mut score = 0i32;
    for sq in 0..64 {
        if let Some(piece) = state.board[sq] {
            let val = piece_value(piece.kind);
            let pst = piece_square_bonus(piece.kind, piece.color, sq as u8);
            let total = val + pst as i32;
            score += if piece.color == ai_color {
                total
            } else {
                -total
            };
        }
    }

    if state.is_in_check(state.side_to_move) {
        if state.side_to_move == ai_color {
            score -= 30;
        } else {
            score += 30;
        }
    }
    score
}

fn piece_square_bonus(kind: PieceKind, color: Color, sq: u8) -> i16 {
    let idx = match color {
        Color::White => sq as usize,
        Color::Black => mirror_square(sq) as usize,
    };
    match kind {
        PieceKind::Pawn => PAWN_PST[idx],
        PieceKind::Knight => KNIGHT_PST[idx],
        PieceKind::Bishop => BISHOP_PST[idx],
        PieceKind::Rook => ROOK_PST[idx],
        PieceKind::Queen => QUEEN_PST[idx],
        PieceKind::King => KING_PST[idx],
    }
}

fn mirror_square(sq: u8) -> u8 {
    let file = sq % 8;
    let rank = sq / 8;
    (7 - rank) * 8 + file
}

fn is_capture(state: &GameState, mv: Move) -> bool {
    mv.is_en_passant || state.board[mv.to as usize].is_some()
}

fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0,
    }
}

fn move_heuristic(state: &GameState, mv: Move, tt_hint: Option<Move>) -> i32 {
    if tt_hint.map_or(false, |m| m == mv) {
        return 10_000;
    }

    let mut score = 0;
    if mv.is_castling {
        score += 50;
    }

    // Capture ordering: MVV/LVA.
    if mv.is_en_passant {
        score += 800;
    } else if let Some(target) = state.board[mv.to as usize] {
        let victim = piece_value(target.kind);
        let attacker = state.board[mv.from as usize]
            .map(|p| piece_value(p.kind))
            .unwrap_or(100);
        score += victim * 10 - attacker;
    }

    if let Some(prom) = mv.promotion {
        score += piece_value(prom) + 400;
    }

    score
}

fn sort_moves(state: &GameState, list: &mut MoveList, tt_hint: Option<Move>, descending: bool) {
    // Simple insertion sort using heuristic; cheap for small lists.
    let mut i = 1;
    while i < list.len {
        let key = list.moves[i];
        let key_h = move_heuristic(state, key, tt_hint);
        let mut j = i;
        while j > 0 {
            let prev = list.moves[j - 1];
            let prev_h = move_heuristic(state, prev, tt_hint);
            let swap = if descending {
                key_h > prev_h
            } else {
                key_h < prev_h
            };
            if swap {
                list.moves[j] = prev;
                j -= 1;
            } else {
                break;
            }
        }
        list.moves[j] = key;
        i += 1;
    }
}

// Zobrist hashing for TT keys.
fn zobrist(state: &GameState) -> u64 {
    let mut h = 0u64;
    for idx in 0..64u8 {
        if let Some(piece) = state.board[idx as usize] {
            let piece_idx = piece_index(piece.color, piece.kind);
            h ^= zobrist_key(piece_idx, idx);
        }
    }
    if state.side_to_move == Color::White {
        h ^= SIDE_KEY;
    }
    h
}

fn piece_index(color: Color, kind: PieceKind) -> usize {
    let base = match color {
        Color::White => 0,
        Color::Black => 6,
    };
    base + match kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    }
}

fn zobrist_key(piece_idx: usize, square: u8) -> u64 {
    // SplitMix64 keyed by piece+square ensures deterministic hash without large tables.
    let mut x = ((piece_idx as u64) << 8) ^ square as u64 ^ 0x9E37_79B9_7F4A_7C15;
    x = x.wrapping_add(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

// Piece-square tables (coarse, midgame-oriented).
const PAWN_PST: [i16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, -5, -5, 5, 5, 5, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 2, 3, 3, 2, 1,
    1, 1, 1, 1, 2, 2, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, -1, -1, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0,
    0, 0, 0,
];

const KNIGHT_PST: [i16; 64] = [
    -5, -4, -3, -3, -3, -3, -4, -5, -4, -2, 0, 0, 0, 0, -2, -4, -3, 0, 1, 1, 1, 1, 0, -3, -3, 0, 2,
    3, 3, 2, 0, -3, -3, 0, 2, 3, 3, 2, 0, -3, -3, 0, 1, 2, 2, 1, 0, -3, -4, -2, 0, 0, 0, 0, -2, -4,
    -5, -4, -3, -3, -3, -3, -4, -5,
];

const BISHOP_PST: [i16; 64] = [
    -2, -1, -1, -1, -1, -1, -1, -2, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 1, 1, 1, 1, 0, -1, -1, 1, 1,
    1, 1, 1, 1, -1, -1, 0, 1, 1, 1, 1, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -2, -1, -1, -1, -1, -1, -1,
    -2, -2, -1, -1, -1, -1, -1, -1, -2,
];

const ROOK_PST: [i16; 64] = [
    0, 0, 1, 2, 2, 1, 0, 0, -2, -2, -2, -2, -2, -2, -2, -2, -1, -1, 0, 0, 0, 0, -1, -1, -1, -1, 0,
    0, 0, 0, -1, -1, -1, -1, 0, 0, 0, 0, -1, -1, -1, -1, 0, 1, 1, 0, -1, -1, -1, -1, 2, 2, 2, 2,
    -1, -1, 0, 0, 0, 0, 2, 2, 0, 0,
];

const QUEEN_PST: [i16; 64] = [
    -4, -2, -2, -1, -1, -2, -2, -4, -2, 0, 0, 0, 0, 0, 0, -2, -2, 0, 1, 1, 1, 1, 0, -2, -1, 0, 1,
    1, 1, 1, 0, -1, 0, 0, 1, 1, 1, 1, 0, -1, -1, 0, 1, 1, 1, 1, 0, -1, -2, -2, 0, 0, 0, 0, -2, -2,
    -4, -2, -2, -1, -1, -2, -2, -4,
];

const KING_PST: [i16; 64] = [
    -3, -4, -4, -5, -5, -4, -4, -3, -3, -4, -4, -5, -5, -4, -4, -3, -3, -4, -4, -5, -5, -4, -4, -3,
    -3, -4, -4, -5, -5, -4, -4, -3, -2, -3, -3, -4, -4, -3, -3, -2, -1, -2, -2, -2, -2, -2, -2, -1,
    2, 2, 0, 0, 0, 0, 2, 2, 2, 3, 1, 0, 0, 1, 3, 2,
];

// Zobrist side key (piece-square keys are generated on the fly).
const SIDE_KEY: u64 = 0x9E37_79B9_7F4A_7C15;
