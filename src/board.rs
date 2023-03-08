use crate::piece::Piece;
use std::fmt;
use std::ops::{Add, AddAssign, Index, IndexMut};

#[derive(Clone)]
pub struct Matrix<T>(pub Vec<Vec<T>>);

#[derive(Clone)]
pub struct Board(Matrix<Piece>);

// for indexing into Board as an (x, y) ordered pair
#[derive(Clone, Debug)]
pub struct Vector(pub i32, pub i32);

impl Index<&Vector> for Board {
    type Output = Piece;

    fn index(&self, pos: &Vector) -> &Self::Output {
        &self.0[pos]
    }
}

impl IndexMut<&Vector> for Board {
    fn index_mut(&mut self, pos: &Vector) -> &mut Self::Output {
        &mut self.0[pos]
    }
}

impl<T> Index<&Vector> for Matrix<T> {
    type Output = T;

    fn index(&self, pos: &Vector) -> &Self::Output {
        &self.0[pos.1 as usize][pos.0 as usize]
    }
}

impl<T> IndexMut<&Vector> for Matrix<T> {
    fn index_mut(&mut self, pos: &Vector) -> &mut Self::Output {
        &mut self.0[pos.1 as usize][pos.0 as usize]
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign<Vector> for Vector {
    fn add_assign(&mut self, rhs: Vector) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Board {
    pub fn new(rows: usize, cols: usize) -> Board {
        Board(Matrix(vec![
            vec![
                Piece {
                    id: ' ',
                    owner: 0,
                    has_moved: false
                };
                cols
            ];
            rows
        ]))
    }
    pub fn standard() -> Board {
        let mut board = Self::new(8, 8);
        let pieces = ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'];
        for (i, id) in pieces.into_iter().enumerate() {
            // fill 8th and 1st rank
            board.set(
                Piece {
                    id,
                    owner: 2,
                    has_moved: false,
                },
                7,
                i,
            );
            board.set(
                Piece {
                    id,
                    owner: 1,
                    has_moved: false,
                },
                0,
                i,
            );
        }
        for i in 0..8 {
            // fill 7th and 2nd rank
            board.set(
                Piece {
                    id: 'P',
                    owner: 2,
                    has_moved: false,
                },
                6,
                i,
            );
            board.set(
                Piece {
                    id: 'P',
                    owner: 1,
                    has_moved: false,
                },
                1,
                i,
            );
        }
        board
    }
    pub fn set(&mut self, piece: Piece, row: usize, col: usize) {
        self.0 .0[row][col] = piece;
    }
    pub fn in_bounds(&self, pos: &Vector) -> bool {
        pos.0 >= 0
            && pos.1 >= 0
            && (pos.0 as usize) < self.0 .0[0].len()
            && (pos.1 as usize) < self.0 .0.len()
    }
    pub fn draw(&self, highlighting: &Matrix<bool>) {
        for (row, hrow) in self.0 .0.iter().rev().zip(highlighting.0.iter().rev()) {
            for _col in row {
                print!("+-----");
            }
            print!("+\n");
            for hcol in hrow {
                let highlight = if *hcol { '*' } else { ' ' };
                print!("| {highlight} {highlight} ");
            }
            print!("|\n");
            for col in row {
                if col.id == ' ' {
                    print!("|     ");
                } else {
                    print!("| {}:{} ", col.id, col.owner);
                }
            }
            print!("|\n");
            for hcol in hrow {
                let highlight = if *hcol { '*' } else { ' ' };
                print!("| {highlight} {highlight} ");
            }
            print!("|\n");
        }
        for _col in &self.0 .0[0] {
            print!("+-----");
        }
        print!("+\n")
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.0 .0.iter().rev() {
            for _col in row {
                write!(f, "+-----")?;
            }
            write!(f, "+\n")?;
            for _col in row {
                write!(f, "|     ")?;
            }
            write!(f, "|\n")?;
            for col in row {
                write!(f, "| {}:{} ", col.id, col.owner)?;
            }
            write!(f, "|\n")?;
            for _col in row {
                write!(f, "|     ")?;
            }
            write!(f, "|\n")?;
        }
        for _col in &self.0 .0[0] {
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
