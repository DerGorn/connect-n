use std::io::stdin;

use crate::{take_turn, Game, Res};

pub fn cli_game(
    player_count: u32,
    connect_size: usize,
    board_width: usize,
    board_height: usize,
) -> Res<()> {
    let mut game = Game::new(player_count, connect_size, board_width, board_height);
    let mut game_over = false;
    while !game_over {
        loop {
            match take_turn(&mut game, &mut get_cli_input) {
                Ok(b) => {
                    game_over = b;
                    break;
                }
                Err(e) => println!("{}", e),
            }
        }
    }
    println!(
        "Player {} won!!{}",
        if game.active_player == 0 {
            game.player_count
        } else {
            game.active_player
        },
        game.board
    );
    Ok(())
}

fn get_cli_input(game: &Game) -> Res<usize> {
    println!(
        "Player {}, it is your turn.\nWhere do you want to place your piece?\n{}\n\ncolumn: ",
        game.active_player + 1,
        &game.board,
    );

    let mut buffer = String::new();
    let x: usize;

    loop {
        stdin().read_line(&mut buffer)?;
        match buffer.trim_end().parse::<usize>() {
            Ok(n) if n < game.board.width => {
                x = n;
                break;
            }
            _ => {
                buffer = String::new();
                println!(
                    "Invalid Column. Please input a whole number between 0 and {}",
                    game.board.width - 1
                )
            }
        }
    }

    Ok(x)
}
