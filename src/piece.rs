use crate::board::{Matrix, Vector};
use crate::game::{GameState, State};

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
    /*
     * Each piece id has an associated closure which marks all valid moves (return_tuple.0)
     * Each piece except for K declares which squares it threatens (return_tuple.1)
     * K instead declares which squares it is threatened by (for end-game condition calculation)
     */
    pub fn claim_squares(
        &self,
    ) -> impl FnOnce(&Vector, &GameState) -> (Matrix<bool>, Matrix<bool>) {
        match self.id {
            'K' => |pos: &Vector, game: &GameState| -> (Matrix<bool>, Matrix<bool>) {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game);
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
                            if game.board.in_bounds(&target) {
                                threatened[&target] = true;
                            }
                        }
                        return;
                    }
                    // handle non-king pieces
                    let (_, threat) = game.board[pos].claim_squares()(pos, game);
                    for row in 0..threat.0.len() as i32 {
                        for col in 0..threat.0[0].len() as i32 {
                            let pos = Vector(col, row);
                            threatened[&pos] = threatened[&pos] || threat[&pos];
                        }
                    }
                };
                for row in 0..game.board.0 .0.len() as i32 {
                    for col in 0..game.board.0 .0[0].len() as i32 {
                        let attacker = game.get_piece(&Vector(col, row));
                        let attacked = game.get_piece(pos);
                        if attacker.owner != attacked.owner {
                            or_assign(&Vector(col, row));
                        }
                    }
                }
                // mark standard moves
                for offset in offsets {
                    let target = pos.clone() + offset;
                    if game.board.in_bounds(&target) && !threatened[&target] {
                        this.attack(target, &game, &mut valid, Some(&mut threat));
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
                    this.attack(target, &game, &mut valid, Some(&mut threat));
                }
                if can_castle(&left) {
                    let target = pos.clone() + left * 2;
                    this.attack(target, &game, &mut valid, Some(&mut threat));
                }
                (valid, threatened)
            },
            'Q' => |pos: &Vector, game: &GameState| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game);
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
                this.extend(pos, directions, &game, &mut valid, &mut threat);
                (valid, threat)
            },
            'R' => |pos: &Vector, game: &GameState| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game);
                let directions = vec![Vector(0, 1), Vector(1, 0), Vector(0, -1), Vector(-1, 0)];
                this.extend(pos, directions, &game, &mut valid, &mut threat);
                (valid, threat)
            },
            'B' => |pos: &Vector, game: &GameState| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game);
                let directions = vec![
                    Vector(1, 1),   // NE
                    Vector(1, -1),  // SE
                    Vector(-1, -1), // SW
                    Vector(-1, 1),  // NW
                ];
                this.extend(pos, directions, &game, &mut valid, &mut threat);
                (valid, threat)
            },
            'N' => |pos: &Vector, game: &GameState| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game);
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
                    this.attack(target, &game, &mut valid, Some(&mut threat));
                }
                (valid, threat)
            },
            // pawn logic is fundamentally horrific
            'P' => |pos: &Vector, game: &GameState| {
                let this = game.get_piece(pos);
                let (mut valid, mut threat) = init_masks(&game);
                let unit_vec = game.get_player(this.owner).direction.clone();
                // initial double move logic
                let near = pos.clone() + unit_vec.clone();
                let far = pos.clone() + unit_vec.clone() * 2;
                let near_is_empty = game.in_bounds(&near) && game.get_piece(&near).id == ' ';
                let far_is_empty = game.in_bounds(&far) && game.get_piece(&far).id == ' ';
                if near_is_empty {
                    this.attack(near, &game, &mut valid, None);
                    if !this.has_moved && far_is_empty {
                        let far = pos.clone() + unit_vec.clone() * 2;
                        this.attack(far, &game, &mut valid, None);
                    }
                }
                // diagonal capture logic
                // 90 deg rotation matrix, useful misnomer, only correct if unit_vec is down
                let right = Vector(-1 * unit_vec.1, unit_vec.0);
                let left = right.clone() * -1;
                let diag_right = pos.clone() + unit_vec.clone() + right.clone();
                let diag_left = pos.clone() + unit_vec.clone() + left.clone();
                let can_diagonal = |pos: Vector| {
                    if !game.board.in_bounds(&pos) {
                        return false;
                    };
                    let diag_piece = game.get_piece(&pos);
                    diag_piece.id != ' ' && diag_piece.owner != this.owner
                };
                let can_right = can_diagonal(diag_right.clone());
                let can_left = can_diagonal(diag_left.clone());
                // en passant logic
                let can_passant = |pos: Vector| {
                    if !game.board.in_bounds(&pos) {
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
                        && passant_piece.owner != this.owner
                        && recent_move.square_dist() == 4
                        && recent_move.end == pos
                };
                let right_passant = can_passant(pos.clone() + right.clone());
                let left_passant = can_passant(pos.clone() + left.clone());
                // need to always threaten diag squares to communicate that enemy K cannot move there
                if game.board.in_bounds(&diag_right) {
                    threat[&diag_right] = true;
                }
                if game.board.in_bounds(&diag_left) {
                    threat[&diag_left] = true;
                }
                // carry out attacks, needs local variables logic *then* attack due to mut borrow rules
                if can_right || right_passant {
                    this.attack(diag_right, &game, &mut valid, Some(&mut threat));
                }
                if can_left || left_passant {
                    this.attack(diag_left, &game, &mut valid, Some(&mut threat));
                }
                (valid, threat)
            },
            _ => |_pos: &Vector, game: &GameState| {
                let (valid, threat) = init_masks(&game);
                (valid, threat)
            },
        }
    }
    // functions must be called as move is being made,
    // before pieces are swapped, after player's recent_move field has been updated
    pub fn side_effects(&self) -> impl FnMut(&Vector, &mut GameState) {
        match self.id {
            // swap R position
            'K' => |pos: &Vector, game: &mut GameState| {
                let player = game.get_player(game.get_piece(pos).owner);
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
                    game.board[&(start + direction)] = game.board[&rook_pos].clone();
                    game.board[&rook_pos] = Piece::new();
                }
            },
            // destroy en passant'ed pawn
            'P' => |pos: &Vector, game: &mut GameState| {
                let player = game.get_player(game.get_piece(pos).owner);
                let recent_move = player.recent_move.clone().unwrap();
                let end_pos = recent_move.end.clone();
                let unit_vec = player.direction.clone();
                // recent_move guaranteed to be Some
                let is_capture = game.get_piece(&end_pos).id != ' ';
                if !is_capture && recent_move.is_diag() {
                    game.board[&(end_pos.clone() + unit_vec.clone() * -1)] = Self::new();
                }
                // promotion
                if !game.in_bounds(&(end_pos + unit_vec)) {
                    game.board[&recent_move.start] = Piece {
                        id: 'Q', // TODO: ask for user input
                        owner: recent_move.piece.owner,
                        has_moved: true,
                    };
                }
            },
            _ => |_pos: &Vector, _game: &mut GameState| {},
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
    /*
     * speculates what would happen if self made each move in valid
     * unmarks as valid and threat if such a move would threaten friendly K
     */
    pub fn speculate(
        &self,
        mut valid: Matrix<bool>,
        mut threat: Matrix<bool>,
        start: &Vector,
        game: &GameState,
    ) -> (Matrix<bool>, Matrix<bool>) {
        let (rows, cols) = valid.shape();
        for row in 0..rows as i32 {
            for col in 0..cols as i32 {
                let end = Vector(col, row);
                if valid[&end] {
                    let mut speculation = game.clone();
                    // speculation.board[start].side_effects()(start, &mut speculation);
                    speculation.board[&end] = speculation.board[start].clone();
                    speculation.board[start] = Self::new();
                    let king_pos = &speculation.get_player(self.owner).king_pos;
                    let (_, threatened) =
                        speculation.board[&king_pos].claim_squares()(king_pos, &speculation);
                    if threatened[king_pos] {
                        valid[&end] = false;
                        threat[&end] = false;
                    }
                }
            }
        }
        (valid, threat)
    }
}
