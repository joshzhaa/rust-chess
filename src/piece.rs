// each piece keeps track of what kind it is (K, Q, R, ...) and which player controls it (1, 2, 3, ...)
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Piece {
    pub piece: char,
    pub owner: u8,
}
