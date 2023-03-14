use crate::board::{Board, Matrix, Vector};
use crate::piece::Piece;

#[derive(Clone)]
pub struct Game {
    pub state: GameState,
    // specifically modified by rewinds
    history: Vec<GameState>,
    // temporary state, reset on rewind
    selection: Option<Vector>, // position of Piece selected to be moved
    valid: Matrix<bool>,       // boolean matrix of legal moves
}

#[derive(Clone)]
pub struct GameState {
    pub board: Board, // pieces on the board
    turn: usize,      // next player to move
    halfmove_counter: u32,
    players: Vec<Player>, // off by one, players[0] corresponds to player 1 (piece.owner 1)
}

// stores a move of some 'piece', from position 'start' to position 'end'
#[derive(Clone, Debug)]
pub struct Move {
    piece: Piece,
    pub start: Vector,
    pub end: Vector,
}

#[derive(Clone)]
pub struct Player {
    pub direction: Vector,
    pub recent_move: Option<Move>, // each player tracks most recent move for en passant
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

impl Game {
    // create standard chess game
    pub fn new() -> Game {
        Game {
            state: GameState {
                board: Board::standard(),
                turn: 1,
                halfmove_counter: 1,
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
            },
            history: Vec::new(),
            selection: None,
            valid: Matrix(vec![vec![false; 8]; 8]),
        }
    }
    pub fn get_piece(&self, pos: &Vector) -> &Piece {
        &self.state.board[pos]
    }
    pub fn get_player(&self, player_id: usize) -> &Player {
        &self.state.players[player_id as usize - 1]
    }
    pub fn in_bounds(&self, pos: &Vector) -> bool {
        self.state.board.in_bounds(pos)
    }
    pub fn current_player(&self) -> &Player {
        return &self.state.players[self.state.turn as usize - 1];
    }
    pub fn show_moves(&mut self, pos: Vector) {
        let selected = &self.state.board[&pos];
        (self.valid, _) = selected.validity_func()(&pos, self); // ask piece to mark the squares it wants
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
            let target = &self.state.board[&pos]; //TODO: panics if out of bounds
            if target.id != ' ' && target.owner == self.state.turn {
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
        let is_turn = self.state.turn == self.state.board[&self.selection.clone().unwrap()].owner;
        let is_legal = self.valid[&pos];
        if is_turn && is_legal {
            self.move_piece_unchecked(&self.selection.clone().unwrap(), &pos);
        }
        self.deselect();
    }
    // moves piece at 'from' to position at 'to' erases piece at 'to' if occupied
    // doesn't check legalitly
    pub fn move_piece_unchecked(&mut self, from: &Vector, to: &Vector) {
        assert_ne!(self.state.board[from].id, ' ');
        // record current state in history
        self.history.push(self.state.clone());
        self.state.players[self.state.turn as usize - 1].recent_move = Some(Move {
            piece: self.state.board[from].clone(),
            start: from.clone(),
            end: to.clone(),
        });
        // perform move side effects (e.g. K castle and P en passant)
        self.state.board[from].side_effects()(self);
        // modify board by swapping pieces around
        self.state.board[to] = self.state.board[from].clone();
        self.state.board[from] = Piece {
            id: ' ',
            owner: 0,
            has_moved: false,
        };
        self.state.board[to].has_moved = true;
        // update turn counters
        self.state.turn = self.state.turn % self.state.players.len() + 1;
        self.state.halfmove_counter += 1;
    }
    pub fn rewind(&mut self, halfmoves: u32) {
        let target_time = self.history.len() - halfmoves as usize;
        self.history.truncate(target_time + 1);
        self.state = self.history.pop().unwrap();
        self.deselect();
    }
    // print board to terminal for debug purposes
    pub fn draw(&self) {
        self.state.board.draw(&self.valid);
        let prev_turn = (self.state.turn + self.state.players.len() - 2) % self.state.players.len();
        let prev_player = &self.state.players[prev_turn];
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
        println!("player to move: {}", self.state.turn);
        println!("halfmove counter: {}", self.state.halfmove_counter);
    }
}
