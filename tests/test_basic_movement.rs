// for some reason, rust doesnt support integration tests that just print output and compare to see
// if it is correct. this seems like it would be a very common use case, and I don't understand why
// cargo doesn't seem to support it. although its always possible i havent read enough of the book
mod game;
mod board;
mod piece;

fn main() {
    let mut game = game::Game::new();
    game.draw();
    let pos = board::Vector(5, 7);
    println!("{}", pos.to_notation().unwrap());
    game.move_piece_unchecked(&pos, &board::Vector(0, 0));
    game.draw();
    game.move_piece_unchecked(&board::Vector(1, 0), &board::Vector(1, 7));
    game.draw();
    game.move_piece_unchecked(&board::Vector(4, 0), &board::Vector(3, 0));
    game.draw();
    game.move_piece_unchecked(&board::Vector(4, 7), &board::Vector(3, 7));
    game.draw();
    println!("SELECTING");
    game.select(board::Vector(3, 0));
    game.draw();
    game.move_piece_unchecked(&board::Vector(3, 0), &board::Vector(3, 1));
    game.draw();
    println!("REWINDING");
    game.rewind(1);
    game.draw();
}
