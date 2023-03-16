use crate::board::{Board, Matrix, Vector};
use crate::piece::Piece;

pub struct Game {
    pub state: GameState,
    // specifically modified by rewinds
    history: Vec<GameState>,
    // temporary state, possibly not persistent
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

#[derive(Clone)]
pub struct Player {
    pub direction: Vector,
    pub recent_move: Option<Move>, // each player tracks most recent move for en passant
    state: State,
    pub king_pos: Vector,
}

#[derive(Clone)]
pub enum State {
    Check,     // K is threatened
    Stalemate, // K has no legal moves
    Checkmate, // K is threatened && K has no legal moves
    None,      // None of the above apply
}

// stores a move of some 'piece', from position 'start' to position 'end'
#[derive(Clone, Debug)]
pub struct Move {
    pub piece: Piece,
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
                        state: State::None,
                        king_pos: Vector(4, 0),
                    },
                    Player {
                        direction: Vector(0, -1),
                        recent_move: None,
                        state: State::None,
                        king_pos: Vector(4, 7),
                    },
                ],
            },
            history: Vec::new(),
            selection: None,
            valid: Matrix(vec![vec![false; 8]; 8]),
        }
    }
    pub fn current_player(&self) -> &Player {
        self.state.get_player(self.state.turn)
    }
    pub fn show_moves(&mut self, pos: Vector) {
        let selected = &self.state.board[&pos];
        let (valid, threat) = selected.claim_squares()(&pos, &self.state); // ask what squares it wants
        (self.valid, _) = selected.speculate(valid, threat, &pos, &self.state);
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
        self.state.board[from].side_effects()(from, &mut self.state);
        // modify board by swapping pieces around
        self.state.board[to] = self.state.board[from].clone();
        self.state.board[from] = Piece::new();
        self.state.board[to].has_moved = true;
        // update king_pos
        if self.state.board[to].id == 'K' {
            self.state.players[self.state.turn - 1].king_pos = to.clone();
        }
        // update turn counters
        self.state.turn = self.state.turn % self.state.players.len() + 1;
        self.state.halfmove_counter += 1;
        // update the end state of each player, enum State
        for player_id in 1..=self.state.players.len() {
            self.state.players[player_id - 1].state = self.state.update_check(player_id);
        }
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
        match self.current_player().state {
            State::Checkmate => println!("player {} is in checkmate!", self.state.turn),
            State::Check => println!("player {} is in check!", self.state.turn),
            State::Stalemate => println!("player {} is in stalemate!", self.state.turn),
            State::None => (),
        };
    }
}
impl GameState {
    pub fn get_piece(&self, pos: &Vector) -> &Piece {
        &self.board[pos]
    }
    pub fn get_player(&self, player_id: usize) -> &Player {
        &self.players[player_id - 1]
    }
    pub fn in_bounds(&self, pos: &Vector) -> bool {
        self.board.in_bounds(pos)
    }
    /*
     * Updates the end states of each player, defined in enum State
     */
    pub fn update_check(&self, player_id: usize) -> State {
        let (rows, cols) = self.board.shape();
        let mut is_threatened = false;
        let mut has_legal_moves = Vec::new();
        for row in 0..rows as i32 {
            for col in 0..cols as i32 {
                let pos = Vector(col, row);
                let piece = &self.board[&pos];
                if piece.owner == player_id {
                    let (valid, threat) = piece.claim_squares()(&pos, self);
                    let (valid, threat) = piece.speculate(valid, threat, &pos, self);
                    if piece.id == 'K' {
                        is_threatened = threat[&pos];
                    }
                    let can_move = valid.0.into_iter().flatten().any(|x| x);
                    println!("{:?} can {:?}", piece, can_move);
                    has_legal_moves.push(can_move);
                }
            }
        }
        println!("{:?}", is_threatened);
        println!("{:?}", has_legal_moves);
        let no_legal_moves = has_legal_moves.into_iter().all(|x| !x);
        println!("{:?}", no_legal_moves);
        match (is_threatened, no_legal_moves) {
            (true, true) => State::Checkmate,
            (true, false) => State::Check,
            (false, true) => State::Stalemate,
            (false, false) => State::None,
        }
    }
}
