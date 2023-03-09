use crate::board::{Board, Matrix, Vector};
use crate::piece::Piece;

#[derive(Clone)]
pub struct Game {
    pub board: Board,     // pieces on the board
    valid: Matrix<bool>,  // boolean matrix of legal moves
    threat: Matrix<bool>, // boolean matrix of threatened squares (subtly diff from valid b/c pawns)
    history: Vec<Game>,
    players: Vec<Player>, // off by one, players[0] corresponds to player 1 (piece.owner 1)
    turn: u32,            // next player to move
    halfmove_counter: u32,
    selection: Option<Vector>, // position of Piece selected to be moved
}

// stores a move of some 'piece', from position 'start' to position 'end'
#[derive(Clone, Debug)]
pub struct Move {
    piece: Piece,
    pub start: Vector,
    pub end: Vector,
}

impl Move {
    // for identifying pawn double moves
    pub fn square_dist(&self) -> i32 {
        let displacement = self.start.clone() + self.end.clone() * -1;
        displacement.0 * displacement.0 + displacement.1 * displacement.1
    }
    pub fn is_diag(&self) -> bool {
        match self.end.clone() + self.start.clone() * -1 {
            Vector(1 | -1, 1 | -1) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Player {
    pub direction: Vector,
    pub recent_move: Option<Move>, // each player tracks most recent move for en passant
}

impl Game {
    // create standard chess game
    pub fn new() -> Game {
        Game {
            board: Board::standard(),
            valid: Matrix(vec![vec![false; 8]; 8]),
            threat: Matrix(vec![vec![false; 8]; 8]),
            history: Vec::new(),
            players: vec![
                Player {
                    direction: Vector(0, 1),
                    recent_move: None,
                },
                Player {
                    direction: Vector(0, -1),
                    recent_move: None,
                },
            ],
            turn: 1,
            halfmove_counter: 1,
            selection: None,
        }
    }
    pub fn get_piece(&self, pos: &Vector) -> &Piece {
        return &self.board[pos];
    }
    pub fn get_player(&self, player_id: u32) -> &Player {
        return &self.players[player_id as usize - 1];
    }
    pub fn current_player(&self) -> &Player {
        return &self.players[self.turn as usize - 1];
    }
    // called from each piece to attack a position, determines whether it is possible
    // returns true if piece is not blocked, false if piece is blocked
    // attacking an opposing piece marks it valid and returns false
    // attack an allied piece does not mark it valid and returns false
    pub fn attack(&mut self, pos: &Vector, threaten: bool) -> bool {
        assert!(self.selection.is_some()); //this should only be called when game has a selection
        if !self.board.in_bounds(&pos) {
            //guard against out of bounds
            return false;
        };
        let is_occupied = self.board[pos].id != ' ';
        let is_opposed =
            self.board[pos].owner != self.board[&self.selection.clone().unwrap()].owner;
        match (is_occupied, is_opposed) {
            (false, _) => {
                self.valid[pos] = true;
                if threaten {
                    self.threat[pos] = true;
                }
                true
            }
            (true, true) => {
                self.valid[pos] = true;
                if threaten {
                    self.threat[pos] = true;
                }
                false
            }
            (true, false) => false,
        }
    }
    pub fn show_moves(&mut self, pos: Vector) {
        let selected = &self.board[&pos];
        selected.validity_func()(&pos, self); // ask piece to mark the squares it wants
    }
    fn deselect(&mut self) {
        self.selection = None;
        for row in self.valid.0.iter_mut() {
            for col in row.iter_mut() {
                *col = false;
            }
        }
    }
    // if no selection readies piece to be moved updating selection, and calculating validity in validity matrix
    // if has selection, moves piece if allowed, otherwise deselects
    pub fn select(&mut self, pos: Vector) {
        if self.selection.is_none() {
            self.deselect();
            let target = &self.board[&pos]; //TODO: panics if out of bounds
            if target.id != ' ' && target.owner == self.turn {
                self.selection = Some(pos.clone());
                self.show_moves(pos);
            }
        } else {
            self.move_piece(pos);
        }
    }
    // moves piece at 'from' to position at 'to' erases piece at 'to' if occupied
    pub fn move_piece(&mut self, pos: Vector) {
        debug_assert!(self.selection.is_some());
        let correct_turn = self.turn == self.board[&self.selection.clone().unwrap()].owner;
        let is_legal = self.valid[&pos];
        if correct_turn && is_legal {
            self.move_piece_unchecked(&self.selection.clone().unwrap(), &pos);
        }
        self.deselect();
    }
    // moves piece at 'from' to position at 'to' erases piece at 'to' if occupied
    // doesn't check legalitly
    pub fn move_piece_unchecked(&mut self, from: &Vector, to: &Vector) {
        assert_ne!(self.board[from].id, ' ');
        // record current state in history
        self.history.push(self.clone());
        self.players[self.turn as usize - 1].recent_move = Some(Move {
            piece: self.board[from].clone(),
            start: from.clone(),
            end: to.clone(),
        });
        // perform move side effects (e.g. K castle and P en passant)
        self.board[from].side_effects()(self);
        // modify board by swapping pieces around
        self.board[to] = self.board[from].clone();
        self.board[from] = Piece {
            id: ' ',
            owner: 0,
            has_moved: false,
        };
        self.board[to].has_moved = true;
        // update turn counters
        self.turn = self.turn % (self.players.len() as u32) + 1;
        self.halfmove_counter += 1;
    }
    pub fn rewind(&mut self, halfmoves: u32) {
        let target_time = self.history.len() - halfmoves as usize;
        self.history.truncate(target_time + 1);
        let target = self.history.pop().unwrap();
        *self = target;
        self.deselect();
    }
    // print board to terminal for debug purposes
    pub fn draw(&self) {
        self.board.draw(&self.valid);
        let prev_player =
            &self.players[(self.turn as usize - 1 + self.players.len() - 1) % self.players.len()];
        match &prev_player.recent_move {
            Some(recent_move) => println!(
                "move: {}:{} from {} to {}",
                recent_move.piece.id,
                recent_move.piece.owner,
                recent_move.start.to_notation().unwrap(),
                recent_move.end.to_notation().unwrap(),
            ),
            None => (),
        };
        println!("player to move: {}", self.turn);
        println!("halfmove counter: {}", self.halfmove_counter);
    }
}
