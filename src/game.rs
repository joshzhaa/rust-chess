use crate::board::{Board, Vector};
use crate::piece::Piece;

pub struct Game {
    board: Board,
    valid: Vec<Vec<bool>>,
    history: Vec<Move>,
    players: u32,
    turn: u32,
}

struct Move {
    state: Board,
    piece: Piece,
    position: Vector,
}

impl Game {
    pub fn new() -> Game {
        Game{
            board: Board::standard(),
            valid: vec![vec![false; 8]; 8],
            history: Vec::new(),
            players: 2,
            turn: 1,
        }
    }
    // moves piece at 'from' to position at 'to' erases piece at 'to' if occupied
    pub fn move_piece(&mut self, from: &Vector, to: &Vector) {
        assert_ne!(self.board[from], Piece{piece: ' ', owner: 0});
        // record current state in history
        self.history.push(Move{
            state: self.board.clone(),
            piece: self.board[from],
            position: to.clone(),
        });
        // modify board by swapping pieces around
        self.board[to] = self.board[from];
        self.board[from] = Piece{piece: ' ', owner: 0};
        self.turn = self.turn % self.players + 1;
    }
    pub fn rewind(&mut self, halfmoves: u32) {
        let target_time = self.history.len() - halfmoves as usize;
        self.history.truncate(target_time);
        let target = self.history.last().unwrap();
        self.board = target.state.clone();
        // to accommodate unsigned integer
        self.turn = (self.turn + self.players - halfmoves) % self.players;
    }
    // print board to terminal for debug purposes
    pub fn draw(&self) {
        let recent_move = self.history.last();
        match recent_move {
            Some(to_print) => println!("most recent move: {}{}",
                to_print.piece.piece,
                to_print.position.to_notation().unwrap()),
            None => println!("BEGIN"),
        }
        println!("{}", self.board);
    }
}
