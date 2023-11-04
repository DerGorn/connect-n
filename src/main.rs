use std::{
    error::Error,
    fmt::{Debug, Display},
};

mod cli_game;
mod gui_game;

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    gui_game::gui_game(2, 4, 7, 6)
    // cli_game::cli_game(2, 4, 7, 6)
}

fn take_turn(game: &mut Game, input_function: &mut dyn FnMut(&Game) -> Res<usize>) -> Res<bool> {
    let x = input_function(game)?;
    let y: usize;
    match game.place_piece(x) {
        Ok(n) => {
            y = n;
            game.end_turn();
            Ok(game.chech_win(x, y))
        }
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
struct Cell {
    occupance: u32,
}
impl Cell {
    fn new() -> Self {
        Cell { occupance: 0 }
    }

    fn is_empty(&self) -> bool {
        self.occupance == 0
    }

    fn occupy(&mut self, player: u32) {
        self.occupance = player + 1
    }
}
impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.occupance == other.occupance
    }
}
impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.occupance)
    }
}

enum Direction {
    Left,
    TopLeft,
    Top,
    TopRight,
    Right,
    DownRight,
    Down,
    DownLeft,
}
impl Direction {
    fn to_tuple(&self) -> (i64, i64) {
        match self {
            Self::Left => (-1, 0),
            Self::TopLeft => (-1, 1),
            Self::Top => (0, 1),
            Self::TopRight => (1, 1),
            Self::Right => (1, 0),
            Self::DownRight => (1, -1),
            Self::Down => (0, -1),
            Self::DownLeft => (-1, -1),
        }
    }
}

struct Board {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}
impl Board {
    fn new(width: usize, height: usize) -> Self {
        Board {
            width,
            height,
            cells: vec![Cell::new(); width * height],
        }
    }

    fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(&self.cells[y * self.width + x])
        }
    }

    fn get_mut_cell(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(&mut self.cells[y * self.width + x])
        }
    }

    fn get_line_length(&self, x: usize, y: usize, direction: Direction) -> usize {
        let (x_dir, y_dir) = direction.to_tuple();
        let new_x = x as i64 + x_dir;
        let new_y = y as i64 + y_dir;
        if new_x < 0 || new_y < 0 || new_x >= self.width as i64 || new_y >= self.height as i64 {
            return 0;
        }
        let new_x = new_x as usize;
        let new_y = new_y as usize;
        if self.get_cell(x, y) == self.get_cell(new_x, new_y) {
            1 + self.get_line_length(new_x, new_y, direction)
        } else {
            0
        }
    }

    fn get_grid(&self) -> Vec<&[Cell]> {
        self.cells.chunks(self.width).rev().collect()
    }
}
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lines = self
            .cells
            .chunks(self.width)
            .rev()
            .fold(String::new(), |string, vec| {
                format!("{}\n{:?}", string, vec)
            });
        write!(f, "{}", lines)
    }
}

struct Game {
    board: Board,
    player_count: u32,
    connect_size: usize,
    active_player: u32,
}
impl Game {
    fn new(
        player_count: u32,
        connect_size: usize,
        board_width: usize,
        board_height: usize,
    ) -> Self {
        Game {
            board: Board::new(board_width, board_height),
            player_count,
            connect_size,
            active_player: 0,
            // players: vec![Player::new(); player_count.try_into().unwrap()],
        }
    }

    fn end_turn(&mut self) {
        self.active_player = (self.active_player + 1) % self.player_count
    }

    fn place_piece(&mut self, x: usize) -> Res<usize> {
        let mut placed_piece_y: i64 = -1;
        for y in 0..self.board.height {
            match self.board.get_mut_cell(x, y) {
                None => {}
                Some(cell) => {
                    if cell.is_empty() {
                        cell.occupy(self.active_player);
                        placed_piece_y = y as i64;
                        break;
                    }
                }
            }
        }
        if placed_piece_y != -1 {
            Ok(placed_piece_y as usize)
        } else {
            Err(format!("Column {} is allready full", x,).into())
        }
    }

    fn chech_win(&self, x: usize, y: usize) -> bool {
        1 + self.board.get_line_length(x, y, Direction::Down)
            + self.board.get_line_length(x, y, Direction::Top)
            >= self.connect_size
            || 1 + self.board.get_line_length(x, y, Direction::Left)
                + self.board.get_line_length(x, y, Direction::Right)
                >= self.connect_size
            || 1 + self.board.get_line_length(x, y, Direction::DownLeft)
                + self.board.get_line_length(x, y, Direction::TopRight)
                >= self.connect_size
            || 1 + self.board.get_line_length(x, y, Direction::DownRight)
                + self.board.get_line_length(x, y, Direction::TopLeft)
                >= self.connect_size
    }
}
