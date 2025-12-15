use super::{Color, GameState, Move, MoveList, PieceKind};

// Mate score is large enough to dominate any material difference.
const MATE_SCORE: i32 = 30_000;

#[derive(Clone, Copy)]
pub struct AiConfig {
    pub max_depth: u8,           // Clamped to 1..=3
    pub node_limit: Option<u32>, // Optional fail-safe for UI responsiveness
}

impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            max_depth: 3,
            node_limit: None,
        }
    }
}

pub fn choose_best_move(state: &GameState, ai_color: Color, cfg: AiConfig) -> Option<Move> {
    if state.side_to_move != ai_color {
        return None;
    }

    let depth = cfg.max_depth.clamp(1, 3);
    let mut moves = state.generate_legal_moves();
    if moves.len == 0 {
        return None;
    }

    let mut ctx = SearchCtx {
        nodes: 0,
        node_limit: cfg.node_limit,
    };

    let mut best_mv = None;
    let mut best_score = i32::MIN;
    sort_moves(state, &mut moves, true);
    for mv in moves.iter() {
        if let Some(next) = state.make_move(*mv) {
            ctx.bump();
            let score = alphabeta(
                &next,
                ai_color,
                depth.saturating_sub(1),
                i32::MIN + 1,
                i32::MAX - 1,
                &mut ctx,
            );
            if score > best_score {
                best_score = score;
                best_mv = Some(*mv);
            }
        }
        if ctx.hit_limit() {
            break;
        }
    }
    best_mv
}

#[derive(Clone, Copy)]
struct SearchCtx {
    nodes: u32,
    node_limit: Option<u32>,
}

impl SearchCtx {
    fn bump(&mut self) {
        self.nodes = self.nodes.saturating_add(1);
    }

    fn hit_limit(&self) -> bool {
        match self.node_limit {
            Some(limit) => self.nodes >= limit,
            None => false,
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
    ctx.bump();
    if depth == 0 || ctx.hit_limit() {
        return evaluate(state, ai_color);
    }

    let mut moves = state.generate_legal_moves();
    if moves.len == 0 {
        return evaluate(state, ai_color);
    }

    let maximizing = state.side_to_move == ai_color;
    sort_moves(state, &mut moves, maximizing);

    if maximizing {
        let mut best = i32::MIN;
        for mv in moves.iter() {
            if let Some(next) = state.make_move(*mv) {
                let score = alphabeta(&next, ai_color, depth - 1, alpha, beta, ctx);
                if score > best {
                    best = score;
                }
                if best > alpha {
                    alpha = best;
                }
                if beta <= alpha || ctx.hit_limit() {
                    break;
                }
            }
        }
        best
    } else {
        let mut best = i32::MAX;
        for mv in moves.iter() {
            if let Some(next) = state.make_move(*mv) {
                let score = alphabeta(&next, ai_color, depth - 1, alpha, beta, ctx);
                if score < best {
                    best = score;
                }
                if best < beta {
                    beta = best;
                }
                if beta <= alpha || ctx.hit_limit() {
                    break;
                }
            }
        }
        best
    }
}

fn evaluate(state: &GameState, ai_color: Color) -> i32 {
    // Terminal checks: checkmate/stalemate detection via move list and check status.
    let moves = state.generate_legal_moves();
    if moves.len == 0 {
        if state.is_in_check(state.side_to_move) {
            return if state.side_to_move == ai_color {
                -MATE_SCORE
            } else {
                MATE_SCORE
            };
        } else {
            return 0; // Stalemate
        }
    }

    let mut score = 0i32;
    for sq in 0..64 {
        if let Some(piece) = state.board[sq] {
            let val = piece_value(piece.kind);
            score += if piece.color == ai_color { val } else { -val };
        }
    }

    // Simple check bonus/penalty to encourage delivering checks.
    if state.is_in_check(state.side_to_move) {
        if state.side_to_move == ai_color {
            score -= 20;
        } else {
            score += 20;
        }
    }
    score
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

fn move_heuristic(state: &GameState, mv: Move) -> i32 {
    let mut score = 0;
    // Promotions are highly valuable.
    if let Some(prom) = mv.promotion {
        score += piece_value(prom) + 400;
    }

    // Capture value: direct capture or en-passant.
    if mv.is_en_passant {
        let captured_sq = if let Some(piece) = state.board[mv.from as usize] {
            match piece.color {
                Color::White => mv.to.wrapping_sub(8),
                Color::Black => mv.to.wrapping_add(8),
            }
        } else {
            mv.to
        };
        if let Some(p) = state.board[captured_sq as usize] {
            score += piece_value(p.kind);
        } else {
            score += 50; // Fallback bonus.
        }
    } else if let Some(p) = state.board[mv.to as usize] {
        score += piece_value(p.kind);
    }

    // Encourage castling slightly.
    if mv.is_castling {
        score += 30;
    }

    score
}

fn sort_moves(state: &GameState, list: &mut MoveList, descending: bool) {
    // Simple insertion sort using heuristic; stable enough for tiny lists.
    let mut i = 1;
    while i < list.len {
        let key = list.moves[i];
        let key_h = move_heuristic(state, key);
        let mut j = i;
        while j > 0 {
            let prev = list.moves[j - 1];
            let prev_h = move_heuristic(state, prev);
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

// Example integration (pseudo):
// if engine.state().side_to_move == ai_color {
//     let mv = choose_best_move(engine.state(), ai_color, AiConfig::default());
//     if let Some(m) = mv {
//         engine.play_move(m).unwrap();
//     }
// }
