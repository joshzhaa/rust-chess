mod game;
mod board;
mod piece;

use std::io::{self, BufRead};

fn parse(input: Vec<u8>) -> board::Vector {
    let file = input[0] - ('a' as u8);
    let rank = input[1] - ('1' as u8);
    board::Vector(file.into(), rank.into())
}

fn terminal_play() -> io::Result<()> {
    let stdin = io::stdin();
    let mut game = game::Game::new();
    game.draw();
    println!("Input a selection");
    loop {
        let input = stdin.lock().lines().next().unwrap().unwrap();
        if input == "quit" {
            break Ok(());
        }
        let target = parse(input.into_bytes());
        game.select(target);
        game.draw();
    }
}

fn main() -> io::Result<()> {
    terminal_play()
}
