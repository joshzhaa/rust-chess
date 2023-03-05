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
    pub fn validity_func(&self) -> impl FnMut(&Vector, &mut Game) {
        match self.id {
            'K' => |pos: &Vector, game: &mut Game| {
                for i in -1..=1 {
                    for j in -1..=1 {
                        game.attack(pos.clone() + Vector(i, j));
                    }
                }
            },
            _ => |_pos: &Vector, _game: &mut Game| {}
        }
    }
}
