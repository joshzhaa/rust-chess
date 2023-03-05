mod game;
mod board;
mod piece;

fn main() {
    let mut game = game::Game::new();
    game.draw();
    let pos = board::Vector(5, 7);
    println!("{}", pos.to_notation().unwrap());
    game.move_piece(&pos, &board::Vector(0, 0));
    game.draw();
    game.move_piece(&board::Vector(1, 0), &board::Vector(1, 7));
    game.draw();
    game.rewind(1);
    game.draw();
}
