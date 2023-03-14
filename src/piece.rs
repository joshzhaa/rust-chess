use crate::board::{Matrix, Vector};
use crate::game::{Game, GameState};

// each piece keeps track of what kind it is (K, Q, R, ...) and which player controls it (1, 2, 3, ...)
#[derive(Clone, PartialEq, Debug)]
pub struct Piece {
    pub id: char,
    pub owner: usize, // indexes into a vector of players
    pub has_moved: bool,
}

fn init_masks(game: &GameState) -> (Matrix<bool>, Matrix<bool>) {
    let (rows, cols) = game.board.shape();
    (
        Matrix::new(false, rows, cols),
        Matrix::new(false, rows, cols),
    )
}

impl Piece {
    pub fn new() -> Piece {
        Piece {
            id: ' ',
            owner: 0,
            has_moved: false,
        }
    }

    pub fn validity_func(&self) -> impl FnOnce(&Vector, &Game) -> (Matrix<bool>, Matrix<bool>) {
        match self.id {
            'K' => move |pos: &Vector, game: &Game| -> (Matrix<bool>, Matrix<bool>) {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game.state);
                let offsets = vec![
                    Vector(-1, -1),
                    Vector(-1, 0),
                    Vector(-1, 1),
                    Vector(0, -1),
                    Vector(0, 1),
                    Vector(1, -1),
                    Vector(1, 0),
                    Vector(1, 1),
                ];
                // check whether a square is threatened by other players
                let mut threatened = threat.clone(); // accumulated threat matrix of all pieces
                let mut or_assign = |pos: &Vector| {
                    // to prevent infinite recursion
                    if game.get_piece(pos).id == 'K' {
                        for offset in &offsets {
                            let target = pos.clone() + offset.clone();
                            if game.state.board.in_bounds(&target) {
                                threatened[&target] = true;
                            }
                        }
                        return;
                    }
                    // handle non-king pieces
                    let (_, threat) = game.state.board[pos].validity_func()(pos, game);
                    for row in 0..threat.0.len() as i32 {
                        for col in 0..threat.0[0].len() as i32 {
                            let pos = Vector(col, row);
                            threatened[&pos] = threatened[&pos] || threat[&pos];
                        }
                    }
                    println!("{}", threatened);
                };
                for row in 0..game.state.board.0 .0.len() as i32 {
                    for col in 0..game.state.board.0 .0[0].len() as i32 {
                        let attacker = game.get_piece(&Vector(col, row));
                        let attacked = game.get_piece(pos);
                        println!("investigate pos ({col}, {row}) with piece {attacker:?} to {attacked:?}");
                        if attacker.owner != attacked.owner {
                            or_assign(&Vector(col, row));
                        }
                    }
                }
                println!("{}", threatened);
                // mark standard moves
                for offset in offsets {
                    let target = pos.clone() + offset;
                    if game.state.board.in_bounds(&target) && !threatened[&target] {
                        this.attack(target, &game.state, &mut valid, Some(&mut threat));
                    }
                }
                // mark castles
                let unit_vec = game.get_player(this.owner).direction.clone();
                let can_castle = |direction: &Vector| {
                    // def of castle here
                    if game.in_bounds(&(pos.clone() + unit_vec.clone() * -1)) || this.has_moved {
                        return false;
                    }
                    let friendly_rook =
                        |piece: &Piece| piece.id == 'R' && piece.owner == this.owner;
                    let mut target = pos.clone();
                    loop {
                        let piece = game.get_piece(&target);
                        if threatened[&target] {
                            return false;
                        } else if friendly_rook(piece) {
                            return !piece.has_moved;
                        } else if piece.id != ' ' && target != *pos {
                            return false;
                        }
                        target += direction.clone();
                        if !game.in_bounds(&target) {
                            return false;
                        }
                    }
                };
                let right = Vector(-1 * unit_vec.1, unit_vec.0);
                let left = right.clone() * -1;
                if can_castle(&right) {
                    let target = pos.clone() + right * 2;
                    this.attack(target, &game.state, &mut valid, Some(&mut threat));
                }
                if can_castle(&left) {
                    let target = pos.clone() + left * 2;
                    this.attack(target, &game.state, &mut valid, Some(&mut threat));
                }
                (valid, threat)
            },
            'Q' => move |pos: &Vector, game: &Game| -> (Matrix<bool>, Matrix<bool>) {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game.state);
                let directions = vec![
                    Vector(-1, -1),
                    Vector(-1, 0),
                    Vector(-1, 1),
                    Vector(0, -1),
                    Vector(0, 1),
                    Vector(1, -1),
                    Vector(1, 0),
                    Vector(1, 1),
                ];
                this.extend(pos, directions, &game.state, &mut valid, &mut threat);
                (valid, threat)
            },
            'R' => move |pos: &Vector, game: &Game| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game.state);
                let directions = vec![Vector(0, 1), Vector(1, 0), Vector(0, -1), Vector(-1, 0)];
                this.extend(pos, directions, &game.state, &mut valid, &mut threat);
                (valid, threat)
            },
            'B' => move |pos: &Vector, game: &Game| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game.state);
                let directions = vec![
                    Vector(1, 1),   // NE
                    Vector(1, -1),  // SE
                    Vector(-1, -1), // SW
                    Vector(-1, 1),  // NW
                ];
                this.extend(pos, directions, &game.state, &mut valid, &mut threat);
                (valid, threat)
            },
            'N' => |pos: &Vector, game: &Game| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game.state);
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
                    let target = pos.clone() + offset;
                    this.attack(target, &game.state, &mut valid, Some(&mut threat));
                }
                (valid, threat)
            },
            // pawn logic is fundamentally horrific
            'P' => move |pos: &Vector, game: &Game| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game.state);
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
                    if !game.state.board.in_bounds(&pos) {
                        return false;
                    };
                    let diag_piece = game.get_piece(&pos);
                    diag_piece.id != ' ' && diag_piece.owner != piece.owner
                };
                let can_right = can_diagonal(diag_right.clone());
                let can_left = can_diagonal(diag_left.clone());
                // en passant logic
                let can_passant = |pos: Vector| {
                    if !game.state.board.in_bounds(&pos) {
                        return false;
                    }
                    let passant_piece = game.get_piece(&pos);
                    // if guard b/c get_player will cause underflow
                    if passant_piece.id == ' ' {
                        return false;
                    }
                    let passant_victim = game.get_player(passant_piece.owner);
                    if passant_victim.recent_move.is_none() {
                        return false;
                    }
                    let recent_move = passant_victim.recent_move.clone().unwrap();
                    passant_piece.id == 'P'
                        && passant_piece.owner != piece.owner
                        && recent_move.square_dist() == 4
                        && recent_move.end == pos
                };
                let right_passant = can_passant(pos.clone() + right.clone());
                let left_passant = can_passant(pos.clone() + left.clone());
                // need to always threaten diag squares to communicate that enemy K cannot move there
                if game.state.board.in_bounds(&diag_right) {
                    threat[&diag_right] = true;
                }
                if game.state.board.in_bounds(&diag_left) {
                    threat[&diag_left] = true;
                }
                // carry out attacks, needs local variables logic *then* attack due to mut borrow rules
                if can_right || right_passant {
                    this.attack(diag_right, &game.state, &mut valid, Some(&mut threat));
                }
                if can_left || left_passant {
                    this.attack(diag_left, &game.state, &mut valid, Some(&mut threat));
                }
                if near_is_empty {
                    let near = pos.clone() + unit_vec.clone();
                    this.attack(near, &game.state, &mut valid, None);
                    if !has_moved && far_is_empty {
                        let far = pos.clone() + unit_vec.clone() * 2;
                        this.attack(far, &game.state, &mut valid, None);
                    }
                }
                (valid, threat)
            },
            _ => move |_pos: &Vector, game: &Game| {
                let (valid, threat) = init_masks(&game.state);
                (valid, threat)
            },
        }
    }
    // functions must be called as move is being made,
    // before pieces are swapped, after player's recent_move field has been updated
    pub fn side_effects(&self) -> impl FnMut(&mut Game) {
        match self.id {
            // swap R position
            'K' => |game: &mut Game| {
                let player = game.current_player();
                let recent_move = player.recent_move.clone().unwrap();
                let start = recent_move.start.clone();
                let end = recent_move.end.clone();
                let displacement = end + start.clone() * -1;
                let is_castle = match displacement {
                    Vector(2, 0) | Vector(-2, 0) | Vector(0, 2) | Vector(0, -2) => true,
                    _ => false,
                };
                let direction = Vector(displacement.0 / 2, displacement.1 / 2);
                let find_rook = || {
                    let mut target = start.clone();
                    while game.get_piece(&target).id != 'R' {
                        target += direction.clone();
                    }
                    target
                };
                if is_castle {
                    let rook_pos = find_rook();
                    game.state.board[&(start + direction)] = game.state.board[&rook_pos].clone();
                    game.state.board[&rook_pos] = Piece::new();
                }
            },
            // destroy en passant'ed pawn
            'P' => |game: &mut Game| {
                let player = game.current_player();
                let recent_move = player.recent_move.clone().unwrap();
                let end_pos = recent_move.end.clone();
                // recent_move guaranteed to be Some
                let is_capture = game.get_piece(&end_pos).id != ' ';
                if !is_capture && recent_move.is_diag() {
                    let unit_vec = player.direction.clone();
                    game.state.board[&(end_pos + unit_vec * -1)] = Self::new();
                }
            },
            _ => |_game: &mut Game| {},
        }
    }
    // called from each piece to attack a position, determines whether it is possible
    // returns true if piece is not blocked, false if piece is blocked
    // attacking an opposing piece marks it valid and returns false
    // attack an allied piece does not mark it valid and returns false
    pub fn attack(
        &self,
        pos: Vector,
        state: &GameState,
        valid: &mut Matrix<bool>,
        threat: Option<&mut Matrix<bool>>,
    ) -> bool {
        if !state.board.in_bounds(&pos) {
            //guard against out of bounds
            return false;
        };
        let is_occupied = state.board[&pos].id != ' ';
        let is_opposed = state.board[&pos].owner != self.owner;
        match (is_occupied, is_opposed) {
            (false, _) => {
                valid[&pos] = true;
                if threat.is_some() {
                    threat.unwrap()[&pos] = true;
                }
                true
            }
            (true, true) => {
                valid[&pos] = true;
                if threat.is_some() {
                    threat.unwrap()[&pos] = true;
                }
                false
            }
            (true, false) => {
                if threat.is_some() {
                    threat.unwrap()[&pos] = true;
                }
                false
            }
        }
    } // sequentially attack in a direction, common to Q R and B
    fn extend(
        &self,
        pos: &Vector,
        directions: Vec<Vector>,
        state: &GameState,
        valid: &mut Matrix<bool>,
        threat: &mut Matrix<bool>,
    ) {
        for direction in directions {
            let mut target = pos.clone();
            loop {
                target += direction.clone();
                if !self.attack(target.clone(), state, valid, Some(threat)) {
                    break;
                }
            }
        }
    }
}
