mod game;
mod board;
mod piece;

use std::io::{self, BufRead};

fn parse(input: Vec<u8>) -> board::Vector {
    let file = input[0] - ('a' as u8);
    let rank = input[1] - ('1' as u8);
    board::Vector(file.into(), rank.into())
}

// repeatedly input squares in chess notation to interact with board
// also can input 'rewind 6' for example to rewind 6 halfmoves earlier
fn terminal_play() -> io::Result<()> {
    let mut history = Vec::new();
    let stdin = io::stdin();
    let mut game = game::Game::new();
    game.draw();
    println!("Input a selection");
    loop {
        let input = stdin.lock().lines().next().unwrap().unwrap();
        history.push(input.clone());
        if input == "quit" {
            println!("Input record:");
            for record in history {
                println!("{record}");
            }
            break Ok(());
        }
        if input == "rewind" {
            let input = stdin.lock().lines().next().unwrap().unwrap();
            let num = input.into_bytes()[0] - ('1' as u8);
            game.rewind(num.into());
        } else {
            let target = parse(input.into_bytes());
            game.select(target);
            game.draw();
        }
    }
}

fn main() -> io::Result<()> {
    terminal_play()
}
