use crate::piece::Piece;
use std::ops::{Index, IndexMut};
use std::fmt;

#[derive(Clone)]
pub struct Board (
    Vec<Vec<Piece>>
);
// for indexing into Board as an (x, y) ordered pair
#[derive(Clone)]
pub struct Vector (
    pub usize,
    pub usize,
);

impl Board {
    pub fn new(rows: usize, cols: usize) -> Board {
        Board(vec![vec![Piece{piece: ' ', owner: 0}; cols]; rows])
    }
    pub fn standard() -> Board {
        let mut board = Self::new(8, 8);
        let pieces = ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'];
        for (i, id) in pieces.into_iter().enumerate() { // fill 8th and 1st rank
            board.set(Piece{piece: id, owner: 2}, 7, i);
            board.set(Piece{piece: id, owner: 1}, 0, i);
        }
        for i in 0..8 { // fill 7th and 2nd rank
            board.set(Piece{piece: 'P', owner: 2}, 6, i);
            board.set(Piece{piece: 'P', owner: 1}, 1, i);
        }
        board
    }
    pub fn set(&mut self, piece: Piece, row: usize, col: usize) {
        self.0[row][col] = piece;
    }
}

impl Index<&Vector> for Board {
    type Output = Piece;
    
    fn index(&self, pos: &Vector) -> &Self::Output {
        &self.0[pos.1][pos.0]
    }
}

impl IndexMut<&Vector> for Board {
    fn index_mut(&mut self, pos: &Vector) -> &mut Self::Output {
        &mut self.0[pos.1][pos.0]
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.0.iter().rev() {
            for _col in row {
                write!(f, "+-----")?;
            }
            write!(f, "+\n")?;
            for _col in row {
                write!(f, "|     ")?;
            }
            write!(f, "|\n")?;
            for col in row {
                write!(f, "| {}:{} ", col.piece, col.owner)?;
            }
            write!(f, "|\n")?;
            for _col in row {
                write!(f, "|     ")?;
            }
            write!(f, "|\n")?;
        }
        for _col in &self.0[0] {
            write!(f, "+-----")?;
        }
        write!(f, "+\n")
    }
}

impl Vector {
    pub fn to_notation(&self) -> Result<String, &'static str> {
        if self.0 > 25 {
            return Err("cannot convert file coordinate to standard notation");
        };
        let file = (self.0 as u8 + b'a') as char;
        let rank = self.1 + 1;
        Ok(format!("{}{}", file, rank))
    }
}
