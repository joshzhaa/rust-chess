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
                        if i != 0 || j != 0 {
                            game.attack(&(pos.clone() + Vector(i, j)));
                        }
                    }
                }
            },
            'Q' => |pos: &Vector, game: &mut Game| {
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i != 0 || j != 0 {
                            extend(pos, Vector(i, j), game);
                        }
                    }
                }
            },
            'R' => |pos: &Vector, game: &mut Game| {
                extend(pos, Vector(0, 1), game); // North
                extend(pos, Vector(1, 0), game); // East
                extend(pos, Vector(0, -1), game); // South
                extend(pos, Vector(-1, 0), game); // West
            },
            'B' => |pos: &Vector, game: &mut Game| {
                println!("1, 1");
                extend(pos, Vector(1, 1), game); // NE
                println!("1, -1");
                extend(pos, Vector(1, -1), game); // SE
                println!("-1, -1");
                extend(pos, Vector(-1, -1), game); // SW
                println!("-1, 1");
                extend(pos, Vector(-1, 1), game); // NW
            },
            'N' => |pos: &Vector, game: &mut Game| {
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
                    game.attack(&(pos.clone() + offset));
                }
            },
            'P' => |pos: &Vector, game: &mut Game| {
                // most pieces don't capture self, capturing self would change closure signature,
                // not sure if there is a better workaround in the language
                let piece = game.get_piece(pos);
                let has_moved = piece.has_moved;
                let unit_vec = game.get_player(piece.owner).direction.clone();
                game.attack(&(pos.clone() + unit_vec.clone()));
                if !has_moved {
                    game.attack(&(pos.clone() + unit_vec.clone() + unit_vec)); // no need for scalar mult impl
                }
            },
            _ => |_pos: &Vector, _game: &mut Game| {},
        }
    }
}

// sequentially attack in a direction, common to Q R and B
fn extend(pos: &Vector, direction: Vector, game: &mut Game) {
    println!("BEGIN {:?} {:?}", pos, direction);
    let mut target = pos.clone();
    loop {
        println!("{:?}", target);
        target += direction.clone();
        println!("{:?}", target);
        if !game.attack(&target) {
            break;
        }
    }
}
