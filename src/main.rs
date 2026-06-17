use std::{
    io::{Stdout, Write, stdout},
    time::Duration,
    vec,
};

use crossterm::{
    cursor::{self, MoveTo},
    event::{
        self,
        Event::{self},
        KeyCode,
    },
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

const CELL_WIDTH: u16 = 2;
const CELL_HEIGHT: u16 = 1;

struct Grid {
    width: u16,
    height: u16,
    cells: Vec<Vec<u8>>,
}

impl Grid {
    fn new(width: u16, height: u16) -> Self {
        let mut cells = vec![vec![0u8; width.into()]; height.into()];

        for x in 0..width.min(7) {
            cells[0][x as usize] = (x + 1) as u8;
        }

        Self {
            width,
            height,
            cells,
        }
    }

    fn draw(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        let (term_width, term_height) = terminal::size()?;
        let board_height = self.height * CELL_HEIGHT + 1;
        let board_width = self.width * CELL_WIDTH + 2;
        let board_left = 0;
        let board_top = 0;

        let border_left = board_left;

        let start_y = term_height / 2 - board_height / 2;
        let start_x = term_width / 2 - board_width / 2;

        let cells_left = board_left + 1;
        let cells_top = board_top + 1;

        for y in 0..self.height {
            for x in 0..self.width {
                let terminal_x = cells_left + x * CELL_WIDTH;
                let terminal_y = cells_top + y * CELL_HEIGHT;
                let cell = self.cells[y as usize][x as usize];

                let tile = if cell != 0 { "[]" } else { "  " };
                // let color = match cell {
                //     1 => Color::Rgb { r: 91, g: 206, b: 250 },
                //     2 => Color::Rgb { r: 92, g: 124, b: 250 },
                //     3 => Color::Rgb { r: 250, g: 222, b: 91 },
                //     4 => Color::Rgb { r: 94, g: 214, b: 137 },
                //     5 => Color::Rgb { r: 190, g: 126, b: 240 },
                //     6 => Color::Rgb { r: 240, g: 96, b: 113 },
                //     7 => Color::Rgb { r: 245, g: 163, b: 89 },
                //     _ => Color::Rgb { r: 210, g: 216, b: 224 },
                // };

                if x == border_left {
                    queue!(
                        stdout,
                        MoveTo(terminal_x - 1 + start_x, terminal_y + start_y),
                        Print("|")
                    )?;
                }

                if x == self.width - 1 {
                    queue!(
                        stdout,
                        MoveTo(terminal_x + CELL_WIDTH + start_x, terminal_y + start_y),
                        Print("|")
                    )?;
                }

                if y == self.height - 1 {
                    queue!(
                        stdout,
                        MoveTo(terminal_x + start_x, terminal_y + CELL_HEIGHT + start_y),
                        Print("##")
                    )?;
                    queue!(
                        stdout,
                        MoveTo(terminal_x - 1 + start_x, terminal_y + CELL_HEIGHT + start_y),
                        Print("#")
                    )?;
                    queue!(
                        stdout,
                        MoveTo(
                            terminal_x + CELL_WIDTH + start_x,
                            terminal_y + CELL_HEIGHT + start_y
                        ),
                        Print("#")
                    )?;
                }
                queue!(
                    stdout,
                    MoveTo(terminal_x + start_x, terminal_y + start_y),
                    SetForegroundColor(Color::Yellow),
                    Print(tile),
                    ResetColor
                )?;
            }
        }
        Ok(())
    }
}

enum TetrominoKind {
    T,
    I,
    J,
    L,
    S,
    Z,
    O,
}

impl TetrominoKind {
    fn color(&self) -> Color {
        match self {
            TetrominoKind::I => Color::Rgb {
                r: 91,
                g: 206,
                b: 250,
            },
            TetrominoKind::O => Color::Rgb {
                r: 250,
                g: 222,
                b: 91,
            },
            TetrominoKind::T => Color::Rgb {
                r: 190,
                g: 126,
                b: 240,
            },
            TetrominoKind::S => Color::Rgb {
                r: 94,
                g: 214,
                b: 137,
            },
            TetrominoKind::Z => Color::Rgb {
                r: 240,
                g: 96,
                b: 113,
            },
            TetrominoKind::J => Color::Rgb {
                r: 92,
                g: 124,
                b: 250,
            },
            TetrominoKind::L => Color::Rgb {
                r: 245,
                g: 163,
                b: 89,
            },
        }
    }

    fn blocks(&self) -> [(i32, i32); 4] {
        match self {
            TetrominoKind::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
            TetrominoKind::O => [(1, 0), (2, 0), (1, 1), (2, 1)],
            TetrominoKind::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
            TetrominoKind::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
            TetrominoKind::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
            TetrominoKind::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
            TetrominoKind::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
        }
    }
}

struct Tetromino {
    kind: TetrominoKind,
    color: Color,
    blocks: [(i32, i32); 4],
}

impl Tetromino {
    fn new(kind: TetrominoKind) -> Self {
        Self {
            color: kind.color(),
            blocks: kind.blocks(),
            kind: kind
        }
    }
}

struct TetrominoBag {
    pieces: Vec<Tetromino>,
}

impl TetrominoBag {
    fn new() -> Self {
        Self {
            pieces: vec![
                Tetromino::new(TetrominoKind::T),
                Tetromino::new(TetrominoKind::I),
                Tetromino::new(TetrominoKind::J),
                Tetromino::new(TetrominoKind::L),
                Tetromino::new(TetrominoKind::S),
                Tetromino::new(TetrominoKind::Z),
                Tetromino::new(TetrominoKind::O),
            ]
        }
    }
}

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    let grid = Grid::new(10, 20);

    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        grid.draw(&mut stdout)?;
        stdout.flush()?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}
