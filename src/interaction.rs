use crate::board;
use crate::drivers::button::PressKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    ToggleSelect,
    SubmitMove,
}

#[derive(Clone, Copy, Debug)]
pub enum PromotionChoice {
    Rook,
    Knight,
    Bishop,
    Queen,
}

pub fn poll_action(board: &mut board::Board) -> Option<Action> {
    if let Some(press) = board.buttons.key1_press(&mut board.delay) {
        return Some(match press {
            PressKind::Short => Action::MoveLeft,
            PressKind::Long => Action::ToggleSelect,
        });
    }
    if let Some(press) = board.buttons.key2_press(&mut board.delay) {
        return Some(match press {
            PressKind::Short => Action::MoveDown,
            PressKind::Long => Action::SubmitMove,
        });
    }
    if let Some(press) = board.buttons.key3_press(&mut board.delay) {
        return match press {
            PressKind::Short => Some(Action::MoveUp),
            PressKind::Long => None, // 未定义长按行为
        };
    }
    if let Some(press) = board.buttons.key4_press(&mut board.delay) {
        return match press {
            PressKind::Short => Some(Action::MoveRight),
            PressKind::Long => None, // 未定义长按行为
        };
    }
    None
}

/// 升变选择：短按 KEY1..KEY4 分别对应 车/马/象/后。
pub fn poll_promotion_choice(board: &mut board::Board) -> Option<PromotionChoice> {
    if let Some(press) = board.buttons.key1_press(&mut board.delay) {
        if matches!(press, PressKind::Short) {
            return Some(PromotionChoice::Rook);
        }
    }
    if let Some(press) = board.buttons.key2_press(&mut board.delay) {
        if matches!(press, PressKind::Short) {
            return Some(PromotionChoice::Knight);
        }
    }
    if let Some(press) = board.buttons.key3_press(&mut board.delay) {
        if matches!(press, PressKind::Short) {
            return Some(PromotionChoice::Bishop);
        }
    }
    if let Some(press) = board.buttons.key4_press(&mut board.delay) {
        if matches!(press, PressKind::Short) {
            return Some(PromotionChoice::Queen);
        }
    }
    None
}
