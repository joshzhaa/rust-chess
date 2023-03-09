use crate::board::Vector;
use crate::game::Game;

// each piece keeps track of what kind it is (K, Q, R, ...) and which player controls it (1, 2, 3, ...)
#[derive(Clone, PartialEq, Debug)]
pub struct Piece {
    pub id: char,
    pub owner: u32,
    pub has_moved: bool,
}

impl Piece {
    pub fn new() -> Piece {
        Piece {
            id: ' ',
            owner: 0,
            has_moved: false,
        }
    }
    fn is_opposed(&self, other: &Piece) -> bool {
        self.owner != other.owner
    }
    pub fn validity_func(&self) -> impl FnMut(&Vector, &mut Game) {
        match self.id {
            'K' => |pos: &Vector, game: &mut Game| {
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i != 0 || j != 0 {
                            game.attack(&(pos.clone() + Vector(i, j)), true);
                        }
                    }
                }
            },
            'Q' => |pos: &Vector, game: &mut Game| {
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i != 0 || j != 0 {
                            extend(pos, Vector(i, j), game);
                        }
                    }
                }
            },
            'R' => |pos: &Vector, game: &mut Game| {
                extend(pos, Vector(0, 1), game); // North
                extend(pos, Vector(1, 0), game); // East
                extend(pos, Vector(0, -1), game); // South
                extend(pos, Vector(-1, 0), game); // West
            },
            'B' => |pos: &Vector, game: &mut Game| {
                extend(pos, Vector(1, 1), game); // NE
                extend(pos, Vector(1, -1), game); // SE
                extend(pos, Vector(-1, -1), game); // SW
                extend(pos, Vector(-1, 1), game); // NW
            },
            'N' => |pos: &Vector, game: &mut Game| {
                //counterclockwise from positive x-axis
                let offsets = [
                    Vector(2, 1),
                    Vector(1, 2),
                    Vector(-1, 2),
                    Vector(-2, 1),
                    Vector(-2, -1),
                    Vector(-1, -2),
                    Vector(1, -2),
                    Vector(2, -1),
                ];
                for offset in offsets {
                    game.attack(&(pos.clone() + offset), true);
                }
            },
            // pawn logic is fundamentally horrific
            'P' => |pos: &Vector, game: &mut Game| {
                // most pieces don't capture self, capturing self would change closure signature,
                // not sure if there is a better workaround in the language
                let piece = game.get_piece(pos);
                let unit_vec = game.get_player(piece.owner).direction.clone();
                // initial double move logic
                let has_moved = piece.has_moved;
                let near_is_empty = game.get_piece(&(pos.clone() + unit_vec.clone())).id == ' ';
                let far_is_empty = game.get_piece(&(pos.clone() + unit_vec.clone() * 2)).id == ' ';
                // diagonal capture logic
                // 90 deg rotation matrix, useful misnomer, only correct if unit_vec is down
                let right = Vector(-1 * unit_vec.1, unit_vec.0);
                let left = right.clone() * -1;
                let diag_right = pos.clone() + unit_vec.clone() + right.clone();
                let diag_left = pos.clone() + unit_vec.clone() + left.clone();
                let can_diagonal = |pos: Vector| {
                    let diag_piece = game.get_piece(&pos);
                    diag_piece.id != ' ' && diag_piece.is_opposed(&piece)
                };
                let can_right = can_diagonal(diag_right.clone());
                let can_left = can_diagonal(diag_left.clone());
                // en passant logic
                let can_passant = |pos: Vector| {
                    let passant_piece = game.get_piece(&pos);
                    if passant_piece.id == ' ' {
                        return false;
                    }
                    let passant_victim = game.get_player(passant_piece.owner);
                    if passant_victim.recent_move.is_none() {
                        return false;
                    }
                    let recent_move = passant_victim.recent_move.clone().unwrap();
                    passant_piece.id == 'P'
                        && passant_piece.is_opposed(&piece)
                        && recent_move.square_dist() == 4
                };
                let right_passant = can_passant(pos.clone() + right.clone());
                let left_passant = can_passant(pos.clone() + left.clone());
                // carry out attacks, needs local variables logic *then* attack due to mut borrow rules
                if can_right || right_passant {
                    game.attack(&diag_right, true);
                }
                if can_left || left_passant {
                    game.attack(&diag_left, true);
                }
                if near_is_empty {
                    game.attack(&(pos.clone() + unit_vec.clone()), false);
                    if !has_moved && far_is_empty {
                        game.attack(&(pos.clone() + unit_vec * 2), false);
                    }
                }
            },
            // branch should never occur
            _ => |_pos: &Vector, _game: &mut Game| {},
        }
    }
    // functions must be called as move is being made,
    // before pieces are swapped, after player's recent_move field has been updated
    pub fn side_effects(&self) -> impl FnMut(&mut Game) {
        match self.id {
            // destroy en passant'ed pawn
            'P' => |game: &mut Game| {
                let player = game.current_player();
                let recent_move = player.recent_move.clone().unwrap();
                let end_pos = recent_move.end.clone();
                // recent_move guaranteed to be Some
                let is_capture = game.get_piece(&end_pos).id != ' ';
                if !is_capture && recent_move.is_diag() {
                    let unit_vec = player.direction.clone();
                    game.board[&(end_pos + unit_vec * -1)] = Self::new();
                }
            },
            _ => |_game: &mut Game| {},
        }
    }
}

// sequentially attack in a direction, common to Q R and B
fn extend(pos: &Vector, direction: Vector, game: &mut Game) {
    let mut target = pos.clone();
    loop {
        target += direction.clone();
        if !game.attack(&target, true) {
            break;
        }
    }
}
